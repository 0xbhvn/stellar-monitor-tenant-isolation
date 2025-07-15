use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantMonitor {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub monitor_id: String, // Original OZ Monitor ID
	pub name: String,
	pub network_id: Uuid,
	pub configuration: JsonValue, // Full monitor config from OZ Monitor
	pub is_active: Option<bool>,
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMonitorRequest {
	pub monitor_id: String,
	pub name: String,
	pub network_id: Uuid,
	pub configuration: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMonitorRequest {
	pub name: Option<String>,
	pub configuration: Option<JsonValue>,
	pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantNetwork {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub network_id: String, // Original OZ Monitor network ID
	pub name: String,
	pub blockchain: String,       // 'stellar', 'evm'
	pub configuration: JsonValue, // Full network config from OZ Monitor
	pub is_active: Option<bool>,
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkRequest {
	pub network_id: String,
	pub name: String,
	pub blockchain: String,
	pub configuration: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNetworkRequest {
	pub name: Option<String>,
	pub configuration: Option<JsonValue>,
	pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantTrigger {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub trigger_id: String, // Original OZ Monitor trigger ID
	pub monitor_id: Uuid,
	pub name: String,
	#[sqlx(rename = "type")]
	pub trigger_type: String, // 'webhook', 'email', 'slack', etc.
	pub configuration: JsonValue, // Full trigger config from OZ Monitor
	pub is_active: Option<bool>,
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTriggerRequest {
	pub trigger_id: String,
	pub monitor_id: Uuid,
	pub name: String,
	pub trigger_type: String,
	pub configuration: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTriggerRequest {
	pub name: Option<String>,
	pub configuration: Option<JsonValue>,
	pub is_active: Option<bool>,
}
