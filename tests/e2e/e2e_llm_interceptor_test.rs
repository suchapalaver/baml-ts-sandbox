//! End-to-end test for LLM interception with actual BAML function execution
//!
//! This test verifies that LLM interceptors are called when executing BAML functions
//! that make actual LLM calls.

use baml_rt::{
    baml::BamlRuntimeManager,
    interceptor::{LLMInterceptor, LLMCallContext, InterceptorDecision},
    error::Result,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use dotenvy;

/// Test interceptor that logs LLM calls
struct E2ELLMLoggingInterceptor {
    calls: Arc<Mutex<Vec<LLMCallContext>>>,
    completions: Arc<Mutex<Vec<(LLMCallContext, bool, u64)>>>, // (context, success, duration_ms)
}

impl E2ELLMLoggingInterceptor {
    fn new() -> (Self, Arc<Mutex<Vec<LLMCallContext>>>, Arc<Mutex<Vec<(LLMCallContext, bool, u64)>>>) {
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
impl LLMInterceptor for E2ELLMLoggingInterceptor {
    async fn intercept_llm_call(&self, context: &LLMCallContext) -> Result<InterceptorDecision> {
        let mut calls = self.calls.lock().await;
        calls.push(context.clone());
        tracing::info!(
            client = context.client,
            model = context.model,
            function = context.function_name,
            "LLM interceptor: intercepting LLM call"
        );
        Ok(InterceptorDecision::Allow)
    }

    async fn on_llm_call_complete(
        &self,
        context: &LLMCallContext,
        result: &Result<Value>,
        duration_ms: u64,
    ) {
        let mut completions = self.completions.lock().await;
        let success = result.is_ok();
        completions.push((context.clone(), success, duration_ms));
        tracing::info!(
            client = context.client,
            model = context.model,
            function = context.function_name,
            success = success,
            duration_ms = duration_ms,
            "LLM interceptor: call completed"
        );
    }
}

#[tokio::test]
#[ignore] // Requires OPENROUTER_API_KEY and makes actual LLM calls
async fn test_e2e_llm_interceptor_with_baml_execution() {
    // Load .env file
    let _ = dotenvy::dotenv();
    
    // Set OPENROUTER_API_KEY from environment
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable must be set");
    
    assert!(!api_key.is_empty(), "OPENROUTER_API_KEY must not be empty");
    
    tracing::info!("E2E Test: LLM interceptor with actual BAML function execution");
    
    // Set up BAML runtime
    let mut baml_manager = BamlRuntimeManager::new().unwrap();
    baml_manager.load_schema("baml_src").unwrap();
    
    // Register an LLM interceptor
    let (interceptor, calls, completions) = E2ELLMLoggingInterceptor::new();
    baml_manager.register_llm_interceptor(interceptor).await;
    
    // Execute a BAML function that makes an LLM call
    tracing::info!("Calling SimpleGreeting BAML function (should trigger LLM interceptor)");
    let result = baml_manager.invoke_function(
        "SimpleGreeting",
        serde_json::json!({"name": "E2E Test"})
    ).await;
    
    // Verify the function executed successfully
    assert!(result.is_ok(), "BAML function should execute successfully");
    let greeting = result.unwrap();
    assert!(greeting.as_str().is_some(), "Result should be a string");
    
    // Give a moment for async completion notifications to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Verify interceptor was called
    let calls_guard = calls.lock().await;
    assert!(!calls_guard.is_empty(), "LLM interceptor should have been called");
    tracing::info!("✅ LLM interceptor was called {} time(s)", calls_guard.len());
    
    // Verify completion was notified
    let completions_guard = completions.lock().await;
    assert!(!completions_guard.is_empty(), "LLM interceptor should have received completion notification");
    tracing::info!("✅ LLM interceptor received {} completion notification(s)", completions_guard.len());
    
    // Verify the completion was successful
    if let Some((_, success, _)) = completions_guard.first() {
        assert!(*success, "LLM call should have completed successfully");
    }
    
    tracing::info!("🎉 E2E LLM interceptor test completed successfully!");
}
