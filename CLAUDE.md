# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

BAML Runtime is a Rust-based runtime for executing BAML (BoundaryML) functions with QuickJS integration. It enables agent systems that combine BAML prompts (for LLM interactions) with TypeScript/JavaScript application logic in a unified execution environment.

**Key Value**: Executes BAML functions in optimized Rust, exposes them to JavaScript/TypeScript via QuickJS, and provides tool registration and interceptor pipelines for governance and tracing.

## Prerequisites

Clone the BAML repository as a sibling directory:

```bash
git clone https://github.com/BoundaryML/baml.git ../baml
```

This provides the `baml-runtime`, `baml-types`, and `internal-baml-core` dependencies referenced in `Cargo.toml`.

## Development Environment

```bash
# Enter Nix dev shell (provides Rust, gcc, openssl, pkg-config)
nix develop

# Or with direnv
direnv allow
```

**NixOS Users**: All cargo commands (build, test, clippy, etc.) must be run inside the Nix development shell. If not using direnv, prefix commands with `nix develop --command`:

```bash
nix develop --command cargo fmt
nix develop --command cargo clippy --all-targets --all-features -- -D warnings
nix develop --command cargo test
```

## Build Commands

```bash
# Build
cargo build --release

# Format and lint
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test

# Run specific test categories
cargo test --test unit          # Unit tests (fast, isolated)
cargo test --test integration   # Integration tests
cargo test --test e2e           # E2E tests (requires OPENROUTER_API_KEY)

# Run single test file
cargo test --test tool_registration_test

# Run with output
cargo test -- --nocapture

# E2E tests with .env loading
dotenv run cargo test --test e2e

# Install CLI binaries
cargo install --path . --bins
```

## CLI Tools

```bash
# Lint TypeScript code
baml-agent-builder lint --agent-dir ./my-agent

# Package agent (compiles TS, generates types, creates tar.gz)
baml-agent-builder package --agent-dir ./my-agent --output agent.tar.gz

# Run agent package
baml-agent-runner --package agent.tar.gz
baml-agent-runner --package agent.tar.gz --function SimpleGreeting --args '{"name": "Alice"}'
```

## Architecture

### Execution Flow

```
TypeScript Code → QuickJS Bridge → BamlRuntimeManager → BamlExecutor → BamlRuntime → LLM Provider
                                        ↓
                              InterceptorRegistry (pre/post hooks)
                                        ↓
                              ToolRegistry (Rust & JS tools)
```

### Key Components

| File | Purpose |
|------|---------|
| `src/baml.rs` | Core runtime manager - function execution, tool registration, schema loading |
| `src/baml_execution.rs` | BAML IL execution engine with interceptor integration |
| `src/baml_pre_execution.rs` | Pre-execution validation and setup |
| `src/quickjs_bridge.rs` | QuickJS integration - JS ↔ Rust bridging, promise handling |
| `src/tools.rs` | Tool registry and `BamlTool` trait |
| `src/tool_mapper.rs` | Maps tools to BAML function definitions |
| `src/interceptor.rs` | Interceptor pipelines for LLM and tool calls |
| `src/interceptors/` | Built-in interceptors (tracing) |
| `src/context.rs` | `BamlContext` for request metadata propagation |
| `src/runtime.rs` | RuntimeBuilder with QuickJSConfig |
| `src/js_value_converter.rs` | Converts between Rust and QuickJS values |
| `src/builder/` | Build pipeline: compiler.rs (OXC TypeScript), linter.rs, packager.rs |

### Binary Targets

- `baml-agent-builder` (`src/bin/baml-agent-builder.rs`): CLI for building/packaging agents
- `baml-agent-runner` (`src/bin/baml-agent-runner.rs`): CLI for running agent packages

## Agent Structure

```
my-agent/
├── baml_src/          # BAML function definitions (*.baml)
├── src/               # TypeScript application code
├── dist/              # Generated JavaScript (created during build)
└── manifest.json      # Agent metadata: name, entry_point
```

## Design Patterns

**Trait-Based Tools**: Implement `BamlTool` trait for Rust tools

```rust
impl BamlTool for MyTool {
    const NAME: &'static str = "my_tool";
    fn description(&self) -> &'static str { ... }
    fn input_schema(&self) -> Value { ... }
    async fn execute(&self, args: Value) -> Result<Value> { ... }
}
```

**Interceptors**: `LLMInterceptor` and `ToolInterceptor` traits for pre/post execution hooks

**Builder Pattern**: `RuntimeBuilder` for constructing runtime with fluent API

## Test Organization

```
tests/
├── unit/           # Component isolation tests
├── integration/    # Component interaction tests
├── e2e/            # Full system tests (real LLM calls)
├── fixtures/       # Test BAML files and agents
└── support/        # Test helpers and example tools
```

Use fixtures via:
- `common::baml_fixture("file.baml")` - BAML schema files
- `common::agent_fixture("agent-name")` - Full agent directories
- `common::package_fixture("package.tar.gz")` - Pre-built packages
- `common::fixture_path("relative/path")` - Generic fixture access

## Environment Variables

- `OPENROUTER_API_KEY`: Required for E2E tests and OpenRouter LLM calls
- `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`: Alternative LLM providers

## Key Dependencies

- `baml-runtime`, `baml-types`, `internal-baml-core`: BAML Rust libraries (see Prerequisites)
- `quickjs_runtime`: Embedded JavaScript engine
- `oxc_*`: TypeScript compilation and linting
- `tokio`: Async runtime
- `tracing`: Structured logging
