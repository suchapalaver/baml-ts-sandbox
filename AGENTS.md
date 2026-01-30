# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the Rust runtime, QuickJS bridge, tooling, and builder pipeline (see `src/builder/`).
- `src/bin/` holds CLI entry points: `baml-agent-builder` and `baml-agent-runner`.
- `tests/` is split into `unit/`, `integration/`, and `e2e/` with fixtures in `tests/fixtures/`.
- `examples/` includes sample agents (e.g., `examples/agent-example/`).
- Docs and design notes live in `docs/` plus root-level guides like `README.md`.

## Build, Test, and Development Commands
- `nix develop` enters the dev shell (required on NixOS; optional elsewhere).
- `cargo build --release` builds optimized binaries.
- `cargo fmt` formats Rust code; `cargo clippy --all-targets --all-features -- -D warnings` enforces linting.
- `cargo test` runs all tests; use `cargo test -- --nocapture` for verbose output.
- CLI examples:
  - `baml-agent-builder package --agent-dir ./my-agent --output agent.tar.gz`
  - `baml-agent-runner --package agent.tar.gz --function SimpleGreeting --args '{"name": "Alice"}'`

## Coding Style & Naming Conventions
- Rust formatting follows `rustfmt`; keep `clippy` clean (warnings as errors).
- Naming: `snake_case` for functions/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts.
- Prefer explicit error handling with `Result` and `?`; keep async code on `tokio`.

## Testing Guidelines
- Framework: Rust `cargo test`.
- Categories: unit (`tests/unit/`), integration (`tests/integration/`), e2e (`tests/e2e/`).
- Run a category with `cargo test --test unit` (or `integration`, `e2e`).
- E2E tests require `OPENROUTER_API_KEY` (see `.env` usage: `dotenv run cargo test --test e2e`).

## Commit & Pull Request Guidelines
- Commit messages follow Conventional Commits (e.g., `feat:`, `fix:`, `chore:`, `refactor:`).
- PRs should include: a concise summary, tests run (or reason skipped), and any relevant logs/screenshots.
- Link related issues or design notes (e.g., `PHASED_IMPLEMENTATION.md`) when applicable.

## Configuration & Secrets
- LLM access uses env vars like `OPENROUTER_API_KEY`, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`.
- Never commit secrets; use `.env` locally for e2e or integration scenarios.
