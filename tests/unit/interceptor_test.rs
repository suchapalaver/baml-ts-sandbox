//! Tests for interceptor system

use baml_rt::{
    baml::BamlRuntimeManager,
    interceptor::{LLMInterceptor, ToolInterceptor, InterceptorDecision, LLMCallContext, ToolCallContext},
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// Simple logging interceptor for LLM calls
struct LoggingLLMInterceptor;

#[async_trait]
impl LLMInterceptor for LoggingLLMInterceptor {
    async fn intercept_llm_call(&self, context: &LLMCallContext) -> baml_rt::Result<InterceptorDecision> {
        tracing::info!(
            client = context.client.as_str(),
            model = context.model.as_str(),
            function = context.function_name.as_str(),
            "Intercepted LLM call"
        );
        Ok(InterceptorDecision::Allow)
    }

    async fn on_llm_call_complete(
        &self,
        context: &LLMCallContext,
        result: &baml_rt::Result<Value>,
        duration_ms: u64,
    ) {
        tracing::info!(
            client = context.client.as_str(),
            function = context.function_name.as_str(),
            duration_ms = duration_ms,
            success = result.is_ok(),
            "LLM call completed"
        );
    }
}

/// Simple logging interceptor for tool calls
struct LoggingToolInterceptor;

#[async_trait]
impl ToolInterceptor for LoggingToolInterceptor {
    async fn intercept_tool_call(&self, context: &ToolCallContext) -> baml_rt::Result<InterceptorDecision> {
        tracing::info!(
            tool = context.tool_name.as_str(),
            args = ?context.args,
            "Intercepted tool call"
        );
        Ok(InterceptorDecision::Allow)
    }

    async fn on_tool_call_complete(
        &self,
        context: &ToolCallContext,
        result: &baml_rt::Result<Value>,
        duration_ms: u64,
    ) {
        tracing::info!(
            tool = context.tool_name.as_str(),
            duration_ms = duration_ms,
            success = result.is_ok(),
            "Tool call completed"
        );
    }
}

/// Blocking interceptor for tool calls (for testing)
struct BlockingToolInterceptor {
    blocked_tool: String,
}

#[async_trait]
impl ToolInterceptor for BlockingToolInterceptor {
    async fn intercept_tool_call(&self, context: &ToolCallContext) -> baml_rt::Result<InterceptorDecision> {
        if context.tool_name == self.blocked_tool {
            Ok(InterceptorDecision::Block(format!("Tool '{}' is blocked", self.blocked_tool)))
        } else {
            Ok(InterceptorDecision::Allow)
        }
    }

    async fn on_tool_call_complete(
        &self,
        _context: &ToolCallContext,
        _result: &baml_rt::Result<Value>,
        _duration_ms: u64,
    ) {
        // No-op
    }
}

#[tokio::test]
async fn test_register_and_use_tool_interceptor() {
    // This test demonstrates the interceptor system but requires actual tool registration
    // For now, we'll just verify the types compile correctly
    let mut manager = BamlRuntimeManager::new().unwrap();
    
    // Register a tool interceptor
    manager.register_tool_interceptor(LoggingToolInterceptor).await;
    
    // Verify interceptor was registered
    let registry = manager.interceptor_registry();
    let registry_guard = registry.lock().await;
    assert_eq!(registry_guard.tool_interceptors().len(), 1);
}

#[tokio::test]
async fn test_blocking_tool_interceptor() {
    let mut manager = BamlRuntimeManager::new().unwrap();
    
    // Register a blocking interceptor
    manager.register_tool_interceptor(BlockingToolInterceptor {
        blocked_tool: "blocked_tool".to_string(),
    }).await;
    
    // Verify interceptor was registered
    let registry = manager.interceptor_registry();
    let registry_guard = registry.lock().await;
    assert_eq!(registry_guard.tool_interceptors().len(), 1);
    
    // Test that the interceptor would block the tool
    let context = ToolCallContext {
        tool_name: "blocked_tool".to_string(),
        function_name: None,
        args: json!({}),
        metadata: json!({}),
    };
    
    let decision = registry_guard.intercept_tool_call(&context).await;
    assert!(decision.is_err());
}

