use chrono::{DateTime, Utc};
use stellar_monitor_tenant_isolation::models::user::User;
use uuid::Uuid;

/// Builder for creating test User instances
pub struct UserBuilder {
	id: Uuid,
	email: String,
	password_hash: String,
	is_active: bool,
	created_at: Option<DateTime<Utc>>,
	updated_at: Option<DateTime<Utc>>,
}

impl Default for UserBuilder {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			email: "test@example.com".to_string(),
			// Default hash for password "password123"
			password_hash: "$argon2id$v=19$m=65536,t=3,p=4$abcd1234$hash".to_string(),
			is_active: true,
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		}
	}
}

impl UserBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_id(mut self, id: Uuid) -> Self {
		self.id = id;
		self
	}

	pub fn with_email(mut self, email: impl Into<String>) -> Self {
		self.email = email.into();
		self
	}

	pub fn with_password_hash(mut self, password_hash: impl Into<String>) -> Self {
		self.password_hash = password_hash.into();
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = is_active;
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

	pub fn build(self) -> User {
		User {
			id: self.id,
			email: self.email,
			password_hash: self.password_hash,
			is_active: self.is_active,
			created_at: self.created_at,
			updated_at: self.updated_at,
		}
	}
}
