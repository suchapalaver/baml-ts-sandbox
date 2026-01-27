//! Tests for QuickJS bridge integration

use async_trait::async_trait;
use baml_rt::baml::BamlRuntimeManager;
use baml_rt::quickjs_bridge::QuickJSBridge;
use baml_rt::tools::BamlTool;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_quickjs_bridge_creation() {
    // Test that we can create a QuickJS bridge
    let baml_manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let bridge = QuickJSBridge::new(baml_manager);

    let bridge = bridge.await;
    assert!(bridge.is_ok(), "Should be able to create QuickJS bridge");
}

#[tokio::test]
async fn test_quickjs_evaluate_simple_code() {
    // Test that we can execute simple JavaScript code
    let baml_manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(baml_manager).await.unwrap();

    // Execute a simple JavaScript expression
    let result = bridge.evaluate("2 + 2").await;

    // The result might be a string representation or actual JSON
    // For now, just check that it doesn't error
    assert!(result.is_ok(), "Should be able to execute JavaScript code");
}

#[tokio::test]
async fn test_quickjs_evaluate_json() {
    // Test JSON stringify/parse
    let baml_manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(baml_manager).await.unwrap();

    // Execute code that returns a JSON object
    let result = bridge.evaluate("({answer: 42})").await;

    assert!(
        result.is_ok(),
        "Should be able to execute JavaScript and get JSON"
    );
}

#[tokio::test]
async fn test_quickjs_pure_js_promise_resolution() {
    // Test pure JavaScript promise resolution (no Rust async involved)
    let baml_manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(baml_manager).await.unwrap();

    // Execute code that returns a promise which resolves immediately
    let code = r#"
        (async function() {
            const result = await Promise.resolve({ value: 42 });
            return result;
        })()
    "#;
    let result = bridge.evaluate(code).await;

    assert!(
        result.is_ok(),
        "Should resolve pure JavaScript promises: {:?}",
        result
    );
    let value = result.unwrap();
    tracing::info!("Promise resolution result: {:?}", value);
}

#[tokio::test]
async fn test_quickjs_delayed_promise_resolution() {
    // Test JavaScript promise with slight delay
    let baml_manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(baml_manager).await.unwrap();

    // Execute code with a promise that resolves after a microtask tick
    let code = r#"
        (async function() {
            const result = await new Promise((resolve) => {
                resolve({ delayed: true, answer: 42 });
            });
            return result;
        })()
    "#;
    let result = bridge.evaluate(code).await;

    assert!(
        result.is_ok(),
        "Should resolve delayed JavaScript promises: {:?}",
        result
    );
    let value = result.unwrap();
    tracing::info!("Delayed promise result: {:?}", value);
}

/// Simple test tool for verifying tool invocation from JavaScript
struct SimpleAddTool;

#[async_trait]
impl BamlTool for SimpleAddTool {
    const NAME: &'static str = "simple_add";

    fn description(&self) -> &'static str {
        "Adds two numbers together"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["a", "b"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> baml_rt::error::Result<serde_json::Value> {
        let a = args["a"].as_i64().unwrap_or(0);
        let b = args["b"].as_i64().unwrap_or(0);
        Ok(json!({ "result": a + b }))
    }
}

#[tokio::test]
async fn test_quickjs_tool_invocation_from_js() {
    // THIS IS THE KEY TEST: JavaScript calling Rust async tools via __tool_invoke
    // This exercises the promise resolution issue where native Rust async functions
    // return promises that need to be properly awaited in JavaScript.

    // Set up manager and register tool
    let mut baml_manager = BamlRuntimeManager::new().unwrap();
    baml_manager.register_tool(SimpleAddTool).await.unwrap();
    let baml_manager = Arc::new(Mutex::new(baml_manager));

    // Create bridge and register tool functions (which registers __tool_invoke)
    let mut bridge = QuickJSBridge::new(baml_manager.clone()).await.unwrap();
    // Note: we can't call register_baml_functions() because that requires a loaded schema
    // So we call register_tool_functions() which is what we need for this test
    bridge.register_tool_functions().await.unwrap();

    // Verify __tool_invoke exists
    let check_code = r#"(function() { return JSON.stringify({ toolInvokeExists: typeof __tool_invoke === 'function', simpleAddExists: typeof simple_add === 'function' }); })()"#;
    let check_result = bridge.evaluate(check_code).await.unwrap();
    let check_obj = check_result.as_object().unwrap();
    assert!(
        check_obj
            .get("toolInvokeExists")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "__tool_invoke should be registered"
    );

    // Test basic promise resolution with constant return
    let simple_code = r#"
        (async function() {
            return { constant: 42 };
        })()
    "#;
    let simple_result = bridge.evaluate(simple_code).await;
    assert!(simple_result.is_ok(), "Simple async IIFE should succeed");
    let simple_value = simple_result.unwrap();
    assert_eq!(
        simple_value.get("constant").and_then(|v| v.as_i64()),
        Some(42)
    );

    // Test await on Promise.resolve
    let await_code = r#"
        (async function() {
            const p = Promise.resolve({ promised: 123 });
            const result = await p;
            return result;
        })()
    "#;
    let await_result = bridge.evaluate(await_code).await;
    assert!(await_result.is_ok(), "Await Promise.resolve should succeed");
    let await_value = await_result.unwrap();
    assert_eq!(
        await_value.get("promised").and_then(|v| v.as_i64()),
        Some(123)
    );

    // NOW THE ACTUAL TEST: Call the Rust tool from JavaScript
    // This requires the Promise returned by __tool_invoke (which wraps a Rust async function)
    // to properly resolve.
    let invoke_code = r#"
        (async function() {
            try {
                const result = await __tool_invoke("simple_add", JSON.stringify({a: 5, b: 3}));
                return result;
            } catch (e) {
                return { error: e.toString() };
            }
        })()
    "#;

    let result = bridge.evaluate(invoke_code).await;
    assert!(
        result.is_ok(),
        "Tool invocation from JavaScript should succeed: {:?}",
        result
    );

    let value = result.unwrap();
    // The result should contain { result: 8 } (5 + 3)
    let result_obj = value.as_object().expect("Expected object result");
    if let Some(error) = result_obj.get("error") {
        panic!("Tool invocation returned error: {}", error);
    }
    let sum = result_obj
        .get("result")
        .and_then(|v| v.as_i64())
        .expect("Expected 'result' field with number");
    assert_eq!(sum, 8, "5 + 3 should equal 8");
}
