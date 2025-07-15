use async_trait::async_trait;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use super::monitor_service::{AuditServiceTrait, ServiceError};
use crate::models::audit::ResourceType;
use crate::models::{AuditLog, CreateAuditLogRequest};

#[derive(Clone)]
pub struct AuditService {
	pool: Pool<Postgres>,
}

impl AuditService {
	pub fn new(pool: Pool<Postgres>) -> Self {
		Self { pool }
	}
}

#[async_trait]
impl AuditServiceTrait for AuditService {
	async fn log(&self, request: CreateAuditLogRequest) -> Result<(), ServiceError> {
		let action_str = request.action.as_str();
		let resource_type_str = request.resource_type.as_ref().map(|rt| rt.as_str());

		sqlx::query!(
			r#"
			INSERT INTO audit_logs (
				tenant_id, user_id, api_key_id, action, resource_type, 
				resource_id, changes, ip_address, user_agent
			)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
			"#,
			request.tenant_id,
			request.user_id,
			request.api_key_id,
			action_str,
			resource_type_str,
			request.resource_id,
			request.changes,
			request.ip_address.map(|ip| IpNetwork::from(ip)),
			request.user_agent
		)
		.execute(&self.pool)
		.await
		.map_err(|e| ServiceError::Internal(format!("Failed to write audit log: {}", e)))?;

		Ok(())
	}
}

// Additional audit query methods
impl AuditService {
	pub async fn get_tenant_logs(
		&self,
		tenant_id: uuid::Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<AuditLog>, ServiceError> {
		let logs = sqlx::query_as!(
			AuditLog,
			r#"
			SELECT 
				id, tenant_id, user_id, api_key_id, action, 
				resource_type, resource_id, changes, 
				ip_address as "ip_address: _", 
				user_agent, created_at
			FROM audit_logs
			WHERE tenant_id = $1
			ORDER BY created_at DESC
			LIMIT $2 OFFSET $3
			"#,
			tenant_id,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await
		.map_err(|e| ServiceError::Internal(format!("Failed to fetch audit logs: {}", e)))?;

		Ok(logs)
	}

	pub async fn get_user_logs(
		&self,
		tenant_id: uuid::Uuid,
		user_id: uuid::Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<AuditLog>, ServiceError> {
		let logs = sqlx::query_as!(
			AuditLog,
			r#"
			SELECT 
				id, tenant_id, user_id, api_key_id, action, 
				resource_type, resource_id, changes, 
				ip_address as "ip_address: _", 
				user_agent, created_at
			FROM audit_logs
			WHERE tenant_id = $1 AND user_id = $2
			ORDER BY created_at DESC
			LIMIT $3 OFFSET $4
			"#,
			tenant_id,
			user_id,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await
		.map_err(|e| ServiceError::Internal(format!("Failed to fetch user audit logs: {}", e)))?;

		Ok(logs)
	}

	pub async fn get_resource_logs(
		&self,
		tenant_id: uuid::Uuid,
		resource_type: ResourceType,
		resource_id: uuid::Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<AuditLog>, ServiceError> {
		let resource_type_str = resource_type.as_str();

		let logs = sqlx::query_as!(
			AuditLog,
			r#"
			SELECT 
				id, tenant_id, user_id, api_key_id, action, 
				resource_type, resource_id, changes, 
				ip_address as "ip_address: _", 
				user_agent, created_at
			FROM audit_logs
			WHERE tenant_id = $1 AND resource_type = $2 AND resource_id = $3
			ORDER BY created_at DESC
			LIMIT $4 OFFSET $5
			"#,
			tenant_id,
			resource_type_str,
			resource_id,
			limit,
			offset
		)
		.fetch_all(&self.pool)
		.await
		.map_err(|e| {
			ServiceError::Internal(format!("Failed to fetch resource audit logs: {}", e))
		})?;

		Ok(logs)
	}
}

#[async_trait]
impl<T: AuditServiceTrait> AuditServiceTrait for Arc<T> {
	async fn log(&self, request: CreateAuditLogRequest) -> Result<(), ServiceError> {
		(**self).log(request).await
	}
}
