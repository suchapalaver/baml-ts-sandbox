use crate::a2a;
use crate::a2a_types::JSONRPCId;
use baml_rt_core::BamlRtError;
use serde_json::Value;

pub trait ResponseFormatter: Send + Sync {
    fn format_success(&self, id: Option<JSONRPCId>, result: Value) -> Value;
    fn format_stream(&self, id: Option<JSONRPCId>, chunks: Vec<Value>) -> Vec<Value>;
    fn format_error(&self, id: Option<JSONRPCId>, error: &BamlRtError) -> Value;
}

pub struct JsonRpcResponseFormatter;

impl ResponseFormatter for JsonRpcResponseFormatter {
    fn format_success(&self, id: Option<JSONRPCId>, result: Value) -> Value {
        a2a::success_response(id, result)
    }

    fn format_stream(&self, id: Option<JSONRPCId>, chunks: Vec<Value>) -> Vec<Value> {
        let total = chunks.len();
        let mut responses = Vec::with_capacity(total);
        for (idx, chunk) in chunks.into_iter().enumerate() {
            responses.push(a2a::stream_chunk_response(
                id.clone(),
                chunk,
                idx,
                idx + 1 == total,
            ));
        }
        responses
    }

    fn format_error(&self, id: Option<JSONRPCId>, error: &BamlRtError) -> Value {
        let (code, message, data) = map_jsonrpc_error(error);
        a2a::error_response(id, code, message, data)
    }
}

fn map_jsonrpc_error(error: &BamlRtError) -> (i64, &'static str, Option<Value>) {
    match error {
        BamlRtError::InvalidArgument(message) => (
            -32600,
            "Invalid request",
            Some(serde_json::json!({
                "error": error.to_string(),
                "details": message,
            })),
        ),
        BamlRtError::FunctionNotFound(name) => (
            -32601,
            "Method not found",
            Some(serde_json::json!({
                "error": error.to_string(),
                "function": name,
            })),
        ),
        BamlRtError::Json(json_err) => (
            -32700,
            "Parse error",
            Some(serde_json::json!({
                "error": error.to_string(),
                "details": json_err.to_string(),
            })),
        ),
        BamlRtError::QuickJsWithSource { context, .. } => (
            -32603,
            "Internal error",
            Some(serde_json::json!({
                "error": error.to_string(),
                "context": context,
            })),
        ),
        _ => (
            -32603,
            "Internal error",
            Some(serde_json::json!({
                "error": error.to_string(),
            })),
        ),
    }
}
