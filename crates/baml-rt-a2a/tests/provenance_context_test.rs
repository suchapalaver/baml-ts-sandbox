use baml_rt_a2a::a2a_types::{JSONRPCId, JSONRPCRequest, Message, MessageRole, Part, SendMessageRequest, ROLE_USER};
use baml_rt_a2a::{A2aAgent, A2aRequestHandler};
use baml_rt::{BamlRuntimeManager, Result};
use baml_rt::tools::BamlTool;
use baml_rt_core::ids::{ContextId, MessageId};
use baml_rt_provenance::InMemoryProvenanceStore;
use baml_rt_provenance::{ProvEventData, ProvEventType};
use baml_rt_a2a::a2a_store::ProvenanceTaskStore;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use test_support::support::a2a::A2aInMemoryClient;
use async_trait::async_trait;
use serde_json::json;

fn user_message(message_id: &str, text: &str, context_id: Option<ContextId>) -> Message {
    Message {
        message_id: MessageId::from(message_id),
        role: MessageRole::String(ROLE_USER.to_string()),
        parts: vec![Part {
            text: Some(text.to_string()),
            ..Part::default()
        }],
        context_id,
        task_id: None,
        reference_task_ids: Vec::new(),
        extensions: Vec::new(),
        metadata: None,
        extra: HashMap::new(),
    }
}

async fn setup_agent(writer: Arc<InMemoryProvenanceStore>) -> A2aAgent {
    let js_code = r#"
        globalThis.handle_a2a_request = async function(request) {
            const params = request && request.params ? request.params : {};
            const ctx = params.message && params.message.contextId ? params.message.contextId : "missing";
            return {
                task: {
                    id: "task-ctx",
                    contextId: ctx,
                    status: { state: "TASK_STATE_WORKING" },
                    history: []
                }
            };
        };
    "#;
    let task_store = Arc::new(ProvenanceTaskStore::new(Some(writer.clone())));
    A2aAgent::builder()
        .with_task_store_backend(task_store)
        .with_provenance_writer(writer)
        .with_init_js(js_code)
        .build()
        .await
        .expect("agent build")
}

fn expect_context_id(responses: Vec<Value>) -> String {
    let response = responses.into_iter().next().expect("response");
    let result = response.get("result").cloned().expect("missing result");
    let task = result.get("task").and_then(Value::as_object).expect("task");
    task.get("contextId")
        .and_then(Value::as_str)
        .expect("contextId")
        .to_string()
}

