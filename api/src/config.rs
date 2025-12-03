//! Configuration module for the API server
//!
//! Supports loading configuration from a TOML file.

use serde::Deserialize;
use std::path::Path;

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Windows service settings
    #[serde(default)]
    pub service: ServiceConfig,
}

/// Windows service configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    /// Service name (default: WinInfraApi)
    #[serde(default = "default_service_name")]
    pub name: String,

    /// Service display name (default: Windows Infrastructure API)
    #[serde(default = "default_display_name")]
    pub display_name: String,

    /// Service description
    #[serde(default = "default_description")]
    pub description: String,

    /// Installation directory (default: C:\Program Files\WinInfraApi)
    #[serde(default = "default_install_path")]
    pub install_path: String,
}

/// Server-specific configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Host address to bind to (default: 0.0.0.0)
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on (default: 3000)
    #[serde(default = "default_port")]
    pub port: u16,
}

/// Logging configuration
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    /// Log level filter (default: "api=info,tower_http=info")
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    6001
}

fn default_log_level() -> String {
    "api=info,tower_http=info".to_string()
}

fn default_service_name() -> String {
    "nodeagent".to_string()
}

fn default_display_name() -> String {
    "Node Agent".to_string()
}

fn default_description() -> String {
    "REST API for Windows Failover Cluster and Hyper-V management".to_string()
}

fn default_install_path() -> String {
    r"C:\Program Files\azurestack\nodeagent".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: default_service_name(),
            display_name: default_display_name(),
            description: default_description(),
            install_path: default_install_path(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            service: ServiceConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    ///
    /// If the file doesn't exist, returns default configuration.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            tracing::info!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.display().to_string(), e.to_string()))?;

        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(path.display().to_string(), e.to_string()))
    }

    /// Get the socket address string
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    ReadError(String, String),
    ParseError(String, String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadError(path, err) => {
                write!(f, "Failed to read config file '{}': {}", path, err)
            }
            ConfigError::ParseError(path, err) => {
                write!(f, "Failed to parse config file '{}': {}", path, err)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 6001);
        assert_eq!(config.logging.level, "api=info,tower_http=info");
        assert_eq!(config.service.name, "nodeagent");
        assert_eq!(config.service.display_name, "Node Agent");
        assert_eq!(
            config.service.install_path,
            r"C:\Program Files\azurestack\nodeagent"
        );
    }

    #[test]
    fn test_socket_addr() {
        let config = Config::default();
        assert_eq!(config.socket_addr(), "0.0.0.0:6001");
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
            [server]
            port = 8080
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0"); // default
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
            [server]
            host = "127.0.0.1"
            port = 8080

            [logging]
            level = "debug"

            [service]
            name = "MyApi"
            display_name = "My Custom API"
            description = "Custom API service"
            install_path = "D:\\Services\\MyApi"
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.service.name, "MyApi");
        assert_eq!(config.service.display_name, "My Custom API");
        assert_eq!(config.service.install_path, r"D:\Services\MyApi");
    }
}
