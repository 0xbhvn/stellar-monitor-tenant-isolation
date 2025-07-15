use async_trait::async_trait;
use serde_json::Value as JsonValue;

use super::monitor_service::{AuditServiceTrait, ServiceError};
use crate::models::audit::ResourceType as AuditResourceType;
use crate::models::{
	AuditAction, CreateAuditLogRequest, CreateNetworkRequest, RequestMetadata, TenantNetwork,
	UpdateNetworkRequest,
};
use crate::repositories::{TenantNetworkRepositoryTrait, TenantRepositoryTrait};
use crate::utils::current_tenant_context;

#[async_trait]
pub trait NetworkServiceTrait: Send + Sync {
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

#[derive(Clone)]
pub struct NetworkService<N, T, A>
where
	N: TenantNetworkRepositoryTrait,
	T: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	network_repo: N,
	tenant_repo: T,
	audit_service: A,
}

impl<N, T, A> NetworkService<N, T, A>
where
	N: TenantNetworkRepositoryTrait,
	T: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	pub fn new(network_repo: N, tenant_repo: T, audit_service: A) -> Self {
		Self {
			network_repo,
			tenant_repo,
			audit_service,
		}
	}
}

#[async_trait]
impl<N, T, A> NetworkServiceTrait for NetworkService<N, T, A>
where
	N: TenantNetworkRepositoryTrait + Send + Sync,
	T: TenantRepositoryTrait + Send + Sync,
	A: AuditServiceTrait + Send + Sync,
{
	async fn create_network(
		&self,
		request: CreateNetworkRequest,
		metadata: RequestMetadata,
	) -> Result<TenantNetwork, ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to create networks".to_string(),
			));
		}

		// Check quota
		let quota_status = self.tenant_repo.get_quota_status(context.tenant_id).await?;
		if !quota_status.can_create_network() {
			return Err(ServiceError::QuotaExceeded(format!(
				"Network quota exceeded: {}/{}",
				quota_status.usage.networks_count, quota_status.quotas.max_networks
			)));
		}

		// Validate blockchain type
		if !["stellar", "evm"].contains(&request.blockchain.as_str()) {
			return Err(ServiceError::ValidationError(format!(
				"Invalid blockchain type: {}. Must be 'stellar' or 'evm'",
				request.blockchain
			)));
		}

		// Create network
		let network = self.network_repo.create(request.clone()).await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::NetworkCreated,
				resource_type: Some(AuditResourceType::Network),
				resource_id: Some(network.id),
				changes: Some(serde_json::to_value(&request).unwrap_or(JsonValue::Null)),
				ip_address: metadata.ip_address,
				user_agent: metadata.user_agent.clone(),
			})
			.await?;

		Ok(network)
	}

	async fn get_network(&self, network_id: &str) -> Result<TenantNetwork, ServiceError> {
		Ok(self.network_repo.get(network_id).await?)
	}

	async fn update_network(
		&self,
		network_id: &str,
		request: UpdateNetworkRequest,
		metadata: RequestMetadata,
	) -> Result<TenantNetwork, ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to update networks".to_string(),
			));
		}

		// Get existing network
		let existing = self.network_repo.get(network_id).await?;

		// Update network
		let network = self
			.network_repo
			.update(network_id, request.clone())
			.await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::NetworkUpdated,
				resource_type: Some(AuditResourceType::Network),
				resource_id: Some(existing.id),
				changes: Some(serde_json::to_value(&request).unwrap_or(JsonValue::Null)),
				ip_address: metadata.ip_address,
				user_agent: metadata.user_agent.clone(),
			})
			.await?;

		Ok(network)
	}

	async fn delete_network(
		&self,
		network_id: &str,
		metadata: RequestMetadata,
	) -> Result<(), ServiceError> {
		let context = current_tenant_context();

		// Check write permissions
		if !context.can_write() {
			return Err(ServiceError::AccessDenied(
				"Insufficient permissions to delete networks".to_string(),
			));
		}

		// Get network for audit log
		let network = self.network_repo.get(network_id).await?;

		// Delete network (repository will check for dependent monitors)
		self.network_repo.delete(network_id).await?;

		// Audit log
		self.audit_service
			.log(CreateAuditLogRequest {
				tenant_id: context.tenant_id,
				user_id: context.user.as_ref().map(|u| u.id),
				api_key_id: context.api_key_id,
				action: AuditAction::NetworkDeleted,
				resource_type: Some(AuditResourceType::Network),
				resource_id: Some(network.id),
				changes: None,
				ip_address: metadata.ip_address,
				user_agent: metadata.user_agent.clone(),
			})
			.await?;

		Ok(())
	}

	async fn list_networks(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantNetwork>, ServiceError> {
		Ok(self.network_repo.list(limit, offset).await?)
	}

	async fn get_network_count(&self) -> Result<i64, ServiceError> {
		let networks = self.network_repo.get_all().await?;
		Ok(networks.len() as i64)
	}
}
