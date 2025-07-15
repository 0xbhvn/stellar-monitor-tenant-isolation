use async_trait::async_trait;
use mockall::mock;
use stellar_monitor_tenant_isolation::{
	models::*,
	services::{
		monitor_service::{AuditServiceTrait, MonitorServiceTrait, ServiceError},
		network_service::NetworkServiceTrait,
		trigger_service::TriggerServiceTrait,
	},
};
use uuid::Uuid;

// Mock for AuditService
mock! {
	pub AuditService {}

	#[async_trait]
	impl AuditServiceTrait for AuditService {
		async fn log(&self, request: CreateAuditLogRequest) -> Result<(), ServiceError>;
	}
}

// Mock for MonitorService
mock! {
	pub MonitorService {}

	#[async_trait]
	impl MonitorServiceTrait for MonitorService {
		async fn create_monitor(
			&self,
			request: CreateMonitorRequest,
			metadata: RequestMetadata,
		) -> Result<TenantMonitor, ServiceError>;

		async fn get_monitor(&self, monitor_id: &str) -> Result<TenantMonitor, ServiceError>;

		async fn update_monitor(
			&self,
			monitor_id: &str,
			request: UpdateMonitorRequest,
			metadata: RequestMetadata,
		) -> Result<TenantMonitor, ServiceError>;

		async fn delete_monitor(
			&self,
			monitor_id: &str,
			metadata: RequestMetadata,
		) -> Result<(), ServiceError>;

		async fn list_monitors(
			&self,
			limit: i64,
			offset: i64,
		) -> Result<Vec<TenantMonitor>, ServiceError>;

		async fn get_monitor_count(&self) -> Result<i64, ServiceError>;
	}
}

// Mock for NetworkService
mock! {
	pub NetworkService {}

	#[async_trait]
	impl NetworkServiceTrait for NetworkService {
		async fn create_network(
			&self,
			request: CreateNetworkRequest,
			metadata: RequestMetadata,
		) -> Result<TenantNetwork, ServiceError>;

		async fn get_network(&self, network_id: &str) -> Result<TenantNetwork, ServiceError>;

		async fn update_network(
			&self,
			network_id: &str,
			request: UpdateNetworkRequest,
			metadata: RequestMetadata,
		) -> Result<TenantNetwork, ServiceError>;

		async fn delete_network(
			&self,
			network_id: &str,
			metadata: RequestMetadata,
		) -> Result<(), ServiceError>;

		async fn list_networks(
			&self,
			limit: i64,
			offset: i64,
		) -> Result<Vec<TenantNetwork>, ServiceError>;

		async fn get_network_count(&self) -> Result<i64, ServiceError>;
	}
}

// Mock for TriggerService
mock! {
	pub TriggerService {}

	#[async_trait]
	impl TriggerServiceTrait for TriggerService {
		async fn create_trigger(
			&self,
			request: CreateTriggerRequest,
			metadata: RequestMetadata,
		) -> Result<TenantTrigger, ServiceError>;

		async fn get_trigger(&self, trigger_id: &str) -> Result<TenantTrigger, ServiceError>;

		async fn update_trigger(
			&self,
			trigger_id: &str,
			request: UpdateTriggerRequest,
			metadata: RequestMetadata,
		) -> Result<TenantTrigger, ServiceError>;

		async fn delete_trigger(
			&self,
			trigger_id: &str,
			metadata: RequestMetadata,
		) -> Result<(), ServiceError>;

		async fn list_triggers(
			&self,
			limit: i64,
			offset: i64,
		) -> Result<Vec<TenantTrigger>, ServiceError>;

		async fn list_triggers_by_monitor(
			&self,
			monitor_id: Uuid,
		) -> Result<Vec<TenantTrigger>, ServiceError>;

		async fn get_trigger_count(&self) -> Result<i64, ServiceError>;
	}
}
