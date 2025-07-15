use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResourceUsage {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub resource_type: ResourceType,
	pub usage_value: i64,
	pub usage_date: NaiveDate,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ResourceType {
	#[sqlx(rename = "rpc_requests")]
	#[serde(rename = "rpc_requests")]
	RpcRequests,
	#[sqlx(rename = "storage")]
	#[serde(rename = "storage")]
	Storage,
	#[sqlx(rename = "compute_minutes")]
	#[serde(rename = "compute_minutes")]
	ComputeMinutes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotaStatus {
	pub tenant_id: Uuid,
	pub quotas: TenantQuotas,
	pub usage: CurrentUsage,
	pub available: AvailableResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
	pub max_monitors: i32,
	pub max_networks: i32,
	pub max_triggers_per_monitor: i32,
	pub max_rpc_requests_per_minute: i32,
	pub max_storage_mb: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUsage {
	pub monitors_count: i32,
	pub networks_count: i32,
	pub triggers_count: i32,
	pub rpc_requests_last_minute: i32,
	pub storage_mb_used: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableResources {
	pub monitors: i32,
	pub networks: i32,
	pub triggers: i32,
	pub rpc_requests_per_minute: i32,
	pub storage_mb: i32,
}

impl ResourceQuotaStatus {
	pub fn can_create_monitor(&self) -> bool {
		self.available.monitors > 0
	}

	pub fn can_create_network(&self) -> bool {
		self.available.networks > 0
	}

	pub fn can_create_trigger(&self) -> bool {
		self.available.triggers > 0
	}

	pub fn has_rpc_capacity(&self, requests: i32) -> bool {
		self.available.rpc_requests_per_minute >= requests
	}

	pub fn has_storage_capacity(&self, mb: i32) -> bool {
		self.available.storage_mb >= mb
	}
}
