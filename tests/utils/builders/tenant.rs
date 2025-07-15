use chrono::{DateTime, Utc};
use stellar_monitor_tenant_isolation::models::tenant::Tenant;
use uuid::Uuid;

/// Builder for creating test Tenant instances
pub struct TenantBuilder {
	id: Uuid,
	name: String,
	slug: String,
	is_active: bool,
	max_monitors: i32,
	max_networks: i32,
	max_triggers_per_monitor: i32,
	max_rpc_requests_per_minute: i32,
	max_storage_mb: i32,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
}

impl Default for TenantBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			name: "Test Tenant".to_string(),
			slug: "test-tenant".to_string(),
			is_active: true,
			max_monitors: 10,
			max_networks: 5,
			max_triggers_per_monitor: 5,
			max_rpc_requests_per_minute: 100,
			max_storage_mb: 1000,
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		}
	}
}

impl TenantBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_id(mut self, id: Uuid) -> Self {
		self.id = id;
		self
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_slug(mut self, slug: impl Into<String>) -> Self {
		self.slug = slug.into();
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = is_active;
		self
	}

	pub fn with_max_monitors(mut self, max_monitors: i32) -> Self {
		self.max_monitors = max_monitors;
		self
	}

	pub fn with_max_networks(mut self, max_networks: i32) -> Self {
		self.max_networks = max_networks;
		self
	}

	pub fn with_max_triggers_per_monitor(mut self, max_triggers: i32) -> Self {
		self.max_triggers_per_monitor = max_triggers;
		self
	}

	pub fn with_max_rpc_requests_per_minute(mut self, max_requests: i32) -> Self {
		self.max_rpc_requests_per_minute = max_requests;
		self
	}

	pub fn with_max_storage_mb(mut self, max_storage: i32) -> Self {
		self.max_storage_mb = max_storage;
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

	pub fn build(self) -> Tenant {
		Tenant {
			id: self.id,
			name: self.name,
			slug: self.slug,
			is_active: self.is_active,
			max_monitors: self.max_monitors,
			max_networks: self.max_networks,
			max_triggers_per_monitor: self.max_triggers_per_monitor,
			max_rpc_requests_per_minute: self.max_rpc_requests_per_minute,
			max_storage_mb: self.max_storage_mb,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}
