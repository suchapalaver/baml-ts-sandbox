# Example Agent Package

This is an example agent package demonstrating how to create and use agents with the BAML runtime.

## Structure

- `baml_src/` - BAML schema definitions
- `src/` - TypeScript agent code
- `dist/` - Compiled JavaScript (generated)
- `manifest.json` - Agent metadata

## Usage

### Build the package

```bash
# Use the baml-agent-builder CLI to build and package
baml-agent-builder package \
  --agent-dir . \
  --output ../agent-packages/example-agent.tar.gz
```

This will:
1. Lint TypeScript source code
2. Generate TypeScript type declarations from BAML runtime
3. Compile TypeScript to JavaScript
4. Package everything into a tar.gz file

### Run the agent

```bash
# Load and list
baml-agent-runner example-agent.tar.gz

# Invoke a function
export OPENROUTER_API_KEY=your_key
baml-agent-runner example-agent.tar.gz --invoke example-agent SimpleGreeting '{"name":"World"}'
```

## BAML Functions

The runtime host automatically exposes BAML functions as global JavaScript functions. For example, if you have a BAML function `SimpleGreeting`, you can call it directly:

```typescript
// Call BAML function directly by name
const greeting = await SimpleGreeting({ name: "Alice" });
```

This is much cleaner than using `__baml_invoke("SimpleGreeting", JSON.stringify({ name: "Alice" }))`.

