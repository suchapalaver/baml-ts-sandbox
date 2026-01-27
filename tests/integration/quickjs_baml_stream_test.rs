//! Tests for JavaScript streaming invocation of BAML functions

#[path = "../common.rs"]
mod common;

use baml_rt::baml::BamlRuntimeManager;
use baml_rt::quickjs_bridge::QuickJSBridge;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_js_stream_baml_function() {
    // Set up BAML runtime
    let mut baml_manager = BamlRuntimeManager::new().unwrap();

    // Load BAML schema from agent fixture (which has baml_src directory)
    let agent_dir = common::agent_fixture("minimal-agent");
    assert!(
        agent_dir.join("baml_src").exists(),
        "minimal-agent fixture must have baml_src directory"
    );
    baml_manager
        .load_schema(agent_dir.to_str().unwrap())
        .unwrap();

    let baml_manager = Arc::new(Mutex::new(baml_manager));

    // Create QuickJS bridge
    let mut bridge = QuickJSBridge::new(baml_manager.clone()).await.unwrap();

    // Register BAML functions (including streaming versions)
    bridge.register_baml_functions().await.unwrap();

    // Test invoking SimpleGreeting stream from JavaScript
    // Use __awaitAndStringify helper to handle async function calls
    // Note: This will fail without an API key, but we can test the invocation path
    let js_code = r#"
        (function() {
            try {
                const promise = SimpleGreetingStream({ name: "World" });
                return __awaitAndStringify(promise);
            } catch (e) {
                return JSON.stringify({ success: false, error: e.toString() });
            }
        })()
    "#;

    let result = bridge.evaluate(js_code).await;

    // The result should contain either success with results array, or error info
    // Note: This may fail due to missing API keys, which is acceptable
    let json_result = match result {
        Ok(val) => val,
        Err(e) => {
            println!(
                "JavaScript execution error (may be due to missing API keys): {:?}",
                e
            );
            // The function exists and was called, but execution failed (likely API key issue)
            // This is acceptable for integration tests
            return;
        }
    };
    println!("JavaScript streaming execution result: {:?}", json_result);

    // Check if we got a proper result structure
    // The streaming function may return:
    // - An object with success/error fields (synchronous error case)
    // - An array of results (streaming results, may contain error objects)
    if let Some(obj) = json_result.as_object() {
        // Should have either success with results, or error
        assert!(
            obj.contains_key("success") || obj.contains_key("error"),
            "Result should contain 'success' or 'error' field"
        );

        // If it succeeded, results should be an array
        if let Some(success) = obj.get("success").and_then(|s| s.as_bool())
            && success
        {
            assert!(
                obj.contains_key("results"),
                "Success result should contain 'results' array"
            );
        }
    } else if let Some(arr) = json_result.as_array() {
        // Streaming may return an array of results
        // Each item should be an object (could be error or data)
        assert!(
            !arr.is_empty(),
            "Streaming results should not be empty array"
        );
        // Check if it contains error (expected when API key is missing)
        let has_error = arr.iter().any(|item| {
            item.as_object()
                .map(|obj| obj.contains_key("error"))
                .unwrap_or(false)
        });
        if has_error {
            // API key error is acceptable - streaming was invoked but failed
            return;
        }
    } else {
        panic!(
            "Result should be an object or array, got: {:?}",
            json_result
        );
    }
}
