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

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::{Duration, Utc};

	#[test]
	fn test_api_permission_all_monitors() {
		let perm = ApiPermission::all_monitors();
		assert_eq!(perm.resource, "monitors");
		assert_eq!(perm.actions.len(), 3);
		assert!(perm.actions.contains(&"read".to_string()));
		assert!(perm.actions.contains(&"write".to_string()));
		assert!(perm.actions.contains(&"delete".to_string()));
	}

	#[test]
	fn test_api_permission_read_only_monitors() {
		let perm = ApiPermission::read_only_monitors();
		assert_eq!(perm.resource, "monitors");
		assert_eq!(perm.actions.len(), 1);
		assert!(perm.actions.contains(&"read".to_string()));
		assert!(!perm.actions.contains(&"write".to_string()));
		assert!(!perm.actions.contains(&"delete".to_string()));
	}

	#[test]
	fn test_api_permission_all_networks() {
		let perm = ApiPermission::all_networks();
		assert_eq!(perm.resource, "networks");
		assert_eq!(perm.actions.len(), 3);
		assert!(perm.actions.contains(&"read".to_string()));
		assert!(perm.actions.contains(&"write".to_string()));
		assert!(perm.actions.contains(&"delete".to_string()));
	}

	#[test]
	fn test_api_permission_all_triggers() {
		let perm = ApiPermission::all_triggers();
		assert_eq!(perm.resource, "triggers");
		assert_eq!(perm.actions.len(), 3);
		assert!(perm.actions.contains(&"read".to_string()));
		assert!(perm.actions.contains(&"write".to_string()));
		assert!(perm.actions.contains(&"delete".to_string()));
	}

	#[test]
	fn test_api_key_creation() {
		let api_key = ApiKey {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			name: "Test API Key".to_string(),
			key_hash: "$argon2id$v=19$m=65536,t=3,p=4$test$hash".to_string(),
			permissions: serde_json::json!([
				{
					"resource": "monitors",
					"actions": ["read", "write"]
				}
			]),
			last_used_at: None,
			expires_at: Some(Utc::now() + Duration::days(30)),
			is_active: true,
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		assert_eq!(api_key.name, "Test API Key");
		assert!(api_key.is_active);
		assert!(api_key.expires_at.is_some());
		assert!(api_key.last_used_at.is_none());
	}

	#[test]
	fn test_create_api_key_request() {
		let request = CreateApiKeyRequest {
			name: "Production Key".to_string(),
			permissions: vec![
				ApiPermission::all_monitors(),
				ApiPermission::read_only_monitors(),
			],
			expires_at: Some(Utc::now() + Duration::days(90)),
		};

		assert_eq!(request.name, "Production Key");
		assert_eq!(request.permissions.len(), 2);
		assert!(request.expires_at.is_some());
	}

	#[test]
	fn test_create_api_key_response() {
		let response = CreateApiKeyResponse {
			id: Uuid::new_v4(),
			name: "New Key".to_string(),
			key: "sk_test_1234567890".to_string(),
			permissions: vec![ApiPermission::all_monitors()],
			expires_at: None,
			created_at: Utc::now(),
		};

		assert_eq!(response.name, "New Key");
		assert!(response.key.starts_with("sk_"));
		assert_eq!(response.permissions.len(), 1);
		assert!(response.expires_at.is_none());
	}

	#[test]
	fn test_api_permission_serialization() {
		let perm = ApiPermission {
			resource: "custom".to_string(),
			actions: vec!["action1".to_string(), "action2".to_string()],
		};

		let json = serde_json::to_value(&perm).unwrap();
		assert_eq!(json["resource"], "custom");
		assert!(json["actions"].is_array());
		assert_eq!(json["actions"].as_array().unwrap().len(), 2);

		let deserialized: ApiPermission = serde_json::from_value(json).unwrap();
		assert_eq!(deserialized.resource, "custom");
		assert_eq!(deserialized.actions, vec!["action1", "action2"]);
	}
}
