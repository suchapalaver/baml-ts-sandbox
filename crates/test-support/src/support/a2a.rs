//! In-memory A2A test client.

use async_trait::async_trait;
use baml_rt::tools::BamlTool;
use baml_rt::{A2aRequestHandler, Result};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::task;

#[derive(Clone)]
pub struct A2aInMemoryClient {
    target: Arc<dyn A2aRequestHandler>,
}

impl A2aInMemoryClient {
    pub fn new(target: Arc<dyn A2aRequestHandler>) -> Self {
        Self { target }
    }

    pub async fn send(&self, request: Value) -> Result<Vec<Value>> {
        self.target.handle_a2a(request).await
    }
}

pub struct A2aRelayTool {
    client: A2aInMemoryClient,
}

impl A2aRelayTool {
    pub fn new(client: A2aInMemoryClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl BamlTool for A2aRelayTool {
    const NAME: &'static str = "a2a_relay";

    fn description(&self) -> &'static str {
        "Relays an A2A request to another in-memory agent."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "request": { "type": "object" }
            },
            "required": ["request"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        let request = args.get("request").cloned().unwrap_or_else(|| json!({}));
        let handle = tokio::runtime::Handle::current();
        let responses = task::block_in_place(|| handle.block_on(self.client.send(request)))?;
        Ok(json!({ "responses": responses }))
    }
}
