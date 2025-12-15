# Test Organization

Tests are organized into three categories:

## Directory Structure

```
tests/
├── unit/              # Unit tests - test individual components in isolation
├── integration/       # Integration tests - test component interactions
├── e2e/              # End-to-end tests - test full system workflows
├── fixtures/         # Test fixtures (BAML files, agents, packages)
├── support/          # Test support code (tools, helpers)
└── common.rs         # Shared test utilities (fixture loading, etc.)
```

## Test Categories

### Unit Tests (`tests/unit/`)
Fast, isolated tests for individual components:
- `baml_runtime_test.rs` - BAML runtime manager tests
- `tool_registration_test.rs` - Tool registration tests
- `js_tool_registration_test.rs` - JavaScript tool registration
- `interceptor_test.rs` - Interceptor system tests
- `llm_interceptor_test.rs` - LLM interceptor tests
- `tracing_interceptor_test.rs` - Tracing interceptor tests

### Integration Tests (`tests/integration/`)
Test interactions between components:
- `baml_invoke_test.rs` - BAML function invocation
- `baml_execution_test.rs` - BAML execution engine
- `baml_compile_test.rs` - BAML compilation tests
- `quickjs_bridge_test.rs` - QuickJS bridge functionality
- `quickjs_baml_invoke_test.rs` - BAML invocation from JS
- `quickjs_baml_stream_test.rs` - Streaming BAML from JS
- `quickjs_sandbox_test.rs` - QuickJS sandboxing
- `cli_integration_test.rs` - CLI tool integration
- `integration_llm_interception_test.rs` - LLM interception integration

### E2E Tests (`tests/e2e/`)
Full system tests with real LLM calls (may require API keys):
- `e2e_llm_test.rs` - End-to-end LLM tests
- `e2e_tool_llm_test.rs` - Tool calling with LLM
- `e2e_llm_tool_calling_test.rs` - LLM tool calling
- `e2e_baml_tool_calling_test.rs` - BAML union tool calling
- `e2e_js_tool_llm_test.rs` - JavaScript tools with LLM
- `e2e_trait_tool_system_test.rs` - Trait-based tool system
- `e2e_llm_interceptor_test.rs` - LLM interceptor E2E
- `e2e_agent_runner_test.rs` - Agent runner E2E
- `agent_runner_test.rs` - Agent runner tests

## Using Fixtures

Tests can use fixtures via the `common` module:

```rust
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn test_example() {
    // Load BAML fixture
    let baml_path = common::baml_fixture("simple_prompt.baml");
    
    // Load agent fixture
    let agent_path = common::agent_fixture("minimal-agent");
    
    // Use fixtures in tests
}
```

## Test Tools

Test tool implementations are in `tests/support/tools.rs`:

```rust
#[path = "../../support/tools.rs"]
mod tool_examples;

use tool_examples::{WeatherTool, CalculatorTool};
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific category
cargo test --test unit
cargo test --test integration
cargo test --test e2e

# Run specific test
cargo test --test unit baml_runtime_test

# Run with output
cargo test -- --nocapture
```

## Fixtures

See `tests/fixtures/README.md` for fixture documentation.

