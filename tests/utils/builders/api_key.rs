use chrono::{DateTime, Utc};
use serde_json::json;
use stellar_monitor_tenant_isolation::models::api_key::ApiKey;
use uuid::Uuid;

/// Builder for creating test ApiKey instances
pub struct ApiKeyBuilder {
	id: Uuid,
	tenant_id: Uuid,
	name: String,
	key_hash: String,
	permissions: serde_json::Value,
	last_used_at: Option<DateTime<Utc>>,
	expires_at: Option<DateTime<Utc>>,
	is_active: bool,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl Default for ApiKeyBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			name: "Test API Key".to_string(),
			// Default hash for a test key
			key_hash: "$argon2id$v=19$m=65536,t=3,p=4$test$hash".to_string(),
			permissions: json!([
				{
					"resource": "monitors",
					"actions": ["read", "write"]
				}
			]),
			last_used_at: None,
			expires_at: None,
			is_active: true,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		}
	}
}

impl ApiKeyBuilder {
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

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_key_hash(mut self, key_hash: impl Into<String>) -> Self {
		self.key_hash = key_hash.into();
		self
	}

	pub fn with_permissions(mut self, permissions: serde_json::Value) -> Self {
		self.permissions = permissions;
		self
	}

	pub fn with_last_used_at(mut self, last_used_at: DateTime<Utc>) -> Self {
		self.last_used_at = Some(last_used_at);
		self
	}

	pub fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
		self.expires_at = Some(expires_at);
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = is_active;
		self
	}

	pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
		self.created_at = created_at;
		self
	}

	pub fn with_updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
		self.updated_at = updated_at;
		self
	}

	pub fn build(self) -> ApiKey {
		ApiKey {
			id: self.id,
			tenant_id: self.tenant_id,
			name: self.name,
			key_hash: self.key_hash,
			permissions: self.permissions,
			last_used_at: self.last_used_at,
			expires_at: self.expires_at,
			is_active: self.is_active,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}
