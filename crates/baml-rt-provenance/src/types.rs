use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    #[serde(rename = "prov:type", skip_serializing_if = "Option::is_none")]
    pub prov_type: Option<String>,
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Activity {
    #[serde(rename = "prov:startTime", skip_serializing_if = "Option::is_none")]
    pub start_time_ms: Option<u64>,
    #[serde(rename = "prov:endTime", skip_serializing_if = "Option::is_none")]
    pub end_time_ms: Option<u64>,
    #[serde(rename = "prov:type", skip_serializing_if = "Option::is_none")]
    pub prov_type: Option<String>,
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    #[serde(rename = "prov:type", skip_serializing_if = "Option::is_none")]
    pub prov_type: Option<String>,
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Used {
    #[serde(rename = "prov:activity")]
    pub activity: String,
    #[serde(rename = "prov:entity")]
    pub entity: String,
    #[serde(rename = "prov:role", skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasGeneratedBy {
    #[serde(rename = "prov:entity")]
    pub entity: String,
    #[serde(rename = "prov:activity")]
    pub activity: String,
    #[serde(rename = "prov:time", skip_serializing_if = "Option::is_none")]
    pub time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasAssociatedWith {
    #[serde(rename = "prov:activity")]
    pub activity: String,
    #[serde(rename = "prov:agent")]
    pub agent: String,
    #[serde(rename = "prov:role", skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}
