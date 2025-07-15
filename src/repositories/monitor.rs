use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

use super::error::TenantRepositoryError;
use crate::models::{CreateMonitorRequest, TenantMonitor, UpdateMonitorRequest};
use crate::utils::current_tenant_id;

// This trait mimics the OpenZeppelin Monitor's MonitorRepositoryTrait
// but adds tenant isolation
#[async_trait]
pub trait TenantMonitorRepositoryTrait: Clone + Send + Sync {
	async fn create(
		&self,
		request: CreateMonitorRequest,
	) -> Result<TenantMonitor, TenantRepositoryError>;
	async fn get(&self, monitor_id: &str) -> Result<TenantMonitor, TenantRepositoryError>;
	async fn get_by_uuid(&self, id: Uuid) -> Result<TenantMonitor, TenantRepositoryError>;
	async fn get_all(&self) -> Result<HashMap<String, TenantMonitor>, TenantRepositoryError>;
	async fn update(
		&self,
		monitor_id: &str,
		request: UpdateMonitorRequest,
	) -> Result<TenantMonitor, TenantRepositoryError>;
	async fn delete(&self, monitor_id: &str) -> Result<(), TenantRepositoryError>;
	async fn list(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantMonitor>, TenantRepositoryError>;

	// Check if we can create more monitors
	async fn check_quota(&self) -> Result<bool, TenantRepositoryError>;
}

#[derive(Clone)]
pub struct TenantMonitorRepository {
	pool: Pool<Postgres>,
}

impl TenantMonitorRepository {
	pub fn new(pool: Pool<Postgres>) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl TenantMonitorRepositoryTrait for TenantMonitorRepository {
	async fn create(
		&self,
		request: CreateMonitorRequest,
	) -> Result<TenantMonitor, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Check quota
		if !self.check_quota().await? {
			return Err(TenantRepositoryError::QuotaExceeded(
				"Monitor quota exceeded".to_string(),
			));
		}

		// Check if monitor_id already exists for this tenant
		let existing = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_monitors WHERE tenant_id = $1 AND monitor_id = $2",
			tenant_id,
			request.monitor_id
		)
		.fetch_one(&self.pool)
		.await?;

		if existing.unwrap_or(0) > 0 {
			return Err(TenantRepositoryError::AlreadyExists {
				resource_type: "monitor".to_string(),
				resource_id: request.monitor_id.clone(),
			});
		}

		let monitor = sqlx::query_as!(
			TenantMonitor,
			r#"
			INSERT INTO tenant_monitors (tenant_id, monitor_id, name, network_id, configuration)
			VALUES ($1, $2, $3, $4, $5)
			RETURNING *
			"#,
			tenant_id,
			request.monitor_id,
			request.name,
			request.network_id,
			request.configuration
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(monitor)
	}

	async fn get(&self, monitor_id: &str) -> Result<TenantMonitor, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let monitor = sqlx::query_as!(
			TenantMonitor,
			"SELECT * FROM tenant_monitors WHERE tenant_id = $1 AND monitor_id = $2",
			tenant_id,
			monitor_id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "monitor".to_string(),
			resource_id: monitor_id.to_string(),
		})?;

		Ok(monitor)
	}

	async fn get_by_uuid(&self, id: Uuid) -> Result<TenantMonitor, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let monitor = sqlx::query_as!(
			TenantMonitor,
			"SELECT * FROM tenant_monitors WHERE tenant_id = $1 AND id = $2",
			tenant_id,
			id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "monitor".to_string(),
			resource_id: id.to_string(),
		})?;

		Ok(monitor)
	}

	async fn get_all(&self) -> Result<HashMap<String, TenantMonitor>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let monitors = sqlx::query_as!(
			TenantMonitor,
			"SELECT * FROM tenant_monitors WHERE tenant_id = $1 AND is_active = true",
			tenant_id
		)
		.fetch_all(&self.pool)
		.await?;

		let map = monitors
			.into_iter()
			.map(|m| (m.monitor_id.clone(), m))
			.collect();

		Ok(map)
	}

	async fn update(
		&self,
		monitor_id: &str,
		request: UpdateMonitorRequest,
	) -> Result<TenantMonitor, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let monitor = sqlx::query_as!(
			TenantMonitor,
			r#"
			UPDATE tenant_monitors
			SET 
				name = COALESCE($3, name),
				configuration = COALESCE($4, configuration),
				is_active = COALESCE($5, is_active),
				updated_at = NOW()
			WHERE tenant_id = $1 AND monitor_id = $2
			RETURNING *
			"#,
			tenant_id,
			monitor_id,
			request.name,
			request.configuration,
			request.is_active
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "monitor".to_string(),
			resource_id: monitor_id.to_string(),
		})?;

		Ok(monitor)
	}

	async fn delete(&self, monitor_id: &str) -> Result<(), TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let result = sqlx::query!(
			"DELETE FROM tenant_monitors WHERE tenant_id = $1 AND monitor_id = $2",
			tenant_id,
			monitor_id
		)
		.execute(&self.pool)
		.await?;

		if result.rows_affected() == 0 {
			return Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "monitor".to_string(),
				resource_id: monitor_id.to_string(),
			});
		}

		Ok(())
	}

	async fn list(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantMonitor>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let monitors = sqlx::query_as!(
			TenantMonitor,
			"SELECT * FROM tenant_monitors WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
			tenant_id,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(monitors)
	}

	async fn check_quota(&self) -> Result<bool, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Get tenant quota
		let max_monitors =
			sqlx::query_scalar!("SELECT max_monitors FROM tenants WHERE id = $1", tenant_id)
				.fetch_optional(&self.pool)
				.await?
				.ok_or(TenantRepositoryError::TenantNotFound(tenant_id))?;

		// Get current count
		let current_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_monitors WHERE tenant_id = $1",
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		Ok(current_count < max_monitors.unwrap_or(10) as i64)
	}
}
