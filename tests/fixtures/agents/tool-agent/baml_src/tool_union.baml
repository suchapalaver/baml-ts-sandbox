// Tool calling using BAML's union type pattern
// Based on: https://boundaryml.com/blog/deepseek-r1-function-calling

// Define tools as class types
class WeatherTool {
  location string
}

class CalculatorTool {
  expression string
}

// Union type for tool choice
type ToolChoice = WeatherTool | CalculatorTool

// Function that returns which tool to call
function ChooseTool(user_message: string) -> ToolChoice {
  client DeepSeekOpenRouter
  prompt #"
    The user said: {{ user_message }}
    
    Determine which tool they need:
    - If they're asking about weather, return a WeatherTool with the location
    - If they're asking for a calculation, return a CalculatorTool with the expression
    
    Return ONLY the tool object, nothing else.
  "#
}

