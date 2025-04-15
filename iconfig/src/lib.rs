
use serde::{Deserialize, Serialize};
use toml::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path};
use std::sync::Arc;
use serde::de::DeserializeOwned;
use rioc::{injectable, provider};

/// A flexible configuration container that can hold any valid TOML data
/// and supports merging configurations.
///
/// # Examples
///
/// ```
/// use iconfig::ApplicationConfig;
///
/// let mut base = ApplicationConfig::from_str(r#"
///     [server]
///     host = "localhost"
///     port = 8080
/// "#).unwrap();
///
/// let overlay = ApplicationConfig::from_str(r#"
///     [server]
///     port = 9090
///     [database]
///     url = "postgres://localhost"
/// "#).unwrap();
///
/// base.merge(overlay);
///
/// assert_eq!(base.get("server.host").unwrap().as_str(), Some("localhost"));
/// assert_eq!(base.get("server.port").unwrap().as_integer(), Some(9090));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[injectable]
pub struct ApplicationConfig {
    #[serde(flatten)]
    value: Value,
}

impl fmt::Display for ApplicationConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        load().unwrap()
    }
}

impl ApplicationConfig {
    /// Create a new TomlConfig from a TOML string
    pub fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        let value = toml::from_str(s)?;
        Ok(Self { value })
    }

    /// Create a new TomlConfig from a TOML string
    pub fn from_file<P: AsRef<Path>>(fname: P) -> Result<Self, anyhow::Error> {
        let path = fname.as_ref();
        if !path.exists() {
            return Err(anyhow::anyhow!("File {} does not exist", path.display()));
        }
        let config = std::fs::read_to_string(path)?;
        let value = Self::from_str(&config)?;


        Ok(value)
    }

    /// Merge another TomlConfig into this one
    /// 
    /// This performs a deep merge where:
    /// - Tables are merged recursively
    /// - Arrays are concatenated
    /// - Other values are overwritten by the new config
    pub fn merge(&mut self, other: Self) {
        self.value = merge_values(&self.value, &other.value);
    }

    /// Get a reference to the underlying TOML value
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Get a value by dotted path (e.g., "server.port")
    pub fn get(&self, path: &str) -> Option<&Value> {
        let mut current = &self.value;
        for part in path.split('.') {
            match current.get(part) {
                Some(v) => current = v,
                None => return None,
            }
        }
        Some(current)
    }

    /// Convert the config to a pretty-printed TOML string
    pub fn to_string_pretty(&self) -> String {
        self.value.to_string()
    }

    pub fn jsonify(&self) -> Result<String, anyhow::Error> {
        let result =  serde_json::to_string(self);
        result.map_err(|e| anyhow::anyhow!("Failed to convert to json: {}", e))
    }

    pub fn resolve<T: DeserializeOwned>(&self) -> Result<T, anyhow::Error> {
        let json = self.jsonify()?;

        let result = serde_json::from_str(&json);
        result.map_err(|e| anyhow::anyhow!("Failed to convert to json: {}", e))
    }

    pub fn resolve_prefix<T: DeserializeOwned>(&self,prefix: &str) -> Result<T, anyhow::Error> {
        if prefix == "" {
            return self.resolve::<T>()
        }

        let part = self.get(&prefix);
        if part.is_none() {
            return Err(anyhow::anyhow!("No config found for {}", prefix))
        }
        let part = part.unwrap();
        let json = serde_json::to_string(part);
        if json.is_err() {
            return Err(anyhow::anyhow!("Failed to convert to json: {}", json.err().unwrap()))
        }
        let json = json?;

        let result = serde_json::from_str(&json);
        result.map_err(|e| anyhow::anyhow!("Failed to convert to json: {}", e))
    }
}

fn merge_values(a: &Value, b: &Value) -> Value {
    match (a, b) {
        // If both are tables, merge them recursively
        (Value::Table(a_map), Value::Table(b_map)) => {
            let mut result = BTreeMap::new();
            // Add all keys from a
            for (k, v) in a_map {
                result.insert(k.clone(), v.clone());
            }
            
            // Add or merge keys from b
            for (k, v) in b_map {
                if let Some(existing) = result.get_mut(k) {
                    *existing = merge_values(existing, v);
                } else {
                    result.insert(k.clone(), v.clone());
                }
            }

            let result = toml::Table::try_from(result).unwrap();

            Value::Table(result)
        }
        // If both are arrays, concatenate them
        (Value::Array(a_vec), Value::Array(b_vec)) => {
            let mut result = a_vec.clone();
            result.extend(b_vec.clone());
            Value::Array(result)
        }
        // In all other cases, use the value from b
        _ => b.clone(),
    }
}


