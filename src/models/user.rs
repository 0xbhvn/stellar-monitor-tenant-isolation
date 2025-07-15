use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
	pub id: Uuid,
	pub email: String,
	#[serde(skip_serializing)]
	pub password_hash: String,
	pub is_active: bool,
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
	pub email: String,
	pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
	pub email: Option<String>,
	pub password: Option<String>,
	pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
	pub email: String,
	pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
	pub access_token: String,
	pub refresh_token: String,
	pub expires_in: i64,
	pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
	pub id: Uuid,
	pub email: String,
	pub tenants: Vec<UserTenant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTenant {
	pub tenant_id: Uuid,
	pub tenant_name: String,
	pub tenant_slug: String,
	pub role: crate::models::tenant::TenantRole,
}
