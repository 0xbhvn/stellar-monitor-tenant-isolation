use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::models::audit::ResourceType as AuditResourceType;
use crate::models::{
	AuditAction, CreateAuditLogRequest, CreateMonitorRequest, RequestMetadata, TenantMonitor,
	UpdateMonitorRequest,
};
use crate::repositories::{
	TenantMonitorRepositoryTrait, TenantRepositoryError, TenantRepositoryTrait,
};
use crate::utils::current_tenant_context;

// Note: The TenantMonitor type in this service stores monitor configurations with multi-tenant isolation.
// The actual monitor execution will be handled by a separate openzeppelin-monitor instance that reads
// configurations from the shared database, while this service handles tenant-specific concerns like
// quotas, permissions, and audit logging.

#[async_trait]
pub trait MonitorServiceTrait: Send + Sync {
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

#[derive(Clone)]
pub struct MonitorService<M, T, A>
where
	M: TenantMonitorRepositoryTrait,
	T: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	monitor_repo: M,
	tenant_repo: T,
	audit_service: A,
}

impl<M, T, A> MonitorService<M, T, A>
where
	M: TenantMonitorRepositoryTrait,
	T: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	pub fn new(monitor_repo: M, tenant_repo: T, audit_service: A) -> Self {
		Self {
			monitor_repo,
			tenant_repo,
			audit_service,
		}
	}
}

#[async_trait]
impl<M, T, A> MonitorServiceTrait for MonitorService<M, T, A>
where
	M: TenantMonitorRepositoryTrait + Send + Sync,
	T: TenantRepositoryTrait + Send + Sync,
	A: AuditServiceTrait + Send + Sync,
{
	async fn create_monitor(
		&self,
		request: CreateMonitorRequest,
		metadata: RequestMetadata,
	) -> Result<TenantMonitor, ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to create monitors".to_string(),
			));
		}

		// Check quota
		let quota_status = self.tenant_repo.get_quota_status(context.tenant_id).await?;
		if !quota_status.can_create_monitor() {
			return Err(ServiceError::QuotaExceeded(format!(
				"Monitor quota exceeded: {}/{}",
				quota_status.usage.monitors_count, quota_status.quotas.max_monitors
			)));
		}

		// Create monitor
		let monitor = self.monitor_repo.create(request.clone()).await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::MonitorCreated,
				resource_type: Some(AuditResourceType::Monitor),
				resource_id: Some(monitor.id),
				changes: Some(serde_json::to_value(&request).unwrap_or(JsonValue::Null)),
				ip_address: metadata.ip_address.clone(),
				user_agent: metadata.user_agent.clone(),
			})
			.await?;

		Ok(monitor)
	}

	async fn get_monitor(&self, monitor_id: &str) -> Result<TenantMonitor, ServiceError> {
		// Read permission is checked by repository through tenant context
		Ok(self.monitor_repo.get(monitor_id).await?)
	}

	async fn update_monitor(
		&self,
		monitor_id: &str,
		request: UpdateMonitorRequest,
		metadata: RequestMetadata,
	) -> Result<TenantMonitor, ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to update monitors".to_string(),
			));
		}

		// Get existing monitor first to ensure it exists
		let existing = self.monitor_repo.get(monitor_id).await?;

		// Update monitor
		let monitor = self
			.monitor_repo
			.update(monitor_id, request.clone())
			.await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::MonitorUpdated,
				resource_type: Some(AuditResourceType::Monitor),
				resource_id: Some(existing.id),
				changes: Some(serde_json::to_value(&request).unwrap_or(JsonValue::Null)),
				ip_address: metadata.ip_address.clone(),
				user_agent: metadata.user_agent.clone(),
			})
			.await?;

		Ok(monitor)
	}

	async fn delete_monitor(
		&self,
		monitor_id: &str,
		metadata: RequestMetadata,
	) -> Result<(), ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to delete monitors".to_string(),
			));
		}

		// Get monitor first to ensure it exists and for audit log
		let monitor = self.monitor_repo.get(monitor_id).await?;

		// Delete monitor
		self.monitor_repo.delete(monitor_id).await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::MonitorDeleted,
				resource_type: Some(AuditResourceType::Monitor),
				resource_id: Some(monitor.id),
				changes: None,
				ip_address: metadata.ip_address.clone(),
				user_agent: metadata.user_agent.clone(),
			})
			.await?;

		Ok(())
	}

	async fn list_monitors(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantMonitor>, ServiceError> {
		Ok(self.monitor_repo.list(limit, offset).await?)
	}

	async fn get_monitor_count(&self) -> Result<i64, ServiceError> {
		let monitors = self.monitor_repo.get_all().await?;
		Ok(monitors.len() as i64)
	}
}

// Service error type
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
	#[error("Repository error: {0}")]
	Repository(#[from] TenantRepositoryError),

	#[error("Access denied: {0}")]
	AccessDenied(String),

	#[error("Quota exceeded: {0}")]
	QuotaExceeded(String),

	#[error("Validation error: {0}")]
	ValidationError(String),

	#[error("Internal error: {0}")]
	Internal(String),
}

// Placeholder for audit service trait
#[async_trait]
pub trait AuditServiceTrait: Send + Sync {
	async fn log(&self, request: CreateAuditLogRequest) -> Result<(), ServiceError>;
}
