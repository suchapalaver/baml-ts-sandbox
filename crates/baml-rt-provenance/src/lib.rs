//! Provenance capture and storage.
//!
//! This crate provides event types and interceptors for provenance recording,
//! along with a pluggable storage interface and an in-memory implementation.

pub mod error;
pub mod events;
pub mod types;
pub mod document;
pub mod builders;
pub mod store;
pub mod interceptors;

pub use error::ProvenanceError;
pub use events::{ProvEvent, ProvEventData, ProvEventType};
pub use store::{InMemoryProvenanceStore, ProvenanceWriter};
pub use interceptors::ProvenanceInterceptor;
