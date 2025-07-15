use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use std::net::IpAddr;
use stellar_monitor_tenant_isolation::models::{AuditAction, AuditLog};
use uuid::Uuid;

/// Builder for creating test AuditLog instances
pub struct AuditLogBuilder {
	id: Uuid,
	tenant_id: Uuid,
	user_id: Option<Uuid>,
	api_key_id: Option<Uuid>,
	action: String,
	resource_type: Option<String>,
	resource_id: Option<Uuid>,
	changes: Option<JsonValue>,
	ip_address: Option<IpAddr>,
	user_agent: Option<String>,
	created_at: Option<DateTime<Utc>>,
}

impl Default for AuditLogBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			user_id: Some(Uuid::new_v4()),
			api_key_id: None,
			action: "MonitorCreated".to_string(),
			resource_type: Some("monitor".to_string()),
			resource_id: Some(Uuid::new_v4()),
			changes: None,
			ip_address: None,
			user_agent: None,
			created_at: Some(Utc::now()),
		}
	}
}

impl AuditLogBuilder {
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

	pub fn with_user_id(mut self, user_id: Uuid) -> Self {
		self.user_id = Some(user_id);
		self
	}

	pub fn with_api_key_id(mut self, api_key_id: Uuid) -> Self {
		self.api_key_id = Some(api_key_id);
		self
	}

	pub fn with_action(mut self, action: AuditAction) -> Self {
		self.action = action.as_str().to_string();
		self
	}

	pub fn with_resource_type(mut self, resource_type: impl Into<String>) -> Self {
		self.resource_type = Some(resource_type.into());
		self
	}

	pub fn with_resource_id(mut self, resource_id: &str) -> Self {
		self.resource_id = Uuid::parse_str(resource_id).ok();
		self
	}

	pub fn with_resource_uuid(mut self, resource_id: Uuid) -> Self {
		self.resource_id = Some(resource_id);
		self
	}

	pub fn with_changes(mut self, changes: Option<JsonValue>) -> Self {
		self.changes = changes;
		self
	}

	pub fn with_ip_address(mut self, ip_address: Option<IpAddr>) -> Self {
		self.ip_address = ip_address;
		self
	}

	pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
		self.user_agent = user_agent;
		self
	}

	pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
		self.created_at = Some(created_at);
		self
	}

	pub fn build(self) -> AuditLog {
		AuditLog {
			id: self.id,
			tenant_id: self.tenant_id,
			user_id: self.user_id,
			api_key_id: self.api_key_id,
			action: self.action,
			resource_type: self.resource_type,
			resource_id: self.resource_id,
			changes: self.changes,
			ip_address: self.ip_address,
			user_agent: self.user_agent,
			created_at: self.created_at,
		}
	}
}
