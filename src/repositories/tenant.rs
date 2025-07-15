use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::error::TenantRepositoryError;
use crate::models::{
	AvailableResources, CreateTenantRequest, CurrentUsage, ResourceQuotaStatus, Tenant,
	TenantMembership, TenantQuotas, TenantRole, UpdateTenantRequest,
};

#[async_trait]
pub trait TenantRepositoryTrait: Clone + Send + Sync {
	async fn create(&self, request: CreateTenantRequest) -> Result<Tenant, TenantRepositoryError>;
	async fn get(&self, tenant_id: Uuid) -> Result<Tenant, TenantRepositoryError>;
	async fn get_by_slug(&self, slug: &str) -> Result<Tenant, TenantRepositoryError>;
	async fn update(
		&self,
		tenant_id: Uuid,
		request: UpdateTenantRequest,
	) -> Result<Tenant, TenantRepositoryError>;
	async fn delete(&self, tenant_id: Uuid) -> Result<(), TenantRepositoryError>;
	async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>, TenantRepositoryError>;

	// Membership management
	async fn add_member(
		&self,
		tenant_id: Uuid,
		user_id: Uuid,
		role: TenantRole,
	) -> Result<TenantMembership, TenantRepositoryError>;
	async fn remove_member(
		&self,
		tenant_id: Uuid,
		user_id: Uuid,
	) -> Result<(), TenantRepositoryError>;
	async fn update_member_role(
		&self,
		tenant_id: Uuid,
		user_id: Uuid,
		role: TenantRole,
	) -> Result<TenantMembership, TenantRepositoryError>;
	async fn get_members(
		&self,
		tenant_id: Uuid,
	) -> Result<Vec<TenantMembership>, TenantRepositoryError>;
	async fn get_user_tenants(
		&self,
		user_id: Uuid,
	) -> Result<Vec<(Tenant, TenantRole)>, TenantRepositoryError>;

	// Resource quota management
	async fn get_quota_status(
		&self,
		tenant_id: Uuid,
	) -> Result<ResourceQuotaStatus, TenantRepositoryError>;
	async fn check_quota(
		&self,
		tenant_id: Uuid,
		resource: &str,
		amount: i32,
	) -> Result<bool, TenantRepositoryError>;
}

#[derive(Clone)]
pub struct TenantRepository {
	pool: Pool<Postgres>,
}

