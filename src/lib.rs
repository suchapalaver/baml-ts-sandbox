//! BAML Runtime Integration with QuickJS
//!
//! This crate provides integration between BAML compiled functions,
//! Rust execution runtime, and QuickJS JavaScript engine.

pub mod baml;
pub mod spans;
pub mod baml_execution;
pub mod quickjs_bridge;
pub mod types;
pub mod error;
pub mod js_value_converter;
pub mod context;
pub mod tools;
pub mod tool_mapper;
pub mod runtime;
pub mod traits;
pub mod interceptor;
pub mod baml_collector;
pub mod baml_pre_execution;

pub mod interceptors;
pub mod builder;

pub use error::{BamlRtError, Result};
pub use runtime::{Runtime, RuntimeBuilder, RuntimeConfig, QuickJSConfig};
pub use interceptor::{
    InterceptorRegistry, InterceptorDecision, LLMInterceptor, ToolInterceptor,
    LLMCallContext, ToolCallContext,
};
pub use interceptors::{
    TracingInterceptor, TracingLLMInterceptor, TracingToolInterceptor,
};
pub use quickjs_bridge::QuickJSBridge;
pub use context::{BamlContext, ContextMetadata};

