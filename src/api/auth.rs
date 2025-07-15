use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::handlers::{ApiError, ApiResponse};
use crate::models::*;
use crate::services::ServiceError;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
	pub email: String,
	pub password: String,
	pub tenant_name: String,
	pub tenant_slug: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
	pub user: User,
	pub tenant: Tenant,
	pub access_token: String,
	pub refresh_token: String,
}

pub async fn register<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: crate::services::MonitorServiceTrait,
	N: crate::services::NetworkServiceTrait,
	T: crate::services::TriggerServiceTrait,
	TR: crate::repositories::TenantRepositoryTrait,
	A: crate::services::AuditServiceTrait,
{
	// Validate email
	if !email_address::EmailAddress::is_valid(&request.email) {
		return Err(ApiError::BadRequest("Invalid email address".to_string()));
	}

	// Hash password
	let password_hash = state
		.auth_service
		.hash_password(&request.password)
		.map_err(|_| ApiError::Internal)?;

	// Start transaction
	let mut tx = state.pool.begin().await.map_err(|_| ApiError::Internal)?;

	// Create tenant
	let tenant = sqlx::query_as!(
		Tenant,
		r#"
		INSERT INTO tenants (name, slug)
		VALUES ($1, $2)
		RETURNING id, name, slug,
		          COALESCE(is_active, true) as "is_active!",
		          COALESCE(max_monitors, 10) as "max_monitors!",
		          COALESCE(max_networks, 5) as "max_networks!",
		          COALESCE(max_triggers_per_monitor, 3) as "max_triggers_per_monitor!",
		          COALESCE(max_rpc_requests_per_minute, 1000) as "max_rpc_requests_per_minute!",
		          COALESCE(max_storage_mb, 1000) as "max_storage_mb!",
		          created_at, updated_at
		"#,
		request.tenant_name,
		request.tenant_slug
	)
	.fetch_one(&mut *tx)
	.await
	.map_err(|e| match e {
		sqlx::Error::Database(ref err) if err.message().contains("unique") => {
			ApiError::BadRequest("Tenant slug already exists".to_string())
		}
		_ => ApiError::Internal,
	})?;

	// Create user
	let user = sqlx::query_as!(
		User,
		r#"
		INSERT INTO users (email, password_hash)
		VALUES ($1, $2)
		RETURNING id, email, password_hash,
		          COALESCE(is_active, true) as "is_active!",
		          created_at, updated_at
		"#,
		request.email,
		password_hash
	)
	.fetch_one(&mut *tx)
	.await
	.map_err(|e| match e {
		sqlx::Error::Database(ref err) if err.message().contains("unique") => {
			ApiError::BadRequest("Email already registered".to_string())
		}
		_ => ApiError::Internal,
	})?;

	// Add user as owner of tenant
	sqlx::query!(
		r#"
		INSERT INTO tenant_memberships (tenant_id, user_id, role)
		VALUES ($1, $2, 'owner')
		"#,
		tenant.id,
		user.id
	)
	.execute(&mut *tx)
	.await
	.map_err(|_| ApiError::Internal)?;

	// Commit transaction
	tx.commit().await.map_err(|_| ApiError::Internal)?;

	// Generate tokens
	let access_token = state
		.auth_service
		.generate_jwt(&user)
		.map_err(|_| ApiError::Internal)?;
	let refresh_token = state
		.auth_service
		.generate_refresh_token(&user)
		.map_err(|_| ApiError::Internal)?;

	Ok((
		StatusCode::CREATED,
		Json(ApiResponse {
			data: RegisterResponse {
				user,
				tenant,
				access_token,
				refresh_token,
			},
			meta: None,
		}),
	))
}

pub async fn login<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: crate::services::MonitorServiceTrait,
	N: crate::services::NetworkServiceTrait,
	T: crate::services::TriggerServiceTrait,
	TR: crate::repositories::TenantRepositoryTrait,
	A: crate::services::AuditServiceTrait,
{
	// Find user
	let user = sqlx::query_as!(
		User,
		r#"
		SELECT id, email, password_hash,
		       COALESCE(is_active, true) as "is_active!",
		       created_at, updated_at
		FROM users 
		WHERE email = $1 AND COALESCE(is_active, true) = true
		"#,
		request.email
	)
	.fetch_optional(&state.pool)
	.await
	.map_err(|_| ApiError::Internal)?
	.ok_or(ApiError::Unauthorized)?;

	// Verify password
	let password_valid = state
		.auth_service
		.verify_password(&request.password, &user.password_hash)
		.map_err(|_| ApiError::Internal)?;

	if !password_valid {
		return Err(ApiError::Unauthorized);
	}

	// Get user's tenants
	let user_tenants = sqlx::query!(
		r#"
		SELECT t.id, t.name, t.slug, tm.role
		FROM tenants t
		INNER JOIN tenant_memberships tm ON t.id = tm.tenant_id
		WHERE tm.user_id = $1 AND t.is_active = true
		ORDER BY t.created_at DESC
		"#,
		user.id
	)
	.fetch_all(&state.pool)
	.await
	.map_err(|_| ApiError::Internal)?;

	let tenants: Vec<UserTenant> = user_tenants
		.into_iter()
		.map(|row| UserTenant {
			tenant_id: row.id,
			tenant_name: row.name,
			tenant_slug: row.slug,
			role: match row.role.as_str() {
				"owner" => TenantRole::Owner,
				"admin" => TenantRole::Admin,
				"member" => TenantRole::Member,
				"viewer" => TenantRole::Viewer,
				_ => TenantRole::Viewer,
			},
		})
		.collect();

	// Generate tokens
	let access_token = state
		.auth_service
		.generate_jwt(&user)
		.map_err(|_| ApiError::Internal)?;
	let refresh_token = state
		.auth_service
		.generate_refresh_token(&user)
		.map_err(|_| ApiError::Internal)?;

	Ok(Json(ApiResponse {
		data: LoginResponse {
			access_token,
			refresh_token,
			expires_in: 86400, // 24 hours
			user: UserInfo {
				id: user.id,
				email: user.email,
				tenants,
			},
		},
		meta: None,
	}))
}

