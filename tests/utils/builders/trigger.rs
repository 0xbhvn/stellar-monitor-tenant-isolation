use chrono::{DateTime, Utc};
use serde_json::json;
use stellar_monitor_tenant_isolation::models::monitor::TenantTrigger;
use uuid::Uuid;

/// Builder for creating test TenantTrigger instances
pub struct TriggerBuilder {
	id: Uuid,
	tenant_id: Uuid,
	trigger_id: String,
	monitor_id: Uuid,
	name: String,
	trigger_type: String,
	configuration: serde_json::Value,
	is_active: Option<bool>,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
}

impl Default for TriggerBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			trigger_id: "trigger-1".to_string(),
			monitor_id: Uuid::new_v4(),
			name: "Test Webhook Trigger".to_string(),
			trigger_type: "webhook".to_string(),
			configuration: json!({
				"url": "https://example.com/webhook",
				"method": "POST",
				"headers": {
					"Content-Type": "application/json"
				},
				"timeout": 30
			}),
			is_active: Some(true),
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		}
	}
}

impl TriggerBuilder {
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

	pub fn with_trigger_id(mut self, trigger_id: impl Into<String>) -> Self {
		self.trigger_id = trigger_id.into();
		self
	}

	pub fn with_monitor_id(mut self, monitor_id: Uuid) -> Self {
		self.monitor_id = monitor_id;
		self
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_trigger_type(mut self, trigger_type: impl Into<String>) -> Self {
		self.trigger_type = trigger_type.into();
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

	pub fn build(self) -> TenantTrigger {
		TenantTrigger {
			id: self.id,
			tenant_id: self.tenant_id,
			trigger_id: self.trigger_id,
			monitor_id: self.monitor_id,
			name: self.name,
			trigger_type: self.trigger_type,
			configuration: self.configuration,
			is_active: self.is_active,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}
