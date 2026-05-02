use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BoundaryManifest {
    pub stack: Option<Stack>,
    pub queues: Option<QueueBoundary>,
    pub db: Option<DbBoundary>,
    #[serde(default)]
    pub streaming_exception: Vec<StreamingException>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Stack {
    pub id: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct QueueBoundary {
    #[serde(default)]
    pub adapter_paths: Vec<String>,
    #[serde(default)]
    pub event_contract_paths: Vec<String>,
    #[serde(default)]
    pub generated_type_paths: Vec<String>,
    #[serde(default)]
    pub client_markers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DbBoundary {
    #[serde(default)]
    pub root_paths: Vec<String>,
    #[serde(default)]
    pub migration_paths: Vec<String>,
    #[serde(default)]
    pub constraint_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamingException {
    pub runtime: String,
    pub classification: Option<String>,
    pub reason: Option<String>,
    pub owner: String,
    pub expires: String,
    pub migration_path: String,
}

pub fn parse(text: &str) -> Result<BoundaryManifest> {
    Ok(toml::from_str(text)?)
}

pub fn load(path: &Path) -> Result<BoundaryManifest> {
    parse(&fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_queue_streaming_and_db_boundaries() {
        let text = r#"
[stack]
id = "rust-ts-vite-react-postgres-bounded-python"
version = "0.4.0"

[queues]
adapter_paths = ["crates/adapters/queues", "crates/adapters/src/queues"]
event_contract_paths = ["contracts/events"]
generated_type_paths = ["contracts/generated"]
client_markers = ["rdkafka", "kafkajs"]

[db]
root_paths = ["db"]
migration_paths = ["db/migrations"]
constraint_paths = ["db/constraints"]

[[streaming_exception]]
runtime = "kafka"
classification = "brownfield"
owner = "platform"
expires = "2026-12-31"
migration_path = "Keep Kafka behind queue adapters."
"#;
        let manifest = parse(text).unwrap();
        assert_eq!(
            manifest.stack.as_ref().map(|stack| stack.id.as_str()),
            Some("rust-ts-vite-react-postgres-bounded-python")
        );
        assert_eq!(
            manifest
                .queues
                .as_ref()
                .map(|queues| queues.adapter_paths.len()),
            Some(2)
        );
        assert_eq!(
            manifest.db.as_ref().map(|db| db.root_paths.as_slice()),
            Some(&["db".to_string()][..])
        );
        assert_eq!(
            manifest.streaming_exception[0].classification.as_deref(),
            Some("brownfield")
        );
    }
}
