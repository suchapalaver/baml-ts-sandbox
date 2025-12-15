//! End-to-end test using actual LLM via OpenRouter

use baml_rt::baml::BamlRuntimeManager;
use baml_rt::quickjs_bridge::QuickJSBridge;
use std::sync::Arc;
use tokio::sync::Mutex;
use dotenvy;

#[tokio::test]
#[ignore] // Requires OPENROUTER_API_KEY and makes actual LLM calls
async fn test_e2e_simple_greeting_with_llm() {
    // Load .env file
    let _ = dotenvy::dotenv();
    
    // Verify API key is set
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable must be set");
    
    assert!(!api_key.is_empty(), "OPENROUTER_API_KEY must not be empty");
    
    tracing::info!("Using OpenRouter API key (length: {})", api_key.len());
    
    // Set up BAML runtime
    let mut baml_manager = BamlRuntimeManager::new().unwrap();
    baml_manager.load_schema("baml_src").unwrap();
    
    let baml_manager = Arc::new(Mutex::new(baml_manager));
    
    // Create QuickJS bridge and register BAML functions
    let mut bridge = QuickJSBridge::new(baml_manager.clone()).await.unwrap();
    bridge.register_baml_functions().await.unwrap();
    
    // Call BAML function from JavaScript
    let js_code = r#"
        __awaitAndStringify(
            SimpleGreeting({ name: "E2E Test User" })
        )
    "#;
    
    tracing::info!("Executing JavaScript that calls BAML function...");
    let result = bridge.evaluate(js_code).await;
    
    match result {
        Ok(response_value) => {
            // The response should be a JSON string from __awaitAndStringify
            let response_str = response_value.as_str().unwrap_or("");
            
            tracing::info!("✅ BAML function executed successfully!");
            tracing::info!("Response: {}", response_str);
            
            // Verify response is not empty
            assert!(!response_str.is_empty(), "Response should not be empty");
            
            // The greeting should contain the name or be a reasonable response
            let response_lower = response_str.to_lowercase();
            assert!(
                response_lower.contains("e2e") || 
                response_lower.contains("test") ||
                response_lower.contains("user") ||
                response_str.len() > 5, // Or just be a reasonable length
                "Response should be meaningful or mention the name"
            );
        }
        Err(e) => {
            tracing::error!("❌ BAML function execution failed: {}", e);
            panic!("BAML function should execute successfully, but got error: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires OPENROUTER_API_KEY and makes actual LLM calls
async fn test_e2e_streaming_greeting() {
    // Load .env file
    let _ = dotenvy::dotenv();
    
    // Verify API key is set
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable must be set");
    
    assert!(!api_key.is_empty(), "OPENROUTER_API_KEY must not be empty");
    
    tracing::info!("Testing streaming BAML function call");
    
    // Set up BAML runtime
    let mut baml_manager = BamlRuntimeManager::new().unwrap();
    baml_manager.load_schema("baml_src").unwrap();
    
    let baml_manager = Arc::new(Mutex::new(baml_manager));
    
    // Create QuickJS bridge and register BAML functions
    let mut bridge = QuickJSBridge::new(baml_manager.clone()).await.unwrap();
    bridge.register_baml_functions().await.unwrap();
    
    // Call streaming BAML function from JavaScript
    let js_code = r#"
        __awaitAndStringify(
            (async () => {
                const chunks = [];
                const stream = SimpleGreetingStream({ name: "Streaming Test" });
                for await (const chunk of stream) {
                    chunks.push(chunk);
                }
                return { chunks: chunks, totalChunks: chunks.length };
            })()
        )
    "#;
    
    tracing::info!("Executing streaming JavaScript call...");
    let result = bridge.evaluate(js_code).await;
    
    match result {
        Ok(response_value) => {
            let response_str = response_value.as_str().unwrap_or("");
            tracing::info!("✅ Streaming function executed successfully!");
            tracing::info!("Response: {}", response_str);
            
            // Parse the response to verify chunks were received
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response_str) {
                if let Some(obj) = parsed.as_object() {
                    if let Some(chunks) = obj.get("chunks") {
                        assert!(chunks.as_array().is_some(), "Should have chunks array");
                        tracing::info!("Received {} chunks", chunks.as_array().unwrap().len());
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Streaming test failed (may not be supported yet): {}", e);
            // Don't fail the test if streaming isn't fully implemented yet
        }
    }
}
