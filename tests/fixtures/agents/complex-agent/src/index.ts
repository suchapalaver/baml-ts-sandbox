// Complex test agent with multiple functions

async function greetUser(name: string): Promise<string> {
  return await SimpleGreeting({ name });
}

async function processRequest(userInput: string): Promise<object> {
  console.log("Processing request:", userInput);
  
  const greeting = await greetUser(userInput);
  
  return {
    input: userInput,
    greeting: greeting,
    processedAt: new Date().toISOString(),
    status: "success"
  };
}

async function chainOperations(input: string): Promise<object> {
  const processed = await processRequest(input);
  return {
    ...processed,
    chained: true,
    steps: ["processRequest", "greetUser"]
  };
}

globalThis.greetUser = greetUser;
globalThis.processRequest = processRequest;
globalThis.chainOperations = chainOperations;

