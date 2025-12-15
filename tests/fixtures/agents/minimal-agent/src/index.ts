// Minimal test agent
async function greetUser(name: string): Promise<string> {
  return await SimpleGreeting({ name });
}

globalThis.greetUser = greetUser;

