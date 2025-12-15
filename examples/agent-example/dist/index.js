"use strict";
// Example agent JavaScript code
// This runs in the QuickJS sandbox and can call BAML functions via the runtime host
// BAML functions are exposed directly by name (e.g., SimpleGreeting, not __baml_invoke)
/**
 * Example agent function that uses BAML to generate a greeting
 * This function is called from the agent runner
 */
// Overload: can accept a string name or an object with name property
async function greetUser(nameOrObj) {
    // Extract name from string or object
    const name = typeof nameOrObj === 'string' ? nameOrObj : nameOrObj.name;
    // Call BAML function directly by name (exposed by runtime host)
    // The runtime host registers BAML functions as global functions
    return await SimpleGreeting({ name });
}
/**
 * Example agent function that chains multiple operations
 */
async function processUserRequest(userInput) {
    console.log("Processing user request:", userInput);
    // Call BAML function
    const greeting = await greetUser(userInput);
    return {
        input: userInput,
        greeting: greeting,
        processedAt: new Date().toISOString()
    };
}
// Export functions to global scope so they can be called by the agent runner
globalThis.greetUser = greetUser;
globalThis.processUserRequest = processUserRequest;
console.log("Example agent loaded successfully");
