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
