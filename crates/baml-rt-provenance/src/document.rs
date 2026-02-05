use crate::types::{Activity, Agent, Entity, Used, WasAssociatedWith, WasGeneratedBy};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ProvDocument {
    pub entity: HashMap<String, Entity>,
    pub activity: HashMap<String, Activity>,
    pub agent: HashMap<String, Agent>,
    pub used: HashMap<String, Used>,
    pub was_generated_by: HashMap<String, WasGeneratedBy>,
    pub was_associated_with: HashMap<String, WasAssociatedWith>,
    blank_node_counter: u64,
}

impl ProvDocument {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn blank_node_id(&mut self, prefix: &str) -> String {
        self.blank_node_counter += 1;
        format!("{}{}", prefix, self.blank_node_counter)
    }
}
