use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

use super::error::TenantRepositoryError;
use crate::models::{CreateTriggerRequest, TenantTrigger, UpdateTriggerRequest};
use crate::utils::current_tenant_id;

#[async_trait]
pub trait TenantTriggerRepositoryTrait: Clone + Send + Sync {
	async fn create(
		&self,
		request: CreateTriggerRequest,
	) -> Result<TenantTrigger, TenantRepositoryError>;
	async fn get(&self, trigger_id: &str) -> Result<TenantTrigger, TenantRepositoryError>;
	async fn get_by_uuid(&self, id: Uuid) -> Result<TenantTrigger, TenantRepositoryError>;
	async fn get_all(&self) -> Result<HashMap<String, TenantTrigger>, TenantRepositoryError>;
	async fn get_by_monitor(
		&self,
		monitor_id: Uuid,
	) -> Result<Vec<TenantTrigger>, TenantRepositoryError>;
	async fn update(
		&self,
		trigger_id: &str,
		request: UpdateTriggerRequest,
	) -> Result<TenantTrigger, TenantRepositoryError>;
	async fn delete(&self, trigger_id: &str) -> Result<(), TenantRepositoryError>;
	async fn list(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantTrigger>, TenantRepositoryError>;

	// Check if we can create more triggers for a monitor
	async fn check_quota(&self, monitor_id: Uuid) -> Result<bool, TenantRepositoryError>;
}

#[derive(Clone)]
pub struct TenantTriggerRepository {
	pool: Pool<Postgres>,
}

impl TenantTriggerRepository {
	pub fn new(pool: Pool<Postgres>) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl TenantTriggerRepositoryTrait for TenantTriggerRepository {
	async fn create(
		&self,
		request: CreateTriggerRequest,
	) -> Result<TenantTrigger, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Check quota
		if !self.check_quota(request.monitor_id).await? {
			return Err(TenantRepositoryError::QuotaExceeded(
				"Trigger quota exceeded for monitor".to_string(),
			));
		}

		// Verify monitor exists and belongs to tenant
		let monitor_exists = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_monitors WHERE tenant_id = $1 AND id = $2",
			tenant_id,
			request.monitor_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		if monitor_exists == 0 {
			return Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "monitor".to_string(),
				resource_id: request.monitor_id.to_string(),
			});
		}

		// Check if trigger_id already exists for this tenant
		let existing = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_triggers WHERE tenant_id = $1 AND trigger_id = $2",
			tenant_id,
			request.trigger_id
		)
		.fetch_one(&self.pool)
		.await?;

		if existing.unwrap_or(0) > 0 {
			return Err(TenantRepositoryError::AlreadyExists {
				resource_type: "trigger".to_string(),
				resource_id: request.trigger_id.clone(),
			});
		}

		let trigger = sqlx::query_as!(
			TenantTrigger,
			r#"
			INSERT INTO tenant_triggers (tenant_id, trigger_id, monitor_id, name, type, configuration)
			VALUES ($1, $2, $3, $4, $5, $6)
			RETURNING id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			"#,
			tenant_id,
			request.trigger_id,
			request.monitor_id,
			request.name,
			request.trigger_type,
			request.configuration
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(trigger)
	}

	async fn get(&self, trigger_id: &str) -> Result<TenantTrigger, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let trigger = sqlx::query_as!(
			TenantTrigger,
			r#"
			SELECT id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			FROM tenant_triggers 
			WHERE tenant_id = $1 AND trigger_id = $2
			"#,
			tenant_id,
			trigger_id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "trigger".to_string(),
			resource_id: trigger_id.to_string(),
		})?;

		Ok(trigger)
	}

	async fn get_by_uuid(&self, id: Uuid) -> Result<TenantTrigger, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let trigger = sqlx::query_as!(
			TenantTrigger,
			r#"
			SELECT id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			FROM tenant_triggers 
			WHERE tenant_id = $1 AND id = $2
			"#,
			tenant_id,
			id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "trigger".to_string(),
			resource_id: id.to_string(),
		})?;

		Ok(trigger)
	}

	async fn get_all(&self) -> Result<HashMap<String, TenantTrigger>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let triggers = sqlx::query_as!(
			TenantTrigger,
			r#"
			SELECT id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			FROM tenant_triggers 
			WHERE tenant_id = $1 AND is_active = true
			"#,
			tenant_id
		)
		.fetch_all(&self.pool)
		.await?;

		let map = triggers
			.into_iter()
			.map(|t| (t.trigger_id.clone(), t))
			.collect();

		Ok(map)
	}

	async fn get_by_monitor(
		&self,
		monitor_id: Uuid,
	) -> Result<Vec<TenantTrigger>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let triggers = sqlx::query_as!(
			TenantTrigger,
			r#"
			SELECT id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			FROM tenant_triggers 
			WHERE tenant_id = $1 AND monitor_id = $2 AND is_active = true
			ORDER BY created_at
			"#,
			tenant_id,
			monitor_id
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(triggers)
	}

	async fn update(
		&self,
		trigger_id: &str,
		request: UpdateTriggerRequest,
	) -> Result<TenantTrigger, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let trigger = sqlx::query_as!(
			TenantTrigger,
			r#"
			UPDATE tenant_triggers
			SET 
				name = COALESCE($3, name),
				configuration = COALESCE($4, configuration),
				is_active = COALESCE($5, is_active),
				updated_at = NOW()
			WHERE tenant_id = $1 AND trigger_id = $2
			RETURNING id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			"#,
			tenant_id,
			trigger_id,
			request.name,
			request.configuration,
			request.is_active
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "trigger".to_string(),
			resource_id: trigger_id.to_string(),
		})?;

		Ok(trigger)
	}

	async fn delete(&self, trigger_id: &str) -> Result<(), TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let result = sqlx::query!(
			"DELETE FROM tenant_triggers WHERE tenant_id = $1 AND trigger_id = $2",
			tenant_id,
			trigger_id
		)
		.execute(&self.pool)
		.await?;

		if result.rows_affected() == 0 {
			return Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "trigger".to_string(),
				resource_id: trigger_id.to_string(),
			});
		}

		Ok(())
	}

	async fn list(
		&self,
		limit: i64,
		offset: i64,
	) -> Result<Vec<TenantTrigger>, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		let triggers = sqlx::query_as!(
			TenantTrigger,
			r#"
			SELECT id, tenant_id, trigger_id, monitor_id, name, type as trigger_type, configuration, is_active, created_at, updated_at
			FROM tenant_triggers 
			WHERE tenant_id = $1 
			ORDER BY created_at DESC 
			LIMIT $2 OFFSET $3
			"#,
			tenant_id,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(triggers)
	}

	async fn check_quota(&self, monitor_id: Uuid) -> Result<bool, TenantRepositoryError> {
		let tenant_id = current_tenant_id();

		// Get tenant quota for triggers per monitor
		let max_triggers_per_monitor = sqlx::query_scalar!(
			"SELECT max_triggers_per_monitor FROM tenants WHERE id = $1",
			tenant_id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or(TenantRepositoryError::TenantNotFound(tenant_id))?;

		// Get current count for this monitor
		let current_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_triggers WHERE tenant_id = $1 AND monitor_id = $2",
			tenant_id,
			monitor_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		Ok(current_count < max_triggers_per_monitor.unwrap_or(0) as i64)
	}
}
