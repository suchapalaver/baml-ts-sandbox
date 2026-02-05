//! Context ID propagation for async invocation flows.
//!
//! This module provides task-local context IDs so async boundaries
//! can retain request context without requiring JS changes.

use crate::ids::ContextId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

tokio::task_local! {
    static CONTEXT_ID: ContextId;
}

static CONTEXT_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn generate_context_id() -> ContextId {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let counter = CONTEXT_COUNTER.fetch_add(1, Ordering::Relaxed);
    ContextId::new(format!("ctx-{}-{}", millis, counter))
}

pub fn current_context_id() -> Option<ContextId> {
    CONTEXT_ID.try_with(|id| id.clone()).ok()
}

pub fn current_or_new() -> ContextId {
    current_context_id().unwrap_or_else(generate_context_id)
}

pub async fn with_context_id<F, T>(id: ContextId, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    CONTEXT_ID.scope(id, fut).await
}
