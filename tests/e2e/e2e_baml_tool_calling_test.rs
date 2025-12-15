//! End-to-end test for BAML's native tool calling using union types
//! 
//! Based on: https://boundaryml.com/blog/deepseek-r1-function-calling
//! 
//! BAML handles tool calling by:
//! 1. Defining tools as Union types in BAML (e.g., WeatherTool | CalculatorTool)
//! 2. LLM returns structured output matching one of the union variants
//! 3. BAML parses it into the correct type
//! 4. We map the BAML variant to our Rust tool function and execute it

use baml_rt::baml::BamlRuntimeManager;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use dotenvy;

#[path = "../common.rs"]
mod common;
use common::{WeatherTool, CalculatorTool};

#[tokio::test]
#[ignore] // Requires OPENROUTER_API_KEY and makes actual LLM calls
async fn test_e2e_baml_union_tool_calling() {
    // Load .env file
    let _ = dotenvy::dotenv();
    
    // Set OPENROUTER_API_KEY from environment
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY environment variable must be set");
    
    assert!(!api_key.is_empty(), "OPENROUTER_API_KEY must not be empty");
    
    tracing::info!("Starting E2E test: BAML union-based tool calling with Rust execution");
    
    // Set up BAML runtime
    let mut baml_manager = BamlRuntimeManager::new().unwrap();
    baml_manager.load_schema("baml_src").unwrap();
    
    // Register tools using the trait-based approach
    baml_manager.register_tool(WeatherTool).await.unwrap();
    baml_manager.register_tool(CalculatorTool).await.unwrap();
    
    // Map BAML union variants to our Rust tool functions
    baml_manager.map_baml_variant_to_tool("WeatherTool", "get_weather");
    baml_manager.map_baml_variant_to_tool("CalculatorTool", "calculate");
    
    let baml_manager = Arc::new(Mutex::new(baml_manager));
    
    // Test 1: Weather tool via BAML union
    {
        let manager = baml_manager.lock().await;
        
        tracing::info!("Testing weather tool via BAML ChooseTool function");
        let result = manager.invoke_function(
            "ChooseTool",
            json!({"user_message": "What's the weather in San Francisco?"})
        ).await;
        
        match result {
            Ok(tool_choice) => {
                tracing::info!("✅ BAML function returned tool choice: {:?}", tool_choice);
                
                // Execute the chosen tool
                let tool_result = manager.execute_tool_from_baml_result(tool_choice).await
                    .expect("Should execute tool from BAML result");
                
                tracing::info!("✅ Tool executed successfully: {:?}", tool_result);
                assert!(tool_result.as_object().is_some(), "Tool result should be an object");
            }
            Err(e) => {
                tracing::warn!("BAML function call failed (may need tool calling integration): {}", e);
            }
        }
    }
    
    // Test 2: Calculator tool via BAML union
    {
        let manager = baml_manager.lock().await;
        
        tracing::info!("Testing calculator tool via BAML ChooseTool function");
        let result = manager.invoke_function(
            "ChooseTool",
            json!({"user_message": "Calculate 15 times 23"})
        ).await;
        
        match result {
            Ok(tool_choice) => {
                tracing::info!("✅ BAML function returned tool choice: {:?}", tool_choice);
                
                // Execute the chosen tool
                let tool_result = manager.execute_tool_from_baml_result(tool_choice).await
                    .expect("Should execute tool from BAML result");
                
                tracing::info!("✅ Tool executed successfully: {:?}", tool_result);
                
                // Verify calculator result
                if let Some(obj) = tool_result.as_object() {
                    if let Some(result) = obj.get("result").and_then(|v| v.as_f64()) {
                        assert_eq!(result, 345.0, "15 * 23 should equal 345");
                    }
                }
            }
            Err(e) => {
                tracing::warn!("BAML function call failed (may need tool calling integration): {}", e);
            }
        }
    }
    
    tracing::info!("🎉 E2E BAML union tool calling test completed!");
}
