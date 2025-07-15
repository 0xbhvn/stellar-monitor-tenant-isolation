use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub user_id: Option<Uuid>,
	pub api_key_id: Option<Uuid>,
	pub action: String,
	pub resource_type: Option<String>,
	pub resource_id: Option<Uuid>,
	pub changes: Option<JsonValue>,
	pub ip_address: Option<IpAddr>,
	pub user_agent: Option<String>,
	pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditLogRequest {
	pub tenant_id: Uuid,
	pub user_id: Option<Uuid>,
	pub api_key_id: Option<Uuid>,
	pub action: AuditAction,
	pub resource_type: Option<ResourceType>,
	pub resource_id: Option<Uuid>,
	pub changes: Option<JsonValue>,
	pub ip_address: Option<IpAddr>,
	pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
	// Authentication
	Login,
	Logout,
	ApiKeyCreated,
	ApiKeyDeleted,
	// Tenant management
	TenantCreated,
	TenantUpdated,
	TenantDeleted,
	// User management
	UserInvited,
	UserRemoved,
	UserRoleChanged,
	// Monitor operations
	MonitorCreated,
	MonitorUpdated,
	MonitorDeleted,
	MonitorEnabled,
	MonitorDisabled,
	// Network operations
	NetworkCreated,
	NetworkUpdated,
	NetworkDeleted,
	// Trigger operations
	TriggerCreated,
	TriggerUpdated,
	TriggerDeleted,
	TriggerEnabled,
	TriggerDisabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
	Tenant,
	User,
	ApiKey,
	Monitor,
	Network,
	Trigger,
}

impl AuditAction {
	pub fn as_str(&self) -> &'static str {
		match self {
			AuditAction::Login => "login",
			AuditAction::Logout => "logout",
			AuditAction::ApiKeyCreated => "api_key_created",
			AuditAction::ApiKeyDeleted => "api_key_deleted",
			AuditAction::TenantCreated => "tenant_created",
			AuditAction::TenantUpdated => "tenant_updated",
			AuditAction::TenantDeleted => "tenant_deleted",
			AuditAction::UserInvited => "user_invited",
			AuditAction::UserRemoved => "user_removed",
			AuditAction::UserRoleChanged => "user_role_changed",
			AuditAction::MonitorCreated => "monitor_created",
			AuditAction::MonitorUpdated => "monitor_updated",
			AuditAction::MonitorDeleted => "monitor_deleted",
			AuditAction::MonitorEnabled => "monitor_enabled",
			AuditAction::MonitorDisabled => "monitor_disabled",
			AuditAction::NetworkCreated => "network_created",
			AuditAction::NetworkUpdated => "network_updated",
			AuditAction::NetworkDeleted => "network_deleted",
			AuditAction::TriggerCreated => "trigger_created",
			AuditAction::TriggerUpdated => "trigger_updated",
			AuditAction::TriggerDeleted => "trigger_deleted",
			AuditAction::TriggerEnabled => "trigger_enabled",
			AuditAction::TriggerDisabled => "trigger_disabled",
		}
	}
}

