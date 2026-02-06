use crate::{
    document::ProvDocument,
    types::{Activity, Agent, Entity, Used, WasAssociatedWith, WasGeneratedBy},
};
use std::collections::HashMap;

pub struct EntityBuilder {
    id: String,
    prov_type: Option<String>,
    attributes: HashMap<String, serde_json::Value>,
}

impl EntityBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            prov_type: None,
            attributes: HashMap::new(),
        }
    }

    pub fn type_(mut self, prov_type: &str) -> Self {
        self.prov_type = Some(prov_type.to_string());
        self
    }

    pub fn attr(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    pub fn build(self) -> (String, Entity) {
        let entity = Entity {
            prov_type: self.prov_type,
            attributes: self.attributes,
        };
        (self.id, entity)
    }
}

pub struct ActivityBuilder {
    id: String,
    start_time_ms: Option<u64>,
    end_time_ms: Option<u64>,
    prov_type: Option<String>,
    attributes: HashMap<String, serde_json::Value>,
}

impl ActivityBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            start_time_ms: None,
            end_time_ms: None,
            prov_type: None,
            attributes: HashMap::new(),
        }
    }

    pub fn start_time_ms(mut self, time_ms: u64) -> Self {
        self.start_time_ms = Some(time_ms);
        self
    }

    pub fn end_time_ms(mut self, time_ms: u64) -> Self {
        self.end_time_ms = Some(time_ms);
        self
    }

    pub fn type_(mut self, prov_type: &str) -> Self {
        self.prov_type = Some(prov_type.to_string());
        self
    }

    pub fn attr(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    pub fn build(self) -> (String, Activity) {
        let activity = Activity {
            start_time_ms: self.start_time_ms,
            end_time_ms: self.end_time_ms,
            prov_type: self.prov_type,
            attributes: self.attributes,
        };
        (self.id, activity)
    }
}

pub struct AgentBuilder {
    id: String,
    prov_type: Option<String>,
    attributes: HashMap<String, serde_json::Value>,
}

impl AgentBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            prov_type: None,
            attributes: HashMap::new(),
        }
    }

    pub fn type_(mut self, prov_type: &str) -> Self {
        self.prov_type = Some(prov_type.to_string());
        self
    }

    pub fn attr(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    pub fn build(self) -> (String, Agent) {
        let agent = Agent {
            prov_type: self.prov_type,
            attributes: self.attributes,
        };
        (self.id, agent)
    }
}

pub struct ProvDocumentBuilder {
    doc: ProvDocument,
}

impl ProvDocumentBuilder {
    pub fn new() -> Self {
        Self {
            doc: ProvDocument::new(),
        }
    }

    pub fn entity<F>(mut self, id: &str, f: F) -> Self
    where
        F: FnOnce(EntityBuilder) -> (String, Entity),
    {
        let (id, entity) = f(EntityBuilder::new(id));
        self.doc.entity.insert(id, entity);
        self
    }

    pub fn activity<F>(mut self, id: &str, f: F) -> Self
    where
        F: FnOnce(ActivityBuilder) -> (String, Activity),
    {
        let (id, activity) = f(ActivityBuilder::new(id));
        self.doc.activity.insert(id, activity);
        self
    }

    pub fn agent<F>(mut self, id: &str, f: F) -> Self
    where
        F: FnOnce(AgentBuilder) -> (String, Agent),
    {
        let (id, agent) = f(AgentBuilder::new(id));
        self.doc.agent.insert(id, agent);
        self
    }

    pub fn used(self, activity: &str, entity: &str) -> UsedBuilder {
        UsedBuilder::new(self, activity.to_string(), entity.to_string())
    }

    pub fn was_generated_by(self, entity: &str, activity: &str) -> WasGeneratedByBuilder {
        WasGeneratedByBuilder::new(self, entity.to_string(), activity.to_string())
    }

    pub fn was_associated_with(self, activity: &str, agent: &str) -> WasAssociatedWithBuilder {
        WasAssociatedWithBuilder::new(self, activity.to_string(), agent.to_string())
    }

    pub fn build(self) -> ProvDocument {
        self.doc
    }
}

pub struct UsedBuilder {
    doc_builder: ProvDocumentBuilder,
    activity: String,
    entity: String,
    role: Option<String>,
}

impl UsedBuilder {
    fn new(doc_builder: ProvDocumentBuilder, activity: String, entity: String) -> Self {
        Self {
            doc_builder,
            activity,
            entity,
            role: None,
        }
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn build(mut self) -> ProvDocumentBuilder {
        let id = self.doc_builder.doc.blank_node_id("u");
        let used = Used {
            activity: self.activity,
            entity: self.entity,
            role: self.role,
        };
        self.doc_builder.doc.used.insert(id, used);
        self.doc_builder
    }
}

pub struct WasGeneratedByBuilder {
    doc_builder: ProvDocumentBuilder,
    entity: String,
    activity: String,
    time_ms: Option<u64>,
}

impl WasGeneratedByBuilder {
    fn new(doc_builder: ProvDocumentBuilder, entity: String, activity: String) -> Self {
        Self {
            doc_builder,
            entity,
            activity,
            time_ms: None,
        }
    }

    pub fn time_ms(mut self, time_ms: u64) -> Self {
        self.time_ms = Some(time_ms);
        self
    }

    pub fn build(mut self) -> ProvDocumentBuilder {
        let id = self.doc_builder.doc.blank_node_id("g");
        let was_generated_by = WasGeneratedBy {
            entity: self.entity,
            activity: self.activity,
            time_ms: self.time_ms,
        };
        self.doc_builder
            .doc
            .was_generated_by
            .insert(id, was_generated_by);
        self.doc_builder
    }
}

pub struct WasAssociatedWithBuilder {
    doc_builder: ProvDocumentBuilder,
    activity: String,
    agent: String,
    role: Option<String>,
}

impl WasAssociatedWithBuilder {
    fn new(doc_builder: ProvDocumentBuilder, activity: String, agent: String) -> Self {
        Self {
            doc_builder,
            activity,
            agent,
            role: None,
        }
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn build(mut self) -> ProvDocumentBuilder {
        let id = self.doc_builder.doc.blank_node_id("assoc");
        let was_associated_with = WasAssociatedWith {
            activity: self.activity,
            agent: self.agent,
            role: self.role,
        };
        self.doc_builder
            .doc
            .was_associated_with
            .insert(id, was_associated_with);
        self.doc_builder
    }
}

impl Default for ProvDocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
