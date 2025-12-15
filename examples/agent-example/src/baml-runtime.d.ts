// TypeScript declarations for BAML runtime host functions
// This file is auto-generated - do not edit manually
// Generated from BAML schema

/**
 * SimpleGreeting BAML function
 */
declare function SimpleGreeting(args: {
  name: string
}): Promise<string>;

/**
 * Dynamically invoke a tool by name.
 * Works for both Rust-registered tools and JavaScript-registered tools.
 * 
 * @param toolName - The name of the tool to invoke
 * @param args - Arguments object for the tool
 * @returns Promise resolving to the tool's result
 * 
 * @example
 * // Invoke a Rust tool
 * const result = await invokeTool("get_weather", { location: "San Francisco" });
 * 
 * @example
 * // Invoke a JavaScript tool
 * const result = await invokeTool("formatText", { text: "Hello" });
 */
declare function invokeTool(toolName: string, args: Record<string, any>): Promise<any>;

/**
 * Low-level tool invocation helper (for Rust tools only).
 * Prefer using invokeTool() which handles both Rust and JavaScript tools.
 * 
 * @param toolName - The name of the Rust tool to invoke
 * @param argsJson - JSON-stringified arguments object
 * @returns Promise resolving to the tool's result
 * 
 * @internal
 */
declare function __tool_invoke(toolName: string, argsJson: string): Promise<any>;

