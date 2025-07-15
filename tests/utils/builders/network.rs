use chrono::{DateTime, Utc};
use serde_json::json;
use stellar_monitor_tenant_isolation::models::monitor::TenantNetwork;
use uuid::Uuid;

/// Builder for creating test TenantNetwork instances
pub struct NetworkBuilder {
	id: Uuid,
	tenant_id: Uuid,
	network_id: String,
	name: String,
	blockchain: String,
	configuration: serde_json::Value,
	is_active: Option<bool>,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
}

impl Default for NetworkBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			network_id: "stellar-testnet".to_string(),
			name: "Stellar Testnet".to_string(),
			blockchain: "stellar".to_string(),
			configuration: json!({
				"rpc_url": "https://horizon-testnet.stellar.org",
				"chain_id": "testnet",
				"block_time": 5
			}),
			is_active: Some(true),
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		}
	}
}

impl NetworkBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_id(mut self, id: Uuid) -> Self {
		self.id = id;
		self
	}

	pub fn with_tenant_id(mut self, tenant_id: Uuid) -> Self {
		self.tenant_id = tenant_id;
		self
	}

	pub fn with_network_id(mut self, network_id: impl Into<String>) -> Self {
		self.network_id = network_id.into();
		self
	}

	// Alias for backward compatibility
	pub fn with_original_id(self, network_id: impl Into<String>) -> Self {
		self.with_network_id(network_id)
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_blockchain(mut self, blockchain: impl Into<String>) -> Self {
		self.blockchain = blockchain.into();
		self
	}

	pub fn with_configuration(mut self, configuration: serde_json::Value) -> Self {
		self.configuration = configuration;
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = Some(is_active);
		self
	}

	pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
		self.created_at = Some(created_at);
		self
	}

	pub fn with_updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
		self.updated_at = Some(updated_at);
		self
	}

	pub fn build(self) -> TenantNetwork {
		TenantNetwork {
			id: self.id,
			tenant_id: self.tenant_id,
			network_id: self.network_id,
			name: self.name,
			blockchain: self.blockchain,
			configuration: self.configuration,
			is_active: self.is_active,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}