impl ResourceType {
	pub fn as_str(&self) -> &'static str {
		match self {
			ResourceType::Tenant => "tenant",
			ResourceType::User => "user",
			ResourceType::ApiKey => "api_key",
			ResourceType::Monitor => "monitor",
			ResourceType::Network => "network",
			ResourceType::Trigger => "trigger",
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::str::FromStr;

	#[test]
	fn test_audit_action_as_str() {
		assert_eq!(AuditAction::Login.as_str(), "login");
		assert_eq!(AuditAction::Logout.as_str(), "logout");
		assert_eq!(AuditAction::ApiKeyCreated.as_str(), "api_key_created");
		assert_eq!(AuditAction::ApiKeyDeleted.as_str(), "api_key_deleted");
		assert_eq!(AuditAction::TenantCreated.as_str(), "tenant_created");
		assert_eq!(AuditAction::TenantUpdated.as_str(), "tenant_updated");
		assert_eq!(AuditAction::TenantDeleted.as_str(), "tenant_deleted");
		assert_eq!(AuditAction::UserInvited.as_str(), "user_invited");
		assert_eq!(AuditAction::UserRemoved.as_str(), "user_removed");
		assert_eq!(AuditAction::UserRoleChanged.as_str(), "user_role_changed");
		assert_eq!(AuditAction::MonitorCreated.as_str(), "monitor_created");
		assert_eq!(AuditAction::MonitorUpdated.as_str(), "monitor_updated");
		assert_eq!(AuditAction::MonitorDeleted.as_str(), "monitor_deleted");
		assert_eq!(AuditAction::MonitorEnabled.as_str(), "monitor_enabled");
		assert_eq!(AuditAction::MonitorDisabled.as_str(), "monitor_disabled");
		assert_eq!(AuditAction::NetworkCreated.as_str(), "network_created");
		assert_eq!(AuditAction::NetworkUpdated.as_str(), "network_updated");
		assert_eq!(AuditAction::NetworkDeleted.as_str(), "network_deleted");
		assert_eq!(AuditAction::TriggerCreated.as_str(), "trigger_created");
		assert_eq!(AuditAction::TriggerUpdated.as_str(), "trigger_updated");
		assert_eq!(AuditAction::TriggerDeleted.as_str(), "trigger_deleted");
		assert_eq!(AuditAction::TriggerEnabled.as_str(), "trigger_enabled");
		assert_eq!(AuditAction::TriggerDisabled.as_str(), "trigger_disabled");
	}

	#[test]
	fn test_resource_type_as_str() {
		assert_eq!(ResourceType::Tenant.as_str(), "tenant");
		assert_eq!(ResourceType::User.as_str(), "user");
		assert_eq!(ResourceType::ApiKey.as_str(), "api_key");
		assert_eq!(ResourceType::Monitor.as_str(), "monitor");
		assert_eq!(ResourceType::Network.as_str(), "network");
		assert_eq!(ResourceType::Trigger.as_str(), "trigger");
	}

	#[test]
	fn test_audit_log_creation() {
		let log = AuditLog {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			user_id: Some(Uuid::new_v4()),
			api_key_id: None,
			action: "monitor_created".to_string(),
			resource_type: Some("monitor".to_string()),
			resource_id: Some(Uuid::new_v4()),
			changes: Some(serde_json::json!({
				"name": "Test Monitor",
				"network": "stellar-testnet"
			})),
			ip_address: Some(IpAddr::from_str("192.168.1.1").unwrap()),
			user_agent: Some("Mozilla/5.0".to_string()),
			created_at: Some(Utc::now()),
		};

		assert_eq!(log.action, "monitor_created");
		assert!(log.user_id.is_some());
		assert!(log.api_key_id.is_none());
		assert!(log.changes.is_some());
		assert!(log.ip_address.is_some());
	}

	#[test]
	fn test_create_audit_log_request() {
		let request = CreateAuditLogRequest {
			tenant_id: Uuid::new_v4(),
			user_id: Some(Uuid::new_v4()),
			api_key_id: None,
			action: AuditAction::MonitorCreated,
			resource_type: Some(ResourceType::Monitor),
			resource_id: Some(Uuid::new_v4()),
			changes: Some(serde_json::json!({"status": "created"})),
			ip_address: Some(IpAddr::from_str("10.0.0.1").unwrap()),
			user_agent: Some("Test Agent".to_string()),
		};

		assert!(matches!(request.action, AuditAction::MonitorCreated));
		assert!(matches!(request.resource_type, Some(ResourceType::Monitor)));
		assert!(request.changes.is_some());
	}

	#[test]
	fn test_audit_action_serialization() {
		let action = AuditAction::MonitorCreated;
		let json = serde_json::to_value(&action).unwrap();
		assert_eq!(json, serde_json::json!("MonitorCreated"));

		let deserialized: AuditAction = serde_json::from_value(json).unwrap();
		assert!(matches!(deserialized, AuditAction::MonitorCreated));
	}

	#[test]
	fn test_resource_type_serialization() {
		let resource = ResourceType::Monitor;
		let json = serde_json::to_value(&resource).unwrap();
		assert_eq!(json, serde_json::json!("Monitor"));

		let deserialized: ResourceType = serde_json::from_value(json).unwrap();
		assert!(matches!(deserialized, ResourceType::Monitor));
	}
}
