use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
	pub server: ServerConfig,
	pub database: DatabaseConfig,
	pub auth: AuthConfig,
	pub monitoring: MonitoringConfig,
	pub quotas: DefaultQuotaConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
	pub host: String,
	pub port: u16,
	pub workers: Option<usize>,
}

impl ServerConfig {
	pub fn socket_addr(&self) -> SocketAddr {
		format!("{}:{}", self.host, self.port)
			.parse()
			.expect("Invalid server address")
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
	pub url: String,
	pub max_connections: u32,
	pub min_connections: u32,
	pub connect_timeout_seconds: u64,
	pub idle_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
	pub jwt_secret: String,
	pub jwt_expiration_hours: i64,
	pub refresh_token_expiration_days: i64,
	pub api_key_prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
	pub metrics_enabled: bool,
	pub metrics_port: u16,
	pub tracing_level: String,
	pub log_format: LogFormat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
	Json,
	Pretty,
	Compact,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultQuotaConfig {
	pub max_monitors: i32,
	pub max_networks: i32,
	pub max_triggers_per_monitor: i32,
	pub max_rpc_requests_per_minute: i32,
	pub max_storage_mb: i32,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			server: ServerConfig {
				host: "0.0.0.0".to_string(),
				port: 3000,
				workers: None,
			},
			database: DatabaseConfig {
				url: "postgres://localhost/stellar_monitor_tenant".to_string(),
				max_connections: 10,
				min_connections: 1,
				connect_timeout_seconds: 10,
				idle_timeout_seconds: 600,
			},
			auth: AuthConfig {
				jwt_secret: "change-me-in-production".to_string(),
				jwt_expiration_hours: 24,
				refresh_token_expiration_days: 30,
				api_key_prefix: "smt_".to_string(),
			},
			monitoring: MonitoringConfig {
				metrics_enabled: true,
				metrics_port: 9090,
				tracing_level: "info".to_string(),
				log_format: LogFormat::Json,
			},
			quotas: DefaultQuotaConfig {
				max_monitors: 10,
				max_networks: 5,
				max_triggers_per_monitor: 10,
				max_rpc_requests_per_minute: 1000,
				max_storage_mb: 1000,
			},
		}
	}
}

impl Config {
	/// Load configuration from environment variables with fallback to defaults
	pub fn from_env() -> Result<Self, config::ConfigError> {
		let mut config = config::Config::builder()
			.add_source(config::Environment::with_prefix("SMT").separator("__"))
			.build()?;

		// Try to load from a config file if specified
		if let Ok(config_path) = std::env::var("SMT_CONFIG_PATH") {
			config = config::Config::builder()
				.add_source(config::File::with_name(&config_path))
				.add_source(config::Environment::with_prefix("SMT").separator("__"))
				.build()?;
		}

		config.try_deserialize()
	}

	/// Load configuration from a specific file path
	pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
		config::Config::builder()
			.add_source(config::File::with_name(path))
			.add_source(config::Environment::with_prefix("SMT").separator("__"))
			.build()?
			.try_deserialize()
	}

	/// Validate configuration values
	pub fn validate(&self) -> Result<(), String> {
		if self.server.port == 0 {
			return Err("Server port cannot be 0".to_string());
		}

		if self.database.max_connections < self.database.min_connections {
			return Err("Database max_connections must be >= min_connections".to_string());
		}

		if self.auth.jwt_secret == "change-me-in-production" {
			tracing::warn!("Using default JWT secret - change this in production!");
		}

		if self.auth.jwt_expiration_hours <= 0 {
			return Err("JWT expiration hours must be positive".to_string());
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config() {
		let config = Config::default();
		assert_eq!(config.server.port, 3000);
		assert_eq!(config.database.max_connections, 10);
	}

	#[test]
	fn test_config_validation() {
		let mut config = Config::default();
		assert!(config.validate().is_ok());

		config.server.port = 0;
		assert!(config.validate().is_err());

		config.server.port = 3000;
		config.database.max_connections = 0;
		config.database.min_connections = 1;
		assert!(config.validate().is_err());
	}
}
