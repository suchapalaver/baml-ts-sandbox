//! Provenance capture and storage.
//!
//! This crate provides event types and interceptors for provenance recording,
//! along with a pluggable storage interface and an in-memory implementation.

pub mod builders;
pub mod document;
pub mod error;
pub mod events;
pub mod interceptors;
pub mod store;
pub mod types;

pub use error::ProvenanceError;
pub use events::{ProvEvent, ProvEventData, ProvEventType};
pub use interceptors::ProvenanceInterceptor;
pub use store::{InMemoryProvenanceStore, ProvenanceWriter};
