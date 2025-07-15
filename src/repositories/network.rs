use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

use super::error::TenantRepositoryError;
use crate::models::{CreateNetworkRequest, TenantNetwork, UpdateNetworkRequest};
use crate::utils::current_tenant_id;

#[async_trait]
pub trait TenantNetworkRepositoryTrait: Clone + Send + Sync {
	async fn create(
		&self,
		request: CreateNetworkRequest,
	) -> Result<TenantNetwork, TenantRepositoryError>;
	async fn get(&self, network_id: &str) -> Result<TenantNetwork, TenantRepositoryError>;
	async fn get_by_uuid(&self, id: Uuid) -> Result<TenantNetwork, TenantRepositoryError>;
	async fn get_all(&self) -> Result<HashMap<String, TenantNetwork>, TenantRepositoryError>;
	async fn update(
		&self,
		network_id: &str,
		request: UpdateNetworkRequest,
	) -> Result<TenantNetwork, TenantRepositoryError>;
	async fn delete(&self, network_id: &str) -> Result<(), TenantRepositoryError>;
	async fn list(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantNetwork>, TenantRepositoryError>;

	// Check if we can create more networks
	async fn check_quota(&self) -> Result<bool, TenantRepositoryError>;
}

#[derive(Clone)]
pub struct TenantNetworkRepository {
	pool: Pool<Postgres>,
}

impl TenantNetworkRepository {
	pub fn new(pool: Pool<Postgres>) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl TenantNetworkRepositoryTrait for TenantNetworkRepository {
	async fn create(
		&self,
		request: CreateNetworkRequest,
	) -> Result<TenantNetwork, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Check quota
		if !self.check_quota().await? {
			return Err(TenantRepositoryError::QuotaExceeded(
				"Network quota exceeded".to_string(),
			));
		}

		// Check if network_id already exists for this tenant
		let existing = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_networks WHERE tenant_id = $1 AND network_id = $2",
			tenant_id,
			request.network_id
		)
		.fetch_one(&self.pool)
		.await?;

		if existing.unwrap_or(0) > 0 {
			return Err(TenantRepositoryError::AlreadyExists {
				resource_type: "network".to_string(),
				resource_id: request.network_id.clone(),
			});
		}

		let network = sqlx::query_as!(
			TenantNetwork,
			r#"
			INSERT INTO tenant_networks (tenant_id, network_id, name, blockchain, configuration)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *
			"#,
			tenant_id,
			request.network_id,
			request.name,
			request.blockchain,
			request.configuration
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(network)
	}

	async fn get(&self, network_id: &str) -> Result<TenantNetwork, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let network = sqlx::query_as!(
			TenantNetwork,
			"SELECT * FROM tenant_networks WHERE tenant_id = $1 AND network_id = $2",
			tenant_id,
			network_id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "network".to_string(),
			resource_id: network_id.to_string(),
		})?;

		Ok(network)
	}

	async fn get_by_uuid(&self, id: Uuid) -> Result<TenantNetwork, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let network = sqlx::query_as!(
			TenantNetwork,
			"SELECT * FROM tenant_networks WHERE tenant_id = $1 AND id = $2",
			tenant_id,
			id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "network".to_string(),
			resource_id: id.to_string(),
		})?;

		Ok(network)
	}

	async fn get_all(&self) -> Result<HashMap<String, TenantNetwork>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let networks = sqlx::query_as!(
			TenantNetwork,
			"SELECT * FROM tenant_networks WHERE tenant_id = $1 AND is_active = true",
			tenant_id
		)
		.fetch_all(&self.pool)
		.await?;

		let map = networks
			.into_iter()
			.map(|n| (n.network_id.clone(), n))
			.collect();

		Ok(map)
	}

	async fn update(
		&self,
		network_id: &str,
		request: UpdateNetworkRequest,
	) -> Result<TenantNetwork, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let network = sqlx::query_as!(
			TenantNetwork,
			r#"
			UPDATE tenant_networks
			SET 
				name = COALESCE($3, name),
				configuration = COALESCE($4, configuration),
				is_active = COALESCE($5, is_active),
				updated_at = NOW()
			WHERE tenant_id = $1 AND network_id = $2
			RETURNING *
			"#,
			tenant_id,
			network_id,
			request.name,
			request.configuration,
			request.is_active
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "network".to_string(),
			resource_id: network_id.to_string(),
		})?;

		Ok(network)
	}

	async fn delete(&self, network_id: &str) -> Result<(), TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Check if any monitors are using this network
		let monitors_using = sqlx::query_scalar!(
			r#"
			SELECT COUNT(*) 
			FROM tenant_monitors 
			WHERE tenant_id = $1 
			AND network_id = (SELECT id FROM tenant_networks WHERE tenant_id = $1 AND network_id = $2)
			"#,
			tenant_id,
			network_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		if monitors_using > 0 {
			return Err(TenantRepositoryError::ValidationError(format!(
				"Cannot delete network: {} monitors are using it",
				monitors_using
			)));
		}

		let result = sqlx::query!(
			"DELETE FROM tenant_networks WHERE tenant_id = $1 AND network_id = $2",
			tenant_id,
			network_id
		)
		.execute(&self.pool)
		.await?;

		if result.rows_affected() == 0 {
			return Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "network".to_string(),
				resource_id: network_id.to_string(),
			});
		}

		Ok(())
	}

	async fn list(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantNetwork>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let networks = sqlx::query_as!(
			TenantNetwork,
			"SELECT * FROM tenant_networks WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
			tenant_id,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(networks)
	}

	async fn check_quota(&self) -> Result<bool, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Get tenant quota
		let max_networks =
			sqlx::query_scalar!("SELECT max_networks FROM tenants WHERE id = $1", tenant_id)
				.fetch_optional(&self.pool)
				.await?
				.ok_or(TenantRepositoryError::TenantNotFound(tenant_id))?;

		// Get current count
		let current_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_networks WHERE tenant_id = $1",
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		Ok(current_count < max_networks.unwrap_or(5) as i64)
	}
}
