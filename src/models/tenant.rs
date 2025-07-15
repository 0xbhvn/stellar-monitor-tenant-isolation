use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::resource_quota::{ApiRateLimits, TenantQuotas};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
	pub id: Uuid,
	pub name: String,
	pub slug: String,
	pub is_active: bool,
	// Resource quotas
	pub max_monitors: i32,
	pub max_networks: i32,
	pub max_triggers_per_monitor: i32,
	pub max_rpc_requests_per_minute: i32,
	pub max_storage_mb: i32,
	// Metadata
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

impl Tenant {
	pub fn resource_quotas(&self) -> TenantQuotas {
		TenantQuotas {
			max_monitors: self.max_monitors,
			max_networks: self.max_networks,
			max_triggers_per_monitor: self.max_triggers_per_monitor,
			max_rpc_requests_per_minute: self.max_rpc_requests_per_minute,
			max_storage_mb: self.max_storage_mb,
			api_rate_limits: ApiRateLimits::default(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantRequest {
	pub name: String,
	pub slug: String,
	pub max_monitors: Option<i32>,
	pub max_networks: Option<i32>,
	pub max_triggers_per_monitor: Option<i32>,
	pub max_rpc_requests_per_minute: Option<i32>,
	pub max_storage_mb: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantRequest {
	pub name: Option<String>,
	pub is_active: Option<bool>,
	pub max_monitors: Option<i32>,
	pub max_networks: Option<i32>,
	pub max_triggers_per_monitor: Option<i32>,
	pub max_rpc_requests_per_minute: Option<i32>,
	pub max_storage_mb: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantMembership {
	pub id: Uuid,
	pub tenant_id: Uuid,
	pub user_id: Uuid,
	pub role: TenantRole,
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum TenantRole {
	#[sqlx(rename = "owner")]
	#[serde(rename = "owner")]
	Owner,
	#[sqlx(rename = "admin")]
	#[serde(rename = "admin")]
	Admin,
	#[sqlx(rename = "member")]
	#[serde(rename = "member")]
	Member,
	#[sqlx(rename = "viewer")]
	#[serde(rename = "viewer")]
	Viewer,
}

impl TenantRole {
	pub fn can_manage_tenant(&self) -> bool {
		matches!(self, TenantRole::Owner | TenantRole::Admin)
	}

	pub fn can_write(&self) -> bool {
		matches!(
			self,
			TenantRole::Owner | TenantRole::Admin | TenantRole::Member
		)
	}

	pub fn can_read(&self) -> bool {
		true // All roles can read
	}
}
