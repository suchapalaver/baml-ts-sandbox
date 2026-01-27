//! BAML Runtime Integration with QuickJS
//!
//! This crate provides integration between BAML compiled functions,
//! Rust execution runtime, and QuickJS JavaScript engine.

pub mod baml;
pub mod baml_collector;
pub mod baml_execution;
pub mod baml_pre_execution;
pub mod context;
pub mod error;
pub mod interceptor;
pub mod js_value_converter;
pub mod quickjs_bridge;
pub mod runtime;
pub mod spans;
pub mod tool_mapper;
pub mod tools;
pub mod traits;
pub mod types;

pub mod builder;
pub mod interceptors;

pub use context::{BamlContext, ContextMetadata};
pub use error::{BamlRtError, Result};
pub use interceptor::{
    InterceptorDecision, InterceptorRegistry, LLMCallContext, LLMInterceptor, ToolCallContext,
    ToolInterceptor,
};
pub use interceptors::{TracingInterceptor, TracingLLMInterceptor, TracingToolInterceptor};
pub use quickjs_bridge::QuickJSBridge;
pub use runtime::{QuickJSConfig, Runtime, RuntimeBuilder, RuntimeConfig};
