use chrono::{DateTime, Utc};
use serde_json::json;
use stellar_monitor_tenant_isolation::models::monitor::TenantMonitor;
use uuid::Uuid;

/// Builder for creating test TenantMonitor instances
pub struct MonitorBuilder {
	id: Uuid,
	tenant_id: Uuid,
	monitor_id: String,
	name: String,
	network_id: Uuid,
	configuration: serde_json::Value,
	is_active: Option<bool>,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
}

impl Default for MonitorBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			monitor_id: "monitor-1".to_string(),
			name: "Test Monitor".to_string(),
			network_id: Uuid::new_v4(),
			configuration: json!({
				"type": "contract_event",
				"contract_address": "0x1234567890abcdef",
				"event_name": "Transfer",
				"filters": []
			}),
			is_active: Some(true),
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		}
	}
}

impl MonitorBuilder {
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

	pub fn with_monitor_id(mut self, monitor_id: impl Into<String>) -> Self {
		self.monitor_id = monitor_id.into();
		self
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_network_id(mut self, network_id: Uuid) -> Self {
		self.network_id = network_id;
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

	pub fn build(self) -> TenantMonitor {
		TenantMonitor {
			id: self.id,
			tenant_id: self.tenant_id,
			monitor_id: self.monitor_id,
			name: self.name,
			network_id: self.network_id,
			configuration: self.configuration,
			is_active: self.is_active,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}