impl TenantRepository {
	pub fn new(pool: Pool<Postgres>) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl TenantRepositoryTrait for TenantRepository {
	async fn create(&self, request: CreateTenantRequest) -> Result<Tenant, TenantRepositoryError> {
		let tenant = sqlx::query_as!(
			Tenant,
			r#"
			INSERT INTO tenants (name, slug, max_monitors, max_networks, max_triggers_per_monitor, max_rpc_requests_per_minute, max_storage_mb)
			VALUES ($1, $2, $3, $4, $5, $6, $7)
			RETURNING id, name, slug,
			          COALESCE(is_active, true) as "is_active!",
			          COALESCE(max_monitors, 10) as "max_monitors!",
			          COALESCE(max_networks, 5) as "max_networks!",
			          COALESCE(max_triggers_per_monitor, 3) as "max_triggers_per_monitor!",
			          COALESCE(max_rpc_requests_per_minute, 1000) as "max_rpc_requests_per_minute!",
			          COALESCE(max_storage_mb, 1000) as "max_storage_mb!",
			          created_at, updated_at
			"#,
			request.name,
			request.slug,
			request.max_monitors.unwrap_or(10),
			request.max_networks.unwrap_or(5),
			request.max_triggers_per_monitor.unwrap_or(10),
			request.max_rpc_requests_per_minute.unwrap_or(1000),
			request.max_storage_mb.unwrap_or(1000)
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(tenant)
	}

	async fn get(&self, tenant_id: Uuid) -> Result<Tenant, TenantRepositoryError> {
		let tenant = sqlx::query_as!(
			Tenant,
			r#"
			SELECT id, name, slug,
			       COALESCE(is_active, true) as "is_active!",
			       COALESCE(max_monitors, 10) as "max_monitors!",
			       COALESCE(max_networks, 5) as "max_networks!",
			       COALESCE(max_triggers_per_monitor, 3) as "max_triggers_per_monitor!",
			       COALESCE(max_rpc_requests_per_minute, 1000) as "max_rpc_requests_per_minute!",
			       COALESCE(max_storage_mb, 1000) as "max_storage_mb!",
			       created_at, updated_at
			FROM tenants 
			WHERE id = $1
			"#,
			tenant_id
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or(TenantRepositoryError::TenantNotFound(tenant_id))?;

		Ok(tenant)
	}

	async fn get_by_slug(&self, slug: &str) -> Result<Tenant, TenantRepositoryError> {
		let tenant = sqlx::query_as!(
			Tenant,
			r#"
			SELECT id, name, slug,
			       COALESCE(is_active, true) as "is_active!",
			       COALESCE(max_monitors, 10) as "max_monitors!",
			       COALESCE(max_networks, 5) as "max_networks!",
			       COALESCE(max_triggers_per_monitor, 3) as "max_triggers_per_monitor!",
			       COALESCE(max_rpc_requests_per_minute, 1000) as "max_rpc_requests_per_minute!",
			       COALESCE(max_storage_mb, 1000) as "max_storage_mb!",
			       created_at, updated_at
			FROM tenants 
			WHERE slug = $1
			"#,
			slug
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "tenant".to_string(),
			resource_id: slug.to_string(),
		})?;

		Ok(tenant)
	}

	async fn update(
		&self,
		tenant_id: Uuid,
		request: UpdateTenantRequest,
	) -> Result<Tenant, TenantRepositoryError> {
		let mut query = String::from("UPDATE tenants SET updated_at = NOW()");
		let mut params: Vec<String> = vec![];
		let mut param_count = 1;

		if let Some(_name) = &request.name {
			params.push(format!(", name = ${}", param_count + 1));
			param_count += 1;
		}
		if let Some(_is_active) = request.is_active {
			params.push(format!(", is_active = ${}", param_count + 1));
			param_count += 1;
		}
		if let Some(_max_monitors) = request.max_monitors {
			params.push(format!(", max_monitors = ${}", param_count + 1));
			param_count += 1;
		}
		if let Some(_max_networks) = request.max_networks {
			params.push(format!(", max_networks = ${}", param_count + 1));
			param_count += 1;
		}
		if let Some(_max_triggers_per_monitor) = request.max_triggers_per_monitor {
			params.push(format!(", max_triggers_per_monitor = ${}", param_count + 1));
			param_count += 1;
		}
		if let Some(_max_rpc_requests_per_minute) = request.max_rpc_requests_per_minute {
			params.push(format!(
				", max_rpc_requests_per_minute = ${}",
				param_count + 1
			));
			param_count += 1;
		}
		if let Some(_max_storage_mb) = request.max_storage_mb {
			params.push(format!(", max_storage_mb = ${}", param_count + 1));
			// param_count += 1; // Removed as it's the last one
		}

		query.push_str(&params.join(""));
		query.push_str(" WHERE id = $1 RETURNING *");

		// For now, we'll use a simpler approach with direct query
		let tenant = sqlx::query_as!(
			Tenant,
			r#"
			UPDATE tenants 
			SET 
				name = COALESCE($2, name),
				is_active = COALESCE($3, is_active),
				max_monitors = COALESCE($4, max_monitors),
				max_networks = COALESCE($5, max_networks),
				max_triggers_per_monitor = COALESCE($6, max_triggers_per_monitor),
				max_rpc_requests_per_minute = COALESCE($7, max_rpc_requests_per_minute),
				max_storage_mb = COALESCE($8, max_storage_mb),
				updated_at = NOW()
			WHERE id = $1
			RETURNING id, name, slug,
			          COALESCE(is_active, true) as "is_active!",
			          COALESCE(max_monitors, 10) as "max_monitors!",
			          COALESCE(max_networks, 5) as "max_networks!",
			          COALESCE(max_triggers_per_monitor, 3) as "max_triggers_per_monitor!",
			          COALESCE(max_rpc_requests_per_minute, 1000) as "max_rpc_requests_per_minute!",
			          COALESCE(max_storage_mb, 1000) as "max_storage_mb!",
			          created_at, updated_at
			"#,
			tenant_id,
			request.name,
			request.is_active,
			request.max_monitors,
			request.max_networks,
			request.max_triggers_per_monitor,
			request.max_rpc_requests_per_minute,
			request.max_storage_mb
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(tenant)
	}

	async fn delete(&self, tenant_id: Uuid) -> Result<(), TenantRepositoryError> {
		let result = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
			.execute(&self.pool)
			.await?;

		if result.rows_affected() == 0 {
			return Err(TenantRepositoryError::TenantNotFound(tenant_id));
		}

		Ok(())
	}

	async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>, TenantRepositoryError> {
		let tenants = sqlx::query_as!(
			Tenant,
			r#"
			SELECT id, name, slug, 
			       COALESCE(is_active, true) as "is_active!",
			       COALESCE(max_monitors, 10) as "max_monitors!",
			       COALESCE(max_networks, 5) as "max_networks!",
			       COALESCE(max_triggers_per_monitor, 3) as "max_triggers_per_monitor!",
			       COALESCE(max_rpc_requests_per_minute, 1000) as "max_rpc_requests_per_minute!",
			       COALESCE(max_storage_mb, 1000) as "max_storage_mb!",
			       created_at, updated_at
			FROM tenants 
			ORDER BY created_at DESC 
			LIMIT $1 OFFSET $2
			"#,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(tenants)
	}

	async fn add_member(
		&self,
		tenant_id: Uuid,
		user_id: Uuid,
		role: TenantRole,
	) -> Result<TenantMembership, TenantRepositoryError> {
		let membership = sqlx::query_as!(
			TenantMembership,
			r#"
			INSERT INTO tenant_memberships (tenant_id, user_id, role)
			VALUES ($1, $2, $3)
			RETURNING id, tenant_id, user_id, role as "role: TenantRole", created_at, updated_at
			"#,
			tenant_id,
			user_id,
			role as TenantRole
		)
		.fetch_one(&self.pool)
		.await?;

		Ok(membership)
	}

	async fn remove_member(
		&self,
		tenant_id: Uuid,
		user_id: Uuid,
	) -> Result<(), TenantRepositoryError> {
		let result = sqlx::query!(
			"DELETE FROM tenant_memberships WHERE tenant_id = $1 AND user_id = $2",
			tenant_id,
			user_id
		)
		.execute(&self.pool)
		.await?;

		if result.rows_affected() == 0 {
			return Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "membership".to_string(),
				resource_id: format!("{}/{}", tenant_id, user_id),
			});
		}

		Ok(())
	}

	async fn update_member_role(
		&self,
		tenant_id: Uuid,
		user_id: Uuid,
		role: TenantRole,
	) -> Result<TenantMembership, TenantRepositoryError> {
		let membership = sqlx::query_as!(
			TenantMembership,
			r#"
			UPDATE tenant_memberships 
			SET role = $3, updated_at = NOW()
			WHERE tenant_id = $1 AND user_id = $2
			RETURNING id, tenant_id, user_id, role as "role: TenantRole", created_at, updated_at
			"#,
			tenant_id,
			user_id,
			role as TenantRole
		)
		.fetch_optional(&self.pool)
		.await?
		.ok_or_else(|| TenantRepositoryError::ResourceNotFound {
			resource_type: "membership".to_string(),
			resource_id: format!("{}/{}", tenant_id, user_id),
		})?;

		Ok(membership)
	}

	async fn get_members(
		&self,
		tenant_id: Uuid,
	) -> Result<Vec<TenantMembership>, TenantRepositoryError> {
		let members = sqlx::query_as!(
			TenantMembership,
			r#"
			SELECT id, tenant_id, user_id, role as "role: TenantRole", created_at, updated_at
			FROM tenant_memberships 
			WHERE tenant_id = $1
			ORDER BY created_at
			"#,
			tenant_id
		)
		.fetch_all(&self.pool)
		.await?;

		Ok(members)
	}

	async fn get_user_tenants(
		&self,
		user_id: Uuid,
	) -> Result<Vec<(Tenant, TenantRole)>, TenantRepositoryError> {
		let results = sqlx::query!(
			r#"
			SELECT t.*, tm.role
			FROM tenants t
			INNER JOIN tenant_memberships tm ON t.id = tm.tenant_id
			WHERE tm.user_id = $1 AND t.is_active = true
			ORDER BY t.created_at DESC
			"#,
			user_id
		)
		.fetch_all(&self.pool)
		.await?;

		let tenants = results
			.into_iter()
			.map(|row| {
				let role = match row.role.as_str() {
					"owner" => TenantRole::Owner,
					"admin" => TenantRole::Admin,
					"member" => TenantRole::Member,
					"viewer" => TenantRole::Viewer,
					_ => TenantRole::Viewer,
				};

				let tenant = Tenant {
					id: row.id,
					name: row.name,
					slug: row.slug,
					is_active: row.is_active.unwrap_or(true),
					max_monitors: row.max_monitors.unwrap_or(10),
					max_networks: row.max_networks.unwrap_or(5),
					max_triggers_per_monitor: row.max_triggers_per_monitor.unwrap_or(3),
					max_rpc_requests_per_minute: row.max_rpc_requests_per_minute.unwrap_or(1000),
					max_storage_mb: row.max_storage_mb.unwrap_or(1000),
					created_at: row.created_at,
					updated_at: row.updated_at,
				};

				(tenant, role)
			})
			.collect();

		Ok(tenants)
	}

	async fn get_quota_status(
		&self,
		tenant_id: Uuid,
	) -> Result<ResourceQuotaStatus, TenantRepositoryError> {
		// Get tenant quotas
		let tenant = self.get(tenant_id).await?;

		// Get current usage counts
		let monitor_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_monitors WHERE tenant_id = $1",
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0) as i32;

		let network_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_networks WHERE tenant_id = $1",
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0) as i32;

		let trigger_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM tenant_triggers WHERE tenant_id = $1",
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0) as i32;

