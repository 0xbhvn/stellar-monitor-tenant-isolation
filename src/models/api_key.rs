use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub name: String,
	#[serde(skip_serializing)]
	pub key_hash: String,
	pub permissions: JsonValue,
	pub last_used_at: Option<DateTime<Utc>>,
	pub expires_at: Option<DateTime<Utc>>,
	pub is_active: bool,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
	pub name: String,
	pub permissions: Vec<ApiPermission>,
	pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
	pub id: Uuid,
	pub name: String,
	pub key: String, // Only returned once during creation
	pub permissions: Vec<ApiPermission>,
	pub expires_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPermission {
	pub resource: String,
	pub actions: Vec<String>,
}

impl ApiPermission {
	pub fn all_monitors() -> Self {
		Self {
			resource: "monitors".to_string(),
			actions: vec![
				"read".to_string(),
				"write".to_string(),
				"delete".to_string(),
			],
		}
	}

	pub fn read_only_monitors() -> Self {
		Self {
			resource: "monitors".to_string(),
			actions: vec!["read".to_string()],
		}
	}

	pub fn all_networks() -> Self {
		Self {
			resource: "networks".to_string(),
			actions: vec![
				"read".to_string(),
				"write".to_string(),
				"delete".to_string(),
			],
		}
	}

	pub fn all_triggers() -> Self {
		Self {
			resource: "triggers".to_string(),
			actions: vec![
				"read".to_string(),
				"write".to_string(),
				"delete".to_string(),
			],
		}
	}
}
