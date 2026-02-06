use crate::a2a_store::TaskStoreBackend;
use crate::a2a_types::{
    Message, SendMessageResponse, StreamResponse, Task, TaskArtifactUpdateEvent,
    TaskStatusUpdateEvent,
};
use crate::events::EventEmitter;
use baml_rt_core::Result;
use std::sync::Arc;

pub struct TaskProcessor {
    task_store: Arc<dyn TaskStoreBackend>,
    emitter: Arc<dyn EventEmitter>,
}

impl TaskProcessor {
    pub fn new(task_store: Arc<dyn TaskStoreBackend>, emitter: Arc<dyn EventEmitter>) -> Self {
        Self {
            task_store,
            emitter,
        }
    }

    pub async fn process_stream_response(&self, stream: StreamResponse) -> Result<()> {
        self.process(
            stream.task,
            stream.message,
            stream.status_update,
            stream.artifact_update,
        )
        .await
    }

    pub async fn process_send_message_response(&self, response: SendMessageResponse) -> Result<()> {
        self.process(response.task, response.message, None, None)
            .await
    }

    pub async fn process_task(&self, task: Task) -> Result<()> {
        self.process(Some(task), None, None, None).await
    }

    async fn process(
        &self,
        task: Option<Task>,
        message: Option<Message>,
        status_update: Option<TaskStatusUpdateEvent>,
        artifact_update: Option<TaskArtifactUpdateEvent>,
    ) -> Result<()> {
        if let Some(task) = task {
            let status = task.status.clone();
            let context_id = task.context_id.clone();
            let task_id = task.id.clone();
            let artifacts = task.artifacts.clone();
            self.task_store.upsert(task).await;
            if let Some(status) = status {
                if let Some(event) = self
                    .task_store
                    .record_status_update(task_id.clone(), context_id.clone(), status)
                    .await
                {
                    self.emitter.emit(event).await;
                }
            }
            if let Some(task_id) = task_id {
                for artifact in artifacts {
                    if let Some(event) = self
                        .task_store
                        .record_artifact_update(
                            Some(task_id.clone()),
                            context_id.clone(),
                            artifact,
                            Some(false),
                            Some(true),
                        )
                        .await
                    {
                        self.emitter.emit(event).await;
                    }
                }
            }
        }
        if let Some(message) = message {
            self.task_store.insert_message(&message).await;
        }
        if let Some(update) = status_update {
            if let Some(status) = update.status {
                if let Some(event) = self
                    .task_store
                    .record_status_update(update.task_id.clone(), update.context_id.clone(), status)
                    .await
                {
                    self.emitter.emit(event).await;
                }
            }
        }
        if let Some(update) = artifact_update {
            if let Some(event) = self
                .task_store
                .record_artifact_update(
                    update.task_id.clone(),
                    update.context_id.clone(),
                    update.artifact.unwrap_or_default(),
                    update.append,
                    update.last_chunk,
                )
                .await
            {
                self.emitter.emit(event).await;
            }
        }
        Ok(())
    }
}
