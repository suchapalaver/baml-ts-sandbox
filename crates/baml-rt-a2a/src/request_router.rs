use crate::a2a;
use crate::handlers::TaskHandler;
use crate::result_pipeline::ResultStoragePipeline;
use crate::stream_normalizer::StreamNormalizer;
use async_trait::async_trait;
use baml_rt_core::{BamlRtError, Result};
use baml_rt_quickjs::QuickJSBridge;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait(?Send)]
pub trait JsInvoker: Send + Sync {
    async fn invoke_handler(&self, request: &a2a::A2aRequest) -> Result<Value>;
    async fn invoke_stream(&self, request: &a2a::A2aRequest) -> Result<Vec<Value>>;
}

pub struct QuickJsInvoker {
    bridge: Arc<Mutex<QuickJSBridge>>,
    stream_normalizer: Arc<dyn StreamNormalizer>,
}

impl QuickJsInvoker {
    pub fn new(
        bridge: Arc<Mutex<QuickJSBridge>>,
        stream_normalizer: Arc<dyn StreamNormalizer>,
    ) -> Self {
        Self {
            bridge,
            stream_normalizer,
        }
    }
}

#[async_trait(?Send)]
impl JsInvoker for QuickJsInvoker {
    async fn invoke_handler(&self, request: &a2a::A2aRequest) -> Result<Value> {
        let js_request = a2a::request_to_js_value(request);
        let mut bridge = self.bridge.lock().await;
        bridge.invoke_js_function("handle_a2a_request", js_request).await
    }

    async fn invoke_stream(&self, request: &a2a::A2aRequest) -> Result<Vec<Value>> {
        let result = self.invoke_handler(request).await?;
        match result {
            Value::Array(values) => values
                .into_iter()
                .map(|value| self.stream_normalizer.normalize_chunk(value))
                .collect::<Result<Vec<Value>>>(),
            Value::Object(map) if map.get("error").is_some() => Err(BamlRtError::QuickJs(
                map.get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string(),
            )),
            other => Ok(vec![self.stream_normalizer.normalize_chunk(other)?]),
        }
    }
}

#[async_trait(?Send)]
pub trait RequestRouter: Send + Sync {
    async fn route(&self, request: &a2a::A2aRequest) -> Result<a2a::A2aOutcome>;
}

pub struct MethodBasedRouter {
    task_handler: Arc<dyn TaskHandler>,
    js_invoker: Arc<dyn JsInvoker>,
    result_pipeline: Arc<dyn ResultStoragePipeline>,
}

impl MethodBasedRouter {
    pub fn new(
        task_handler: Arc<dyn TaskHandler>,
        js_invoker: Arc<dyn JsInvoker>,
        result_pipeline: Arc<dyn ResultStoragePipeline>,
    ) -> Self {
        Self {
            task_handler,
            js_invoker,
            result_pipeline,
        }
    }
}

#[async_trait(?Send)]
impl RequestRouter for MethodBasedRouter {
    async fn route(&self, request: &a2a::A2aRequest) -> Result<a2a::A2aOutcome> {
        match request.method {
            a2a::A2aMethod::TasksGet => {
                let req =
                    serde_json::from_value(request.params.clone()).map_err(BamlRtError::Json)?;
                self.task_handler.handle_get(req).await
            }
            a2a::A2aMethod::TasksList => {
                let req =
                    serde_json::from_value(request.params.clone()).map_err(BamlRtError::Json)?;
                self.task_handler.handle_list(req).await
            }
            a2a::A2aMethod::TasksCancel => {
                let req =
                    serde_json::from_value(request.params.clone()).map_err(BamlRtError::Json)?;
                self.task_handler.handle_cancel(req).await
            }
            a2a::A2aMethod::TasksSubscribe => {
                let req =
                    serde_json::from_value(request.params.clone()).map_err(BamlRtError::Json)?;
                self.task_handler.handle_subscribe(req, request.is_stream).await
            }
            _ => {
                if request.is_stream {
                    let chunks = self.js_invoker.invoke_stream(request).await?;
                    for chunk in &chunks {
                        self.result_pipeline.store_result(chunk).await?;
                    }
                    Ok(a2a::A2aOutcome::Stream(chunks))
                } else {
                    let result = self.js_invoker.invoke_handler(request).await?;
                    self.result_pipeline.store_result(&result).await?;
                    Ok(a2a::A2aOutcome::Response(result))
                }
            }
        }
    }
}
