use crate::a2a_types::{Message, StreamResponse, Task};
use baml_rt_core::{BamlRtError, Result};
use serde_json::Value;
use std::collections::HashMap;

pub trait StreamNormalizer: Send + Sync {
    fn normalize_chunk(&self, value: Value) -> Result<Value>;
    fn is_stream_response(&self, value: &Value) -> bool;
}

pub struct A2aStreamNormalizer;

impl StreamNormalizer for A2aStreamNormalizer {
    fn normalize_chunk(&self, value: Value) -> Result<Value> {
        if self.is_stream_response(&value) {
            return Ok(value);
        }
        if let Ok(message) = serde_json::from_value::<Message>(value.clone()) {
            let response = StreamResponse {
                message: Some(message),
                task: None,
                status_update: None,
                artifact_update: None,
                extra: HashMap::new(),
            };
            return serde_json::to_value(response).map_err(BamlRtError::Json);
        }
        if let Ok(task) = serde_json::from_value::<Task>(value.clone()) {
            let response = StreamResponse {
                task: Some(task),
                message: None,
                status_update: None,
                artifact_update: None,
                extra: HashMap::new(),
            };
            return serde_json::to_value(response).map_err(BamlRtError::Json);
        }
        Ok(value)
    }

    fn is_stream_response(&self, value: &Value) -> bool {
        let Some(map) = value.as_object() else {
            return false;
        };
        map.contains_key("message")
            || map.contains_key("task")
            || map.contains_key("statusUpdate")
            || map.contains_key("artifactUpdate")
    }
}
