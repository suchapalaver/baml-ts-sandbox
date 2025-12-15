//! Tests for LLM interceptor system integration with BAML

use baml_rt::{
    interceptor::{InterceptorRegistry, LLMInterceptor, LLMCallContext, InterceptorDecision},
    error::Result,
    BamlRtError,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test interceptor that logs LLM calls
struct LoggingLLMInterceptor {
    calls: Arc<Mutex<Vec<LLMCallContext>>>,
    completions: Arc<Mutex<Vec<(LLMCallContext, Result<Value>, u64)>>>,
}

impl LoggingLLMInterceptor {
    fn new() -> (Self, Arc<Mutex<Vec<LLMCallContext>>>, Arc<Mutex<Vec<(LLMCallContext, Result<Value>, u64)>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let completions = Arc::new(Mutex::new(Vec::new()));
        let interceptor = Self {
            calls: calls.clone(),
            completions: completions.clone(),
        };
        (interceptor, calls, completions)
    }
}

#[async_trait::async_trait]
impl LLMInterceptor for LoggingLLMInterceptor {
    async fn intercept_llm_call(&self, context: &LLMCallContext) -> Result<InterceptorDecision> {
        let mut calls = self.calls.lock().await;
        calls.push(context.clone());
        Ok(InterceptorDecision::Allow)
    }

    async fn on_llm_call_complete(
        &self,
        context: &LLMCallContext,
        result: &Result<Value>,
        duration_ms: u64,
    ) {
        let mut completions = self.completions.lock().await;
        // Store a simplified representation since Result doesn't implement Clone
        // In a real scenario, we'd serialize/deserialize or use a different storage format
        let result_clone = result.as_ref().map(|v| v.clone());
        completions.push((context.clone(), result_clone.map(Ok).unwrap_or_else(|_| Err(baml_rt::BamlRtError::BamlRuntime("test error".to_string()))), duration_ms));
    }
}

/// Test interceptor that blocks LLM calls
struct BlockingLLMInterceptor;

#[async_trait::async_trait]
impl LLMInterceptor for BlockingLLMInterceptor {
    async fn intercept_llm_call(&self, context: &LLMCallContext) -> Result<InterceptorDecision> {
        // Block all LLM calls to "blocked_model"
        if context.model.contains("blocked") {
            Ok(InterceptorDecision::Block("Model is blocked".to_string()))
        } else {
            Ok(InterceptorDecision::Allow)
        }
    }

    async fn on_llm_call_complete(
        &self,
        _context: &LLMCallContext,
        _result: &Result<Value>,
        _duration_ms: u64,
    ) {
        // No-op for blocking interceptor
    }
}

#[tokio::test]
async fn test_llm_interceptor_logging() {
    // This test verifies that LLM interceptors are called and can log calls
    // Note: This is a unit test of the interceptor system, not a full E2E test
    // Full E2E tests would require actual BAML function execution with LLM calls
    
    let mut registry = InterceptorRegistry::new();
    let (interceptor, calls, completions) = LoggingLLMInterceptor::new();
    registry.register_llm_interceptor(interceptor);

    // Create a test context
    let context = LLMCallContext {
        client: "test_client".to_string(),
        model: "test_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: serde_json::json!({
            "messages": [{"role": "user", "content": "Hello"}]
        }),
        metadata: serde_json::json!({}),
    };

    // Test interception
    let decision = registry.intercept_llm_call(&context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));

    // Verify the call was logged
    let calls_guard = calls.lock().await;
    assert_eq!(calls_guard.len(), 1);
    assert_eq!(calls_guard[0].client, "test_client");
    assert_eq!(calls_guard[0].model, "test_model");

    // Test completion notification
    let result: Result<Value> = Ok(serde_json::json!({"result": "success"}));
    registry.notify_llm_call_complete(&context, &result, 100).await;

    // Verify completion was logged
    let completions_guard = completions.lock().await;
    assert_eq!(completions_guard.len(), 1);
    assert_eq!(completions_guard[0].2, 100); // duration_ms
}

#[tokio::test]
async fn test_llm_interceptor_blocking() {
    let mut registry = InterceptorRegistry::new();
    registry.register_llm_interceptor(BlockingLLMInterceptor);

    // Test blocking a call
    let blocked_context = LLMCallContext {
        client: "test_client".to_string(),
        model: "blocked_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: serde_json::json!({}),
        metadata: serde_json::json!({}),
    };

    let result = registry.intercept_llm_call(&blocked_context).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("blocked"));

    // Test allowing a call
    let allowed_context = LLMCallContext {
        client: "test_client".to_string(),
        model: "allowed_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: serde_json::json!({}),
        metadata: serde_json::json!({}),
    };

    let decision = registry.intercept_llm_call(&allowed_context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));
}

/// Test interceptor that verifies pre-execution interception
/// This demonstrates that interceptors are called before execution
struct PreExecutionVerifyingInterceptor {
    called: Arc<tokio::sync::Mutex<bool>>,
}

impl PreExecutionVerifyingInterceptor {
    fn new() -> (Self, Arc<tokio::sync::Mutex<bool>>) {
        let called = Arc::new(tokio::sync::Mutex::new(false));
        let interceptor = Self {
            called: called.clone(),
        };
        (interceptor, called)
    }
}

#[async_trait::async_trait]
impl LLMInterceptor for PreExecutionVerifyingInterceptor {
    async fn intercept_llm_call(&self, _context: &LLMCallContext) -> Result<InterceptorDecision> {
        let mut called = self.called.lock().await;
        *called = true;
        Ok(InterceptorDecision::Allow)
    }

    async fn on_llm_call_complete(
        &self,
        _context: &LLMCallContext,
        _result: &Result<Value>,
        _duration_ms: u64,
    ) {
        // No-op
    }
}

#[tokio::test]
async fn test_pre_execution_interception() {
    // This test verifies that the interceptor system works correctly
    // for pre-execution interception. In a real scenario, this would
    // be tested with actual BAML function execution, but we test the
    // interceptor registry directly here.
    
    let mut registry = InterceptorRegistry::new();
    let (interceptor, called_flag) = PreExecutionVerifyingInterceptor::new();
    registry.register_llm_interceptor(interceptor);

    // Simulate a pre-execution interception call
    let context = LLMCallContext {
        client: "test_client".to_string(),
        model: "test_model".to_string(),
        function_name: "test_function".to_string(),
        prompt: serde_json::json!({
            "messages": [{"role": "user", "content": "Hello"}]
        }),
        metadata: serde_json::json!({}),
    };

    // Call intercept_llm_call (simulating pre-execution)
    let decision = registry.intercept_llm_call(&context).await.unwrap();
    assert!(matches!(decision, InterceptorDecision::Allow));

    // Verify the interceptor was called
    let called = called_flag.lock().await;
    assert!(*called, "Pre-execution interceptor should have been called");
}