#[tokio::test]
async fn test_context_id_propagates_across_agents() {
    let writer1 = Arc::new(InMemoryProvenanceStore::new());
    let writer2 = Arc::new(InMemoryProvenanceStore::new());
    let agent1 = setup_agent(writer1.clone()).await;
    let agent2 = setup_agent(writer2.clone()).await;

    let params = SendMessageRequest {
        message: user_message("msg-1", "hello", None),
        configuration: None,
        metadata: None,
        tenant: None,
        extra: HashMap::new(),
    };
    let request = JSONRPCRequest {
        jsonrpc: "2.0".to_string(),
        method: "message.send".to_string(),
        params: Some(serde_json::to_value(params).expect("serialize params")),
        id: Some(JSONRPCId::String("req-1".to_string())),
    };
    let request_value = serde_json::to_value(request).expect("serialize request");
    let responses = agent1.handle_a2a(request_value).await.expect("a2a handle");
    let context_id = expect_context_id(responses);

    let client = A2aInMemoryClient::new(Arc::new(agent2));
    let params = SendMessageRequest {
        message: user_message("msg-2", "forward", Some(ContextId::from(context_id.clone()))),
        configuration: None,
        metadata: None,
        tenant: None,
        extra: HashMap::new(),
    };
    let request = JSONRPCRequest {
        jsonrpc: "2.0".to_string(),
        method: "message.send".to_string(),
        params: Some(serde_json::to_value(params).expect("serialize params")),
        id: Some(JSONRPCId::String("req-2".to_string())),
    };
    let request_value = serde_json::to_value(request).expect("serialize request");
    let _ = client.send(request_value).await.expect("agent2 handle");

    let events = writer2.events().await;
    let context_id_typed = ContextId::from(context_id);
    assert!(
        events.iter().any(|event| event.context_id == context_id_typed),
        "expected provenance events to include propagated context_id"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_context_id_is_task_local_under_concurrency() {
    let writer = Arc::new(InMemoryProvenanceStore::new());
    let task_store = Arc::new(ProvenanceTaskStore::new(Some(writer.clone())));

    let js_code = r#"
        globalThis.handle_a2a_request = async function(request) {
            const text = request?.params?.message?.parts?.[0]?.text || "";
            return await invokeTool("echo_tool", { text });
        };
    "#;

    let mut runtime = BamlRuntimeManager::new().expect("runtime");
    runtime
        .register_tool(EchoTool)
        .await
        .expect("register echo tool");

    let agent = A2aAgent::builder()
        .with_task_store_backend(task_store)
        .with_provenance_writer(writer.clone())
        .with_runtime_manager(runtime)
        .with_init_js(js_code)
        .build()
        .await
        .expect("agent build");

    let context_ids: Vec<String> = (0..8).map(|i| format!("ctx-conc-{i}")).collect();
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let mut handles = Vec::new();
            for (idx, context_id) in context_ids.iter().enumerate() {
                let agent_clone = agent.clone();
                let request = JSONRPCRequest {
                    jsonrpc: "2.0".to_string(),
                    method: "message.send".to_string(),
                    params: Some(
                        serde_json::to_value(SendMessageRequest {
                            message: user_message(
                                &format!("msg-{idx}"),
                                "hello",
                                Some(ContextId::from(context_id.clone())),
                            ),
                            configuration: None,
                            metadata: None,
                            tenant: None,
                            extra: HashMap::new(),
                        })
                        .expect("serialize params"),
                    ),
                    id: Some(JSONRPCId::String(format!("req-{idx}"))),
                };
                let request_value = serde_json::to_value(request).expect("serialize request");
                handles.push(tokio::task::spawn_local(async move {
                    let _ = agent_clone
                        .handle_a2a(request_value)
                        .await
                        .expect("a2a handle");
                }));
            }

            for handle in handles {
                handle.await.expect("join");
            }
        })
        .await;

    let events = writer.events().await;
    for context_id in context_ids {
        let (starts, completes, successes) = tool_event_counts(&events, "echo_tool", &context_id);
        // Current invokeTool -> JS wrapper -> __tool_invoke path produces two tool calls per request.
        assert_eq!(
            starts, 2,
            "expected 2 tool starts for {context_id}, got {starts}"
        );
        assert_eq!(
            completes, 2,
            "expected 2 tool completions for {context_id}, got {completes}"
        );
        assert_eq!(
            successes, 2,
            "expected 2 tool successes for {context_id}, got {successes}"
        );
        assert_eq!(starts, completes, "pre/post pairing mismatch for {context_id}");
    }
}

#[derive(Debug)]
struct EchoTool;

#[async_trait]
impl BamlTool for EchoTool {
    const NAME: &'static str = "echo_tool";

    fn description(&self) -> &'static str {
        "Echo tool for concurrency testing."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, args: Value) -> Result<Value> {
        Ok(json!({ "echo": args }))
    }
}

fn tool_event_counts(
    events: &[baml_rt_provenance::ProvEvent],
    tool_name: &str,
    context_id: &str,
) -> (usize, usize, usize) {
    let mut starts = 0;
    let mut completes = 0;
    let mut successes = 0;
    let context_id = ContextId::from(context_id);
    for event in events {
        if event.context_id != context_id {
            continue;
        }
        match &event.data {
            ProvEventData::ToolCall { tool_name: name, success, .. } if name == tool_name => {
                match (event.event_type.clone(), success) {
                    (ProvEventType::ToolCallStarted, None) => starts += 1,
                    (ProvEventType::ToolCallCompleted, Some(result)) => {
                        completes += 1;
                        if *result {
                            successes += 1;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (starts, completes, successes)
}
