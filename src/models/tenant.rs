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

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::Utc;

	#[test]
	fn test_tenant_role_permissions() {
		// Test Owner permissions
		assert!(TenantRole::Owner.can_manage_tenant());
		assert!(TenantRole::Owner.can_write());
		assert!(TenantRole::Owner.can_read());

		// Test Admin permissions
		assert!(TenantRole::Admin.can_manage_tenant());
		assert!(TenantRole::Admin.can_write());
		assert!(TenantRole::Admin.can_read());

		// Test Member permissions
		assert!(!TenantRole::Member.can_manage_tenant());
		assert!(TenantRole::Member.can_write());
		assert!(TenantRole::Member.can_read());

		// Test Viewer permissions
		assert!(!TenantRole::Viewer.can_manage_tenant());
		assert!(!TenantRole::Viewer.can_write());
		assert!(TenantRole::Viewer.can_read());
	}

	#[test]
	fn test_tenant_resource_quotas() {
		let tenant = Tenant {
			id: Uuid::new_v4(),
			name: "Test Corp".to_string(),
			slug: "test-corp".to_string(),
			is_active: true,
			max_monitors: 20,
			max_networks: 10,
			max_triggers_per_monitor: 5,
			max_rpc_requests_per_minute: 1000,
			max_storage_mb: 5000,
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		};

		let quotas = tenant.resource_quotas();
		assert_eq!(quotas.max_monitors, 20);
		assert_eq!(quotas.max_networks, 10);
		assert_eq!(quotas.max_triggers_per_monitor, 5);
		assert_eq!(quotas.max_rpc_requests_per_minute, 1000);
		assert_eq!(quotas.max_storage_mb, 5000);
	}

	#[test]
	fn test_tenant_role_serialization() {
		// Test serialization
		let owner = serde_json::to_string(&TenantRole::Owner).unwrap();
		let admin = serde_json::to_string(&TenantRole::Admin).unwrap();
		let member = serde_json::to_string(&TenantRole::Member).unwrap();
		let viewer = serde_json::to_string(&TenantRole::Viewer).unwrap();

		assert_eq!(owner, "\"owner\"");
		assert_eq!(admin, "\"admin\"");
		assert_eq!(member, "\"member\"");
		assert_eq!(viewer, "\"viewer\"");

		// Test deserialization
		let role: TenantRole = serde_json::from_str("\"owner\"").unwrap();
		assert!(matches!(role, TenantRole::Owner));

		let role: TenantRole = serde_json::from_str("\"viewer\"").unwrap();
		assert!(matches!(role, TenantRole::Viewer));
	}

	#[test]
	fn test_create_tenant_request() {
		let request = CreateTenantRequest {
			name: "New Tenant".to_string(),
			slug: "new-tenant".to_string(),
			max_monitors: Some(15),
			max_networks: None,
			max_triggers_per_monitor: Some(10),
			max_rpc_requests_per_minute: None,
			max_storage_mb: Some(2000),
		};

		assert_eq!(request.name, "New Tenant");
		assert_eq!(request.slug, "new-tenant");
		assert_eq!(request.max_monitors, Some(15));
		assert_eq!(request.max_networks, None);
	}

	#[test]
	fn test_update_tenant_request() {
		let request = UpdateTenantRequest {
			name: Some("Updated Name".to_string()),
			is_active: Some(false),
			max_monitors: None,
			max_networks: Some(20),
			max_triggers_per_monitor: None,
			max_rpc_requests_per_minute: None,
			max_storage_mb: None,
		};

		assert_eq!(request.name, Some("Updated Name".to_string()));
		assert_eq!(request.is_active, Some(false));
		assert_eq!(request.max_networks, Some(20));
	}

	#[test]
	fn test_tenant_membership() {
		let membership = TenantMembership {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			user_id: Uuid::new_v4(),
			role: TenantRole::Member,
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		};

		assert!(matches!(membership.role, TenantRole::Member));
		assert!(membership.created_at.is_some());
		assert!(membership.updated_at.is_some());
	}
}
