use async_trait::async_trait;
use mockall::mock;
use std::collections::HashMap;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::{
		error::TenantRepositoryError, monitor::TenantMonitorRepositoryTrait,
		network::TenantNetworkRepositoryTrait, tenant::TenantRepositoryTrait,
		trigger::TenantTriggerRepositoryTrait,
	},
};
use uuid::Uuid;

// Mock for TenantRepository
mock! {
	pub TenantRepository {}

	impl Clone for TenantRepository {
		fn clone(&self) -> Self;
	}

	#[async_trait]
	impl TenantRepositoryTrait for TenantRepository {
		async fn create(&self, request: CreateTenantRequest) -> Result<Tenant, TenantRepositoryError>;
		async fn get(&self, tenant_id: Uuid) -> Result<Tenant, TenantRepositoryError>;
		async fn get_by_slug(&self, slug: &str) -> Result<Tenant, TenantRepositoryError>;
		async fn update(&self, tenant_id: Uuid, request: UpdateTenantRequest) -> Result<Tenant, TenantRepositoryError>;
		async fn delete(&self, tenant_id: Uuid) -> Result<(), TenantRepositoryError>;
		async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>, TenantRepositoryError>;

		// Membership management
		async fn add_member(&self, tenant_id: Uuid, user_id: Uuid, role: TenantRole) -> Result<TenantMembership, TenantRepositoryError>;
		async fn remove_member(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), TenantRepositoryError>;
		async fn update_member_role(&self, tenant_id: Uuid, user_id: Uuid, role: TenantRole) -> Result<TenantMembership, TenantRepositoryError>;
		async fn get_members(&self, tenant_id: Uuid) -> Result<Vec<TenantMembership>, TenantRepositoryError>;
		async fn get_user_tenants(&self, user_id: Uuid) -> Result<Vec<(Tenant, TenantRole)>, TenantRepositoryError>;

		// Resource quota management
		async fn get_quota_status(&self, tenant_id: Uuid) -> Result<ResourceQuotaStatus, TenantRepositoryError>;
		async fn check_quota(&self, tenant_id: Uuid, resource: &str, amount: i32) -> Result<bool, TenantRepositoryError>;
	}
}

// Mock for TenantMonitorRepository
mock! {
	pub TenantMonitorRepository {}

	impl Clone for TenantMonitorRepository {
		fn clone(&self) -> Self;
	}

	#[async_trait]
	impl TenantMonitorRepositoryTrait for TenantMonitorRepository {
		async fn create(&self, request: CreateMonitorRequest) -> Result<TenantMonitor, TenantRepositoryError>;
		async fn get(&self, monitor_id: &str) -> Result<TenantMonitor, TenantRepositoryError>;
		async fn get_by_uuid(&self, id: Uuid) -> Result<TenantMonitor, TenantRepositoryError>;
		async fn get_all(&self) -> Result<HashMap<String, TenantMonitor>, TenantRepositoryError>;
		async fn update(&self, monitor_id: &str, request: UpdateMonitorRequest) -> Result<TenantMonitor, TenantRepositoryError>;
		async fn delete(&self, monitor_id: &str) -> Result<(), TenantRepositoryError>;
		async fn list(&self, limit: i64, offset: i64) -> Result<Vec<TenantMonitor>, TenantRepositoryError>;
		async fn check_quota(&self) -> Result<bool, TenantRepositoryError>;
	}
}

// Mock for TenantNetworkRepository
mock! {
	pub TenantNetworkRepository {}

	impl Clone for TenantNetworkRepository {
		fn clone(&self) -> Self;
	}

	#[async_trait]
	impl TenantNetworkRepositoryTrait for TenantNetworkRepository {
		async fn create(&self, request: CreateNetworkRequest) -> Result<TenantNetwork, TenantRepositoryError>;
		async fn get(&self, network_id: &str) -> Result<TenantNetwork, TenantRepositoryError>;
		async fn get_by_uuid(&self, id: Uuid) -> Result<TenantNetwork, TenantRepositoryError>;
		async fn get_all(&self) -> Result<HashMap<String, TenantNetwork>, TenantRepositoryError>;
		async fn update(&self, network_id: &str, request: UpdateNetworkRequest) -> Result<TenantNetwork, TenantRepositoryError>;
		async fn delete(&self, network_id: &str) -> Result<(), TenantRepositoryError>;
		async fn list(&self, limit: i64, offset: i64) -> Result<Vec<TenantNetwork>, TenantRepositoryError>;
		async fn check_quota(&self) -> Result<bool, TenantRepositoryError>;
	}
}

// Mock for TenantTriggerRepository
mock! {
	pub TenantTriggerRepository {}

	impl Clone for TenantTriggerRepository {
		fn clone(&self) -> Self;
	}

	#[async_trait]
	impl TenantTriggerRepositoryTrait for TenantTriggerRepository {
		async fn create(&self, request: CreateTriggerRequest) -> Result<TenantTrigger, TenantRepositoryError>;
		async fn get(&self, trigger_id: &str) -> Result<TenantTrigger, TenantRepositoryError>;
		async fn get_by_uuid(&self, id: Uuid) -> Result<TenantTrigger, TenantRepositoryError>;
		async fn get_all(&self) -> Result<HashMap<String, TenantTrigger>, TenantRepositoryError>;
		async fn get_by_monitor(&self, monitor_id: Uuid) -> Result<Vec<TenantTrigger>, TenantRepositoryError>;
		async fn update(&self, trigger_id: &str, request: UpdateTriggerRequest) -> Result<TenantTrigger, TenantRepositoryError>;
		async fn delete(&self, trigger_id: &str) -> Result<(), TenantRepositoryError>;
		async fn list(&self, limit: i64, offset: i64) -> Result<Vec<TenantTrigger>, TenantRepositoryError>;
		async fn count(&self) -> Result<i64, TenantRepositoryError>;
		async fn check_quota(&self, monitor_id: Uuid) -> Result<bool, TenantRepositoryError>;
	}
}
