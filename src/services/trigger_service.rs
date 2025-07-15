use async_trait::async_trait;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use super::monitor_service::{AuditServiceTrait, ServiceError};
use crate::models::audit::ResourceType as AuditResourceType;
use crate::models::{
	AuditAction, CreateAuditLogRequest, CreateTriggerRequest, TenantTrigger, UpdateTriggerRequest,
};
use crate::repositories::{
	TenantMonitorRepositoryTrait, TenantRepositoryTrait, TenantTriggerRepositoryTrait,
};
use crate::utils::current_tenant_context;

#[async_trait]
pub trait TriggerServiceTrait: Send + Sync {
	async fn create_trigger(
		&self,
		request: CreateTriggerRequest,
	) -> Result<TenantTrigger, ServiceError>;
	async fn get_trigger(&self, trigger_id: &str) -> Result<TenantTrigger, ServiceError>;
	async fn update_trigger(
		&self,
		trigger_id: &str,
		request: UpdateTriggerRequest,
	) -> Result<TenantTrigger, ServiceError>;
	async fn delete_trigger(&self, trigger_id: &str) -> Result<(), ServiceError>;
	async fn list_triggers(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantTrigger>, ServiceError>;
	async fn list_triggers_by_monitor(
		&self,
		monitor_id: Uuid,
	) -> Result<Vec<TenantTrigger>, ServiceError>;
}

#[derive(Clone)]
pub struct TriggerService<Tr, M, T, A>
where
	Tr: TenantTriggerRepositoryTrait,
	M: TenantMonitorRepositoryTrait,
	T: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	trigger_repo: Tr,
	monitor_repo: M,
	tenant_repo: T,
	audit_service: A,
}

impl<Tr, M, T, A> TriggerService<Tr, M, T, A>
where
	Tr: TenantTriggerRepositoryTrait,
	M: TenantMonitorRepositoryTrait,
	T: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	pub fn new(trigger_repo: Tr, monitor_repo: M, tenant_repo: T, audit_service: A) -> Self {
		Self {
			trigger_repo,
			monitor_repo,
			tenant_repo,
			audit_service,
		}
	}
}

#[async_trait]
impl<Tr, M, T, A> TriggerServiceTrait for TriggerService<Tr, M, T, A>
where
	Tr: TenantTriggerRepositoryTrait + Send + Sync,
	M: TenantMonitorRepositoryTrait + Send + Sync,
	T: TenantRepositoryTrait + Send + Sync,
	A: AuditServiceTrait + Send + Sync,
{
	async fn create_trigger(
		&self,
		request: CreateTriggerRequest,
	) -> Result<TenantTrigger, ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to create triggers".to_string(),
			));
		}

		// Verify monitor exists and belongs to tenant
		let _monitor = self.monitor_repo.get_by_uuid(request.monitor_id).await?;

		// Check quota for triggers per monitor
		let quota_status = self.tenant_repo.get_quota_status(context.tenant_id).await?;
		if !self.trigger_repo.check_quota(request.monitor_id).await? {
			return Err(ServiceError::QuotaExceeded(format!(
				"Trigger quota exceeded for monitor. Max allowed: {}",
				quota_status.quotas.max_triggers_per_monitor
			)));
		}

		// Validate trigger type
		let valid_types = ["webhook", "email", "slack", "discord", "telegram", "script"];
		if !valid_types.contains(&request.trigger_type.as_str()) {
			return Err(ServiceError::ValidationError(format!(
				"Invalid trigger type: {}. Must be one of: {:?}",
				request.trigger_type, valid_types
			)));
		}

		// Create trigger
		let trigger = self.trigger_repo.create(request.clone()).await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::TriggerCreated,
				resource_type: Some(AuditResourceType::Trigger),
				resource_id: Some(trigger.id),
				changes: Some(serde_json::to_value(&request).unwrap_or(JsonValue::Null)),
				ip_address: None,
				user_agent: None,
			})
			.await?;

		Ok(trigger)
	}

	async fn get_trigger(&self, trigger_id: &str) -> Result<TenantTrigger, ServiceError> {
		Ok(self.trigger_repo.get(trigger_id).await?)
	}

	async fn update_trigger(
		&self,
		trigger_id: &str,
		request: UpdateTriggerRequest,
	) -> Result<TenantTrigger, ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to update triggers".to_string(),
			));
		}

		// Get existing trigger
		let existing = self.trigger_repo.get(trigger_id).await?;

		// Update trigger
		let trigger = self
			.trigger_repo
			.update(trigger_id, request.clone())
			.await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::TriggerUpdated,
				resource_type: Some(AuditResourceType::Trigger),
				resource_id: Some(existing.id),
				changes: Some(serde_json::to_value(&request).unwrap_or(JsonValue::Null)),
				ip_address: None,
				user_agent: None,
			})
			.await?;

		Ok(trigger)
	}

	async fn delete_trigger(&self, trigger_id: &str) -> Result<(), ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to delete triggers".to_string(),
			));
		}

		// Get trigger for audit log
		let trigger = self.trigger_repo.get(trigger_id).await?;

		// Delete trigger
		self.trigger_repo.delete(trigger_id).await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::TriggerDeleted,
				resource_type: Some(AuditResourceType::Trigger),
				resource_id: Some(trigger.id),
				changes: None,
				ip_address: None,
				user_agent: None,
			})
			.await?;

		Ok(())
	}

	async fn list_triggers(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantTrigger>, ServiceError> {
		Ok(self.trigger_repo.list(limit, offset).await?)
	}

	async fn list_triggers_by_monitor(
		&self,
		monitor_id: Uuid,
	) -> Result<Vec<TenantTrigger>, ServiceError> {
		// Verify monitor belongs to tenant
		let _ = self.monitor_repo.get_by_uuid(monitor_id).await?;

		Ok(self.trigger_repo.get_by_monitor(monitor_id).await?)
	}
}
