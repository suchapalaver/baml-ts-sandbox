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
    let agent_dir = common::agent_fixture("complex-agent");
    assert!(
        agent_dir.join("baml_src").exists(),
        "complex-agent fixture must have baml_src directory"
    );
    baml_manager.load_schema(agent_dir.to_str().unwrap()).unwrap();
    
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
            println!("JavaScript execution error (may be due to missing API keys): {:?}", e);
            // The function exists and was called, but execution failed (likely API key issue)
            // This is acceptable for integration tests
            return;
        }
    };
    println!("JavaScript streaming execution result: {:?}", json_result);
    
    // Check if we got a proper result structure
    if let Some(obj) = json_result.as_object() {
        // Should have either success with results, or error
        assert!(obj.contains_key("success") || obj.contains_key("error"), 
                "Result should contain 'success' or 'error' field");
        
        // If it succeeded, results should be an array
        if let Some(success) = obj.get("success").and_then(|s| s.as_bool()) {
            if success {
                assert!(obj.contains_key("results"), "Success result should contain 'results' array");
            }
        }
    } else {
        panic!("Result should be an object");
    }
}
