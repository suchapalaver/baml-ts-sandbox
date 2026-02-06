use baml_rt_core::ids::ContextId;
use baml_rt_provenance::{InMemoryProvenanceStore, ProvEvent, ProvenanceWriter};
use serde_json::json;

#[tokio::test]
async fn test_in_memory_store_adds_events() {
    let store = InMemoryProvenanceStore::new();
    let event = ProvEvent::tool_call_started(
        ContextId::from("ctx-1"),
        None,
        "tool".to_string(),
        None,
        json!({"input": "value"}),
        json!({}),
    );

    store.add_event(event).await.expect("add event");
    let events = store.events().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].context_id, ContextId::from("ctx-1"));
}
