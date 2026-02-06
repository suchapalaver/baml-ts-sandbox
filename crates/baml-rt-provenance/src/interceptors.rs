use crate::events::ProvEvent;
use crate::store::ProvenanceWriter;
use async_trait::async_trait;
use baml_rt_core::Result;
use baml_rt_interceptor::{
    InterceptorDecision, LLMCallContext, LLMInterceptor, ToolCallContext, ToolInterceptor,
};
use serde_json::Value;
use std::sync::Arc;

pub struct ProvenanceInterceptor {
    writer: Arc<dyn ProvenanceWriter>,
}

impl ProvenanceInterceptor {
    pub fn new(writer: Arc<dyn ProvenanceWriter>) -> Self {
        Self { writer }
    }
}

#[async_trait]
impl LLMInterceptor for ProvenanceInterceptor {
    async fn intercept_llm_call(&self, context: &LLMCallContext) -> Result<InterceptorDecision> {
        let event = ProvEvent::llm_call_started(
            context.context_id.clone(),
            None,
            context.client.clone(),
            context.model.clone(),
            context.function_name.clone(),
            context.prompt.clone(),
            context.metadata.clone(),
        );
        self.writer
            .add_event_with_logging(event, "LLM call start")
            .await;
        Ok(InterceptorDecision::Allow)
    }

    async fn on_llm_call_complete(
        &self,
        context: &LLMCallContext,
        result: &Result<Value>,
        duration_ms: u64,
    ) {
        let success = result.is_ok();
        let event = ProvEvent::llm_call_completed(
            context.context_id.clone(),
            None,
            context.client.clone(),
            context.model.clone(),
            context.function_name.clone(),
            context.prompt.clone(),
            context.metadata.clone(),
            duration_ms,
            success,
        );
        self.writer
            .add_event_with_logging(event, "LLM call completion")
            .await;
    }
}

#[async_trait]
impl ToolInterceptor for ProvenanceInterceptor {
    async fn intercept_tool_call(&self, context: &ToolCallContext) -> Result<InterceptorDecision> {
        let event = ProvEvent::tool_call_started(
            context.context_id.clone(),
            None,
            context.tool_name.clone(),
            context.function_name.clone(),
            context.args.clone(),
            context.metadata.clone(),
        );
        self.writer
            .add_event_with_logging(event, "tool call start")
            .await;
        Ok(InterceptorDecision::Allow)
    }

    async fn on_tool_call_complete(
        &self,
        context: &ToolCallContext,
        result: &Result<Value>,
        duration_ms: u64,
    ) {
        let success = result.is_ok();
        let event = ProvEvent::tool_call_completed(
            context.context_id.clone(),
            None,
            context.tool_name.clone(),
            context.function_name.clone(),
            context.args.clone(),
            context.metadata.clone(),
            duration_ms,
            success,
        );
        self.writer
            .add_event_with_logging(event, "tool call completion")
            .await;
    }
}