pub async fn create_api_key<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(tenant_slug): Path<String>,
	Json(request): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, ApiError>
where
	M: crate::services::MonitorServiceTrait,
	N: crate::services::NetworkServiceTrait,
	T: crate::services::TriggerServiceTrait,
	TR: crate::repositories::TenantRepositoryTrait,
	A: crate::services::AuditServiceTrait,
{
	let context = crate::utils::current_tenant_context();

	// Verify user can manage tenant
	if !context.can_manage() {
		return Err(ApiError::Service(ServiceError::AccessDenied(
			"Insufficient permissions to create API keys".to_string(),
		)));
	}

	// Get tenant
	let tenant = state
		.tenant_repo
		.get_by_slug(&tenant_slug)
		.await
		.map_err(|e| ApiError::Service(crate::services::ServiceError::Repository(e)))?;

	// Verify user has access to this tenant
	if tenant.id != context.tenant_id {
		return Err(ApiError::Service(ServiceError::AccessDenied(
			"Access denied to this tenant".to_string(),
		)));
	}

	// Generate API key
	let api_key = state.auth_service.generate_api_key();
	let key_hash = state
		.auth_service
		.hash_password(&api_key)
		.map_err(|_| ApiError::Internal)?;

	// Store API key
	let stored_key = sqlx::query!(
		r#"
		INSERT INTO api_keys (tenant_id, name, key_hash, permissions, expires_at)
		VALUES ($1, $2, $3, $4, $5)
		RETURNING id, created_at
		"#,
		tenant.id,
		request.name,
		key_hash,
		serde_json::to_value(&request.permissions).unwrap(),
		request.expires_at
	)
	.fetch_one(&state.pool)
	.await
	.map_err(|_| ApiError::Internal)?;

	Ok((
		StatusCode::CREATED,
		Json(ApiResponse {
			data: CreateApiKeyResponse {
				id: stored_key.id,
				name: request.name.clone(),
				key: format!(
					"{}_{}",
					crate::config::Config::default().auth.api_key_prefix,
					api_key
				),
				permissions: request.permissions.clone(),
				expires_at: request.expires_at,
				created_at: stored_key.created_at.unwrap_or_else(|| chrono::Utc::now()),
			},
			meta: None,
		}),
	))
}

pub async fn list_api_keys<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path(_tenant_slug): Path<String>,
) -> Result<impl IntoResponse, ApiError>
where
	M: crate::services::MonitorServiceTrait,
	N: crate::services::NetworkServiceTrait,
	T: crate::services::TriggerServiceTrait,
	TR: crate::repositories::TenantRepositoryTrait,
	A: crate::services::AuditServiceTrait,
{
	let context = crate::utils::current_tenant_context();

	// List API keys (without the actual key values)
	let keys = sqlx::query!(
		r#"
		SELECT id, name, permissions, last_used_at, expires_at, is_active, created_at, updated_at
		FROM api_keys
		WHERE tenant_id = $1
		ORDER BY created_at DESC
		"#,
		context.tenant_id
	)
	.fetch_all(&state.pool)
	.await
	.map_err(|_| ApiError::Internal)?;

	let api_keys: Vec<serde_json::Value> = keys
		.into_iter()
		.map(|row| {
			serde_json::json!({
				"id": row.id,
				"name": row.name,
				"permissions": row.permissions,
				"last_used_at": row.last_used_at,
				"expires_at": row.expires_at,
				"is_active": row.is_active,
				"created_at": row.created_at,
				"updated_at": row.updated_at,
			})
		})
		.collect();

	Ok(Json(ApiResponse {
		data: api_keys,
		meta: None,
	}))
}

pub async fn revoke_api_key<M, N, T, TR, A>(
	State(state): State<super::routes::AppState<M, N, T, TR, A>>,
	Path((_tenant_slug, key_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, ApiError>
where
	M: crate::services::MonitorServiceTrait,
	N: crate::services::NetworkServiceTrait,
	T: crate::services::TriggerServiceTrait,
	TR: crate::repositories::TenantRepositoryTrait,
	A: crate::services::AuditServiceTrait,
{
	let context = crate::utils::current_tenant_context();

	// Verify user can manage tenant
	if !context.can_manage() {
		return Err(ApiError::Service(ServiceError::AccessDenied(
			"Insufficient permissions to revoke API keys".to_string(),
		)));
	}

	// Revoke API key
	let result = sqlx::query!(
		"UPDATE api_keys SET is_active = false WHERE tenant_id = $1 AND id = $2",
		context.tenant_id,
		key_id
	)
	.execute(&state.pool)
	.await
	.map_err(|_| ApiError::Internal)?;

	if result.rows_affected() == 0 {
		return Err(ApiError::NotFound);
	}

	Ok(StatusCode::NO_CONTENT)
}
