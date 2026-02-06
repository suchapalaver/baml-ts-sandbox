# baml-rt-quickjs

QuickJS-backed runtime host for BAML execution.

## Responsibilities
- `BamlRuntimeManager` orchestration for schema loading and function execution.
- `QuickJSBridge` integration to expose BAML functions to JavaScript.
- Context handling and JS value conversion utilities.

## Event Loop Polling

QuickJS timers and promise jobs advance only when the host polls the runtime.
If you start a long-running JS workflow without awaiting it via `evaluate()`,
call `QuickJSBridge::poll_event_loop()` periodically to keep timers/promises
progressing.