		// Get RPC usage from last minute
		let rpc_requests = sqlx::query_scalar!(
			r#"
			SELECT COALESCE(SUM(usage_value), 0)::integer
			FROM resource_usage
			WHERE tenant_id = $1 
			AND resource_type = 'rpc_requests'
			AND created_at >= NOW() - INTERVAL '1 minute'
			"#,
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		// Get storage usage
		let storage_mb = sqlx::query_scalar!(
			r#"
			SELECT COALESCE(usage_value, 0)::integer
			FROM resource_usage
			WHERE tenant_id = $1 
			AND resource_type = 'storage'
			AND usage_date = CURRENT_DATE
			ORDER BY created_at DESC
			LIMIT 1
			"#,
			tenant_id
		)
		.fetch_one(&self.pool)
		.await?
		.unwrap_or(0);

		let quotas = TenantQuotas {
			max_monitors: tenant.max_monitors,
			max_networks: tenant.max_networks,
			max_triggers_per_monitor: tenant.max_triggers_per_monitor,
			max_rpc_requests_per_minute: tenant.max_rpc_requests_per_minute,
			max_storage_mb: tenant.max_storage_mb,
		};

		let usage = CurrentUsage {
			monitors_count: monitor_count,
			networks_count: network_count,
			triggers_count: trigger_count,
			rpc_requests_last_minute: rpc_requests,
			storage_mb_used: storage_mb,
		};

		let available = AvailableResources {
			monitors: (quotas.max_monitors - usage.monitors_count).max(0),
			networks: (quotas.max_networks - usage.networks_count).max(0),
			triggers: (quotas.max_triggers_per_monitor * usage.monitors_count
				- usage.triggers_count)
				.max(0),
			rpc_requests_per_minute: (quotas.max_rpc_requests_per_minute
				- usage.rpc_requests_last_minute)
				.max(0),
			storage_mb: (quotas.max_storage_mb - usage.storage_mb_used).max(0),
		};

		Ok(ResourceQuotaStatus {
			tenant_id,
			quotas,
			usage,
			available,
		})
	}

	async fn check_quota(
		&self,
		tenant_id: Uuid,
		resource: &str,
		amount: i32,
	) -> Result<bool, TenantRepositoryError> {
		let status = self.get_quota_status(tenant_id).await?;

		let has_capacity = match resource {
			"monitors" => status.available.monitors >= amount,
			"networks" => status.available.networks >= amount,
			"triggers" => status.available.triggers >= amount,
			"rpc_requests" => status.available.rpc_requests_per_minute >= amount,
			"storage_mb" => status.available.storage_mb >= amount,
			_ => {
				return Err(TenantRepositoryError::InvalidConfiguration(format!(
					"Unknown resource type: {}",
					resource
				)))
			}
		};

		Ok(has_capacity)
	}
}
