//! Golden tests for QuickJS value conversion invariants
//!
//! These tests verify that evaluate() correctly handles different JavaScript return types:
//! - Strings (JSON and non-JSON)
//! - Objects
//! - Arrays
//! - Primitives (numbers, booleans, null, undefined)
//! - Promises (async operations)
//!
//! Purpose: Ensure we maintain correct conversion semantics as we evolve the QuickJS bridge.

use baml_rt::baml::BamlRuntimeManager;
use baml_rt::quickjs_bridge::QuickJSBridge;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test that JSON.stringify results are correctly parsed
#[tokio::test]
async fn test_json_stringify_is_parsed() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = r#"
        JSON.stringify({
            name: "Alice",
            age: 30,
            active: true
        })
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: JSON.stringify results should be parsed into JSON objects
    assert!(
        result.is_object(),
        "JSON.stringify should return a parsed object"
    );
    assert_eq!(result["name"], "Alice");
    assert_eq!(result["age"], 30);
    assert_eq!(result["active"], true);
}

/// Test that direct object returns are correctly converted
#[tokio::test]
async fn test_direct_object_return() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = r#"
        ({
            status: "success",
            count: 42
        })
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: Direct object literals should be converted to JSON objects
    assert!(
        result.is_object(),
        "Direct object should return a parsed object"
    );
    assert_eq!(result["status"], "success");
    assert_eq!(result["count"], 42);
}

/// Test that arrays are correctly converted
#[tokio::test]
async fn test_array_return() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = "[1, 2, 3, 4, 5]";

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: Arrays should be converted to JSON arrays
    assert!(
        result.is_array(),
        "Array literal should return a JSON array"
    );
    assert_eq!(result.as_array().unwrap().len(), 5);
    assert_eq!(result[0], 1);
    assert_eq!(result[4], 5);
}

/// Test that primitives are correctly handled
#[tokio::test]
async fn test_primitive_returns() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    // Number
    let result = bridge.evaluate("42").await.unwrap();
    assert_eq!(result, 42);

    // Boolean
    let result = bridge.evaluate("true").await.unwrap();
    assert_eq!(result, true);

    // String (non-JSON) - wrapped in result object since it's not valid JSON
    let result = bridge.evaluate(r#""hello world""#).await.unwrap();
    // Note: Non-JSON strings get wrapped in {result: "..."} by evaluate()
    assert_eq!(result["result"], "hello world");
}

/// Test that null and undefined are handled
#[tokio::test]
async fn test_null_undefined() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    // null
    let result = bridge.evaluate("null").await.unwrap();
    assert!(result.is_null(), "null should be converted to JSON null");

    // undefined
    let result = bridge.evaluate("undefined").await.unwrap();
    assert!(
        result.is_null(),
        "undefined should be converted to JSON null"
    );
}

/// Test that IIFE returns work correctly
#[tokio::test]
async fn test_iife_returns() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = r#"
        (function() {
            return {
                computed: 2 + 2,
                nested: {
                    value: "test"
                }
            };
        })()
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: IIFEs should return their computed values
    assert!(result.is_object());
    assert_eq!(result["computed"], 4);
    assert_eq!(result["nested"]["value"], "test");
}

/// Test that typeof checks work correctly
#[tokio::test]
async fn test_typeof_checks() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = r#"
        JSON.stringify({
            hasConsole: typeof console !== 'undefined',
            hasFunction: typeof function(){} === 'function',
            hasUndefined: typeof undefined === 'undefined'
        })
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: typeof checks should work in the sandbox
    assert!(result.is_object());
    assert_eq!(result["hasConsole"], true, "console should be available");
    assert_eq!(result["hasFunction"], true, "function type should work");
    assert_eq!(
        result["hasUndefined"], true,
        "undefined should be detectable"
    );
}

/// Test that multi-line expressions work
#[tokio::test]
async fn test_multiline_expressions() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = r#"
        JSON.stringify({
            line1: "first",
            line2: "second",
            line3: "third"
        })
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: Multi-line code with proper formatting should work
    assert!(result.is_object());
    assert_eq!(result["line1"], "first");
    assert_eq!(result["line2"], "second");
    assert_eq!(result["line3"], "third");
}

/// Test that nested objects and arrays work
#[tokio::test]
async fn test_complex_nested_structures() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    let code = r#"
        ({
            users: [
                { name: "Alice", age: 30 },
                { name: "Bob", age: 25 }
            ],
            metadata: {
                total: 2,
                active: true
            }
        })
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // INVARIANT: Complex nested structures should be fully converted
    assert!(result.is_object());
    assert!(result["users"].is_array());
    assert_eq!(result["users"][0]["name"], "Alice");
    assert_eq!(result["users"][1]["age"], 25);
    assert_eq!(result["metadata"]["total"], 2);
    assert_eq!(result["metadata"]["active"], true);
}

/// Test error handling for invalid code
#[tokio::test]
async fn test_error_handling() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    // Syntax error
    let result = bridge.evaluate("{ invalid syntax }").await;
    assert!(result.is_err(), "Invalid syntax should return an error");

    // ReferenceError
    let result = bridge.evaluate("nonExistentVariable").await;
    assert!(result.is_err(), "Undefined variable should return an error");
}

/// Test that the fix for the IIFE wrapping issue is maintained
#[tokio::test]
async fn test_iife_wrapping_returns_value() {
    let manager = Arc::new(Mutex::new(BamlRuntimeManager::new().unwrap()));
    let mut bridge = QuickJSBridge::new(manager).await.unwrap();

    // This is the exact pattern that was failing before the fix
    let code = r#"
        JSON.stringify({
            exists: true,
            value: 42
        })
    "#;

    let result = bridge.evaluate(code).await.unwrap();

    // REGRESSION TEST: This used to return Null because the IIFE didn't return the value
    assert!(
        !result.is_null(),
        "REGRESSION: evaluate() returned Null instead of the actual value"
    );
    assert!(result.is_object(), "Should return a parsed JSON object");
    assert_eq!(result["exists"], true);
    assert_eq!(result["value"], 42);
}
