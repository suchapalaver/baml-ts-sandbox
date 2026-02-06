use baml_rt_core::ids::{ArtifactId, ContextId, EventId, MessageId, TaskId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn next_event_id() -> EventId {
    let id = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    EventId::new(format!("prov-{}", id))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProvEventType {
    LlmCallStarted,
    LlmCallCompleted,
    ToolCallStarted,
    ToolCallCompleted,
    TaskCreated,
    TaskStatusChanged,
    TaskArtifactGenerated,
    MessageReceived,
    MessageSent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvEventData {
    LlmCall {
        client: String,
        model: String,
        function_name: String,
        prompt: Value,
        metadata: Value,
        duration_ms: Option<u64>,
        success: Option<bool>,
    },
    ToolCall {
        tool_name: String,
        function_name: Option<String>,
        args: Value,
        metadata: Value,
        duration_ms: Option<u64>,
        success: Option<bool>,
    },
    TaskCreated {
        task_id: TaskId,
        agent_type: Option<String>,
    },
    TaskStatusChanged {
        task_id: TaskId,
        old_status: Option<String>,
        new_status: Option<String>,
    },
    TaskArtifactGenerated {
        task_id: TaskId,
        artifact_id: Option<ArtifactId>,
        artifact_type: Option<String>,
    },
    Message {
        id: MessageId,
        role: String,
        content: Vec<String>,
        metadata: Option<HashMap<String, String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvEvent {
    pub id: EventId,
    pub event_type: ProvEventType,
    pub context_id: ContextId,
    pub task_id: Option<TaskId>,
    pub timestamp_ms: u64,
    pub data: ProvEventData,
}

impl ProvEvent {
    pub fn llm_call_started(
        context_id: ContextId,
        task_id: Option<TaskId>,
        client: String,
        model: String,
        function_name: String,
        prompt: Value,
        metadata: Value,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::LlmCallStarted,
            context_id,
            task_id,
            timestamp_ms: now_millis(),
            data: ProvEventData::LlmCall {
                client,
                model,
                function_name,
                prompt,
                metadata,
                duration_ms: None,
                success: None,
            },
        }
    }

    pub fn llm_call_completed(
        context_id: ContextId,
        task_id: Option<TaskId>,
        client: String,
        model: String,
        function_name: String,
        prompt: Value,
        metadata: Value,
        duration_ms: u64,
        success: bool,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::LlmCallCompleted,
            context_id,
            task_id,
            timestamp_ms: now_millis(),
            data: ProvEventData::LlmCall {
                client,
                model,
                function_name,
                prompt,
                metadata,
                duration_ms: Some(duration_ms),
                success: Some(success),
            },
        }
    }

    pub fn tool_call_started(
        context_id: ContextId,
        task_id: Option<TaskId>,
        tool_name: String,
        function_name: Option<String>,
        args: Value,
        metadata: Value,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::ToolCallStarted,
            context_id,
            task_id,
            timestamp_ms: now_millis(),
            data: ProvEventData::ToolCall {
                tool_name,
                function_name,
                args,
                metadata,
                duration_ms: None,
                success: None,
            },
        }
    }

    pub fn tool_call_completed(
        context_id: ContextId,
        task_id: Option<TaskId>,
        tool_name: String,
        function_name: Option<String>,
        args: Value,
        metadata: Value,
        duration_ms: u64,
        success: bool,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::ToolCallCompleted,
            context_id,
            task_id,
            timestamp_ms: now_millis(),
            data: ProvEventData::ToolCall {
                tool_name,
                function_name,
                args,
                metadata,
                duration_ms: Some(duration_ms),
                success: Some(success),
            },
        }
    }

    pub fn task_created(
        context_id: ContextId,
        task_id: TaskId,
        agent_type: Option<String>,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::TaskCreated,
            context_id,
            task_id: Some(task_id.clone()),
            timestamp_ms: now_millis(),
            data: ProvEventData::TaskCreated {
                task_id,
                agent_type,
            },
        }
    }

    pub fn task_status_changed(
        context_id: ContextId,
        task_id: TaskId,
        old_status: Option<String>,
        new_status: Option<String>,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::TaskStatusChanged,
            context_id,
            task_id: Some(task_id.clone()),
            timestamp_ms: now_millis(),
            data: ProvEventData::TaskStatusChanged {
                task_id,
                old_status,
                new_status,
            },
        }
    }

    pub fn task_artifact_generated(
        context_id: ContextId,
        task_id: TaskId,
        artifact_id: Option<ArtifactId>,
        artifact_type: Option<String>,
    ) -> Self {
        Self {
            id: next_event_id(),
            event_type: ProvEventType::TaskArtifactGenerated,
            context_id,
            task_id: Some(task_id.clone()),
            timestamp_ms: now_millis(),
            data: ProvEventData::TaskArtifactGenerated {
                task_id,
                artifact_id,
                artifact_type,
            },
        }
    }
}
