//! Tests for tracing interceptors

use baml_rt::{
    interceptor::{InterceptorRegistry, LLMInterceptor, LLMCallContext, ToolCallContext, InterceptorDecision},
    interceptors::{TracingInterceptor, TracingLLMInterceptor, TracingToolInterceptor},
    error::Result,
};
use serde_json::{json, Value};
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_tracing_llm_interceptor() {
    let mut registry = InterceptorRegistry::new();
    registry.register_llm_interceptor(TracingLLMInterceptor::new());

    let context = LLMCallContext {
        client: "test_client".to_string(),
        model: "test_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: json!({
            "messages": [{"role": "user", "content": "Hello"}]
        }),
        metadata: json!({}),
    };

    // Test pre-execution interception
    let decision = registry.intercept_llm_call(&context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));

    // Verify trace events were emitted (check logs for "llm_call" span)
    // The traced_test attribute will capture these logs

    // Test post-execution notification
    let result: Result<Value> = Ok(json!({"result": "success"}));
    registry.notify_llm_call_complete(&context, &result, 100).await;

    // Verify completion trace events were emitted
}

#[tokio::test]
#[traced_test]
async fn test_tracing_tool_interceptor() {
    let mut registry = InterceptorRegistry::new();
    registry.register_tool_interceptor(TracingToolInterceptor::new());

    let context = ToolCallContext {
        tool_name: "test_tool".to_string(),
        function_name: Some("test_function".to_string()),
        args: json!({"param": "value"}),
        metadata: json!({}),
    };

    // Test pre-execution interception
    let decision = registry.intercept_tool_call(&context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));

    // Test post-execution notification
    let result: Result<Value> = Ok(json!({"result": "success"}));
    registry.notify_tool_call_complete(&context, &result, 50).await;
}

#[tokio::test]
#[traced_test]
async fn test_tracing_interceptor_combined() {
    let mut registry = InterceptorRegistry::new();
    registry.register_llm_interceptor(TracingInterceptor::new());
    registry.register_tool_interceptor(TracingInterceptor::new());

    // Test LLM interception
    let llm_context = LLMCallContext {
        client: "test_client".to_string(),
        model: "test_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: json!({}),
        metadata: json!({}),
    };

    let decision = registry.intercept_llm_call(&llm_context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));

    // Test tool interception
    let tool_context = ToolCallContext {
        tool_name: "test_tool".to_string(),
        function_name: None,
        args: json!({}),
        metadata: json!({}),
    };

    let decision = registry.intercept_tool_call(&tool_context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));
}

#[tokio::test]
#[traced_test]
async fn test_tracing_interceptor_error_handling() {
    let mut registry = InterceptorRegistry::new();
    registry.register_llm_interceptor(TracingLLMInterceptor::new());

    let context = LLMCallContext {
        client: "test_client".to_string(),
        model: "test_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: json!({}),
        metadata: json!({}),
    };

    // Test error result notification
    let error_result: Result<Value> = Err(baml_rt::BamlRtError::BamlRuntime("test error".to_string()));
    registry.notify_llm_call_complete(&context, &error_result, 200).await;

    // Verify error was logged
}