pub fn load() -> Result<ApplicationConfig,anyhow::Error> {
    //load from /etc/rioc/config.toml
    let mut config = ApplicationConfig::from_file("/etc/rioc/config.toml");
    if config.is_err() {
        //load from config/config.toml
        config = ApplicationConfig::from_file("config/config.toml");
        if config.is_err() {
            //load from current directory
            let config_config = ApplicationConfig::from_file("./config.toml");
            if config_config.is_ok() {
                Ok(config_config.unwrap())
            }else {
                Err(anyhow::anyhow!("No config file found"))
            }
        }else {
            //load from current directory
            let mut config = config.unwrap();
            let config_config = ApplicationConfig::from_file("./config.toml");
            if config_config.is_ok() {
                config.merge(config_config.unwrap());
                Ok(config)
            }else {
                Ok(config)
            }
        }
    }else {
        //load from config/config.toml
        let mut config = config.unwrap();
        let config_config = ApplicationConfig::from_file("config/config.toml");
        if config_config.is_err() {
            //load from current directory
            let config_config = ApplicationConfig::from_file("./config.toml");
            if config_config.is_err() {
                Ok(config)
            }else {
                config.merge(config_config.unwrap());
                Ok(config)
            }
        } else {
            config.merge(config_config.unwrap());
            //load from current directory
            let config_config = ApplicationConfig::from_file("./config.toml");
            if config_config.is_err() {
                Ok(config)
            } else {
                config.merge(config_config.unwrap());
                Ok(config)
            }
        }
    }
}

#[derive(Debug,Clone)]
#[provider]
#[provide(Arc<ApplicationConfig>, self.get())]
pub struct Provider {
    config: ApplicationConfig,
}

impl Provider {
    pub fn new() -> Self{
        let conf = load();
        Provider {
            config: conf.unwrap(),
        }
    }

    pub fn get(&self) -> Arc<ApplicationConfig> {
        Arc::new(self.config.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_tables() {
        let mut config1 = ApplicationConfig::from_str(r#"
            [server]
            host = "localhost"
            port = 8080
        "#).unwrap();

        let config2 = ApplicationConfig::from_str(r#"
            [server]
            port = 9090
            [database]
            url = "postgres://localhost"
        "#).unwrap();

        config1.merge(config2);

        let merged = config1.value();
        assert_eq!(merged["server"]["host"].as_str(), Some("localhost"));
        assert_eq!(merged["server"]["port"].as_integer(), Some(9090));
        assert_eq!(merged["database"]["url"].as_str(), Some("postgres://localhost"));
    }

    #[test]
    fn test_merge_arrays() {
        let mut config1 = ApplicationConfig::from_str(r#"
            items = [1, 2, 3]
        "#).unwrap();

        let config2 = ApplicationConfig::from_str(r#"
            items = [4, 5]
        "#).unwrap();

        config1.merge(config2);

        let merged = config1.value();
        let items = merged["items"].as_array().unwrap();
        assert_eq!(items.len(), 5);
        assert_eq!(items[0].as_integer(), Some(1));
        assert_eq!(items[4].as_integer(), Some(5));
    }

    #[test]
    fn test_get_by_path() {
        let config = ApplicationConfig::from_str(r#"
            [server]
            host = "localhost"
            port = 8080
            [database]
            url = "postgres://localhost"
        "#).unwrap();

        assert_eq!(config.get("server.host").unwrap().as_str(), Some("localhost"));
        assert_eq!(config.get("server.port").unwrap().as_integer(), Some(8080));
        assert_eq!(config.get("database.url").unwrap().as_str(), Some("postgres://localhost"));
        assert!(config.get("nonexistent.key").is_none());
    }

    #[test]
    fn test_serialization() {
        let config = ApplicationConfig::from_str(r#"
            key = "value"
        "#).unwrap();

        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_deserialization() {
        //current directory
        let current_dir = std::env::current_dir().unwrap();
        println!("{}", current_dir.display());
        let config = load();
        if config.is_err() {
            return;
        }
        let config = config.unwrap();

        #[derive(Debug, Deserialize)]
        pub struct TestConfig {
           pub  resolver: Option<String>,
        }

       let t =  config.resolve_prefix::<TestConfig>("workspace").unwrap();
       println!("{:?}", t);
    }

    #[test]
    fn test_provider() {
        let provider = Provider::new();
        let facade: Arc<ApplicationConfig> = provider.provide();
        println!("{:?}", facade);

        let facade1: Arc<ApplicationConfig> = provider.provide();
        println!("{:?}", facade1);
    }
}