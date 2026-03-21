use log::warn;
use std::collections::HashMap;

pub fn load_config() -> HashMap<String, serde_json::Value> {
    let Some(config_dir) = dirs::config_dir() else {
        return HashMap::new();
    };
    let path = config_dir.join("zap").join("config.toml");

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return HashMap::new(),
        Err(e) => {
            warn!("Failed to read config at {}: {e}", path.display());
            return HashMap::new();
        }
    };

    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to parse config at {}: {e}", path.display());
            return HashMap::new();
        }
    };

    table
        .into_iter()
        .map(|(key, value)| (key, toml_to_json(value)))
        .collect()
}

fn toml_to_json(value: toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::json!(i),
        toml::Value::Float(f) => serde_json::json!(f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let map = table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}
