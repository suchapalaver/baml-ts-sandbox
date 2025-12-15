// Test agent with tool calling support
async function callTool(toolName: string, args: Record<string, any>): Promise<any> {
  return await invokeTool(toolName, args);
}

async function useCalculator(expression: string): Promise<number> {
  const result = await callTool("calculate", { expression });
  return result.result || 0;
}

globalThis.callTool = callTool;
globalThis.useCalculator = useCalculator;

