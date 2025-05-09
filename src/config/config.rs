use std::{path::PathBuf, str::FromStr};

use error_stack::{Result, ResultExt, bail};
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use toml_edit::DocumentMut;

use crate::errors::Error;

pub struct LoadConfigResult {
    pub config: Config,
    pub warnings: Vec<String>,
}

pub fn load_config(config_file: PathBuf) -> Result<LoadConfigResult, Error> {
    let content = std::fs::read_to_string(&config_file).change_context_lazy(|| {
        Error::Message(format!(
            "failed to read config file: {}",
            config_file.display()
        ))
    })?;

    let mut config = DocumentMut::from_str(&content)
        .change_context_lazy(|| Error::Message("failed to parse config file".to_string()))?;

    let env = std::env::vars()
        .filter(|(key, _)| key.starts_with("POEM_ADMIN_CONFIG_"))
        .collect::<std::collections::HashMap<String, String>>();

    fn set_toml_path(
        doc: &mut DocumentMut,
        key: &str,
        path: &'static str,
        value: toml_edit::Item,
    ) -> Vec<String> {
        let mut current = doc.as_item_mut();
        let mut warnings = vec![];
        let parts = path.split('.').collect::<Vec<_>>();
        let len = parts.len();
        assert!(len > 0, "path must not be empty");
        for part in parts.iter().take(len - 1) {
            if current.get(part).is_none() {
                warnings.push(format!(
                    "[key={key}] config path '{path}' has missing parent '{part}'; created",
                ));
            }
            current = &mut current[part];
        }
        current[parts[len - 1]] = value;
        warnings
    }
    let known_option_entries = known_option_entries();
    let mut warnings = vec![];
    for (k, v) in env {
        let Some(ent) = known_option_entries.iter().find(|e| e.env_name == k) else {
            bail!(Error::Message(format!(
                "failed to parse unknown environment variable {k} with value {v}"
            )))
        };
        let (path, item) = match ent.ent_type {
            "string" => {
                let path = ent.ent_path;
                let value = toml_edit::value(v);
                (path, value)
            }
            "integer" => {
                let path = ent.ent_path;
                let value = v.parse::<i64>().change_context_lazy(|| {
                    Error::Message(format!("failed to parse integer value {v} of key {k}"))
                })?;
                let value = toml_edit::value(value);
                (path, value)
            }
            "boolean" => {
                let path = ent.ent_path;
                let value = v.parse::<bool>().change_context_lazy(|| {
                    Error::Message(format!("failed to parse boolean value {v} of key {k}"))
                })?;
                let value = toml_edit::value(value);
                (path, value)
            }
            ty => {
                bail!(Error::Message(format!(
                    "failed to parse environment variable {k} with value {v} and resolved type {ty}"
                )))
            }
        };
        let new_warnings = set_toml_path(&mut config, &k, path, item);
        warnings.extend(new_warnings);
    }

    let config = Config::deserialize(config.into_deserializer())
        .change_context_lazy(|| Error::Message("failed to deserialize config".to_string()))?;
    Ok(LoadConfigResult { config, warnings })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advertise_addr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Database {
    #[serde(default = "default_database_host")]
    pub host: String,
    #[serde(default = "default_database_port")]
    pub port: u16,
    #[serde(default = "default_database_username")]
    pub username: String,
    #[serde(default = "default_database_password")]
    pub password: String,
    #[serde(default = "default_database_name")]
    pub database_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    #[serde(default = "LogsConfig::disabled")]
    pub logs: LogsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileAppenderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<StderrAppenderConfig>,
}

impl LogsConfig {
    pub fn disabled() -> Self {
        Self {
            file: None,
            stderr: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileAppenderConfig {
    pub filter: String,
    pub dir: String,
    pub max_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrAppenderConfig {
    pub filter: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: Server {
                listen_addr: default_listen_addr(),
                advertise_addr: None,
            },
            database: Database {
                host: default_database_host(),
                port: default_database_port(),
                username: default_database_username(),
                password: default_database_password(),
                database_name: default_database_name(),
            },
            telemetry: TelemetryConfig {
                logs: LogsConfig {
                    file: Some(FileAppenderConfig {
                        filter: "INFO".to_string(),
                        dir: "logs".to_string(),
                        max_files: 64,
                    }),
                    stderr: Some(StderrAppenderConfig {
                        filter: "INFO".to_string(),
                    }),
                },
            },
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct OptionEntry {
    /// The name of the environment variable.
    pub env_name: &'static str,
    /// The path in the config file.
    pub ent_path: &'static str,
    /// The type of the value.
    pub ent_type: &'static str,
}

pub const fn known_option_entries() -> &'static [OptionEntry] {
    &[
        OptionEntry {
            env_name: "POEM_ADMIN_CONFIG_SERVER_LISTEN_ADDR",
            ent_path: "server.listen_addr",
            ent_type: "string",
        },
        OptionEntry {
            env_name: "POEM_ADMIN_CONFIG_DATABASE_HOST",
            ent_path: "database.host",
            ent_type: "string",
        },
        OptionEntry {
            env_name: "POEM_ADMIN_CONFIG_DATABASE_PORT",
            ent_path: "database.port",
            ent_type: "integer",
        },
        OptionEntry {
            env_name: "POEM_ADMIN_CONFIG_DATABASE_NAME",
            ent_path: "database.database_name",
            ent_type: "string",
        },
        OptionEntry {
            env_name: "POEM_ADMIN_CONFIG_DATABASE_USERNAME",
            ent_path: "database.username",
            ent_type: "string",
        },
        OptionEntry {
            env_name: "POEM_ADMIN_CONFIG_DATABASE_PASSWORD",
            ent_path: "database.password",
            ent_type: "string",
        },
    ]
}

fn default_listen_addr() -> String {
    "0.0.0.0:9000".to_string()
}

fn default_database_host() -> String {
    "localhost".to_string()
}

fn default_database_port() -> u16 {
    5432
}

fn default_database_username() -> String {
    "postgres".to_string()
}

fn default_database_password() -> String {
    "postgres".to_string()
}

fn default_database_name() -> String {
    "poem-admin".to_string()
}
