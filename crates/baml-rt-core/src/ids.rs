//! Strongly-typed ID wrappers for domain concepts.
//!
//! These newtypes prevent mixing different ID types at compile time,
//! following the production-rust.md guidelines for strong types at boundaries.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id_type {
    ($(#[$doc:meta])* $name:ident) => {
        define_id_type!($(#[$doc])* $name, []);
    };
    ($(#[$doc:meta])* $name:ident, $derive:path) => {
        define_id_type!($(#[$doc])* $name, [$derive]);
    };
    ($(#[$doc:meta])* $name:ident, [$($derive:path),*]) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize $(, $derive)*)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(id: String) -> Self {
                Self(id)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(id: String) -> Self {
                Self(id)
            }
        }

        impl From<&str> for $name {
            fn from(id: &str) -> Self {
                Self(id.to_string())
            }
        }
    };
}

define_id_type!(
    /// Message identifier for A2A messages
    MessageId
);

define_id_type!(
    /// Task identifier for A2A tasks
    TaskId,
    [Default]
);

define_id_type!(
    /// Context identifier for agent execution contexts
    ContextId
);

define_id_type!(
    /// Correlation identifier for distributed tracing
    CorrelationId
);

define_id_type!(
    /// Artifact identifier for task artifacts
    ArtifactId
);

define_id_type!(
    /// Event identifier for provenance events
    EventId
);
