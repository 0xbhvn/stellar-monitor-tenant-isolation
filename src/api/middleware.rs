use axum::{
	extract::{Path, State},
	http::{Request, StatusCode},
	middleware::Next,
	response::Response,
};
use axum_extra::{
	extract::TypedHeader,
	headers::{authorization::Bearer, Authorization},
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::repositories::TenantRepositoryTrait;
use crate::utils::{with_tenant_context, AuthService, AuthenticatedUser, TenantContext};

pub async fn tenant_auth_middleware(
	Path(tenant_slug): Path<String>,
	TypedHeader(auth_header): TypedHeader<Authorization<Bearer>>,
	State((pool, auth_service)): State<(sqlx::PgPool, AuthService)>,
	mut req: Request<axum::body::Body>,
	next: Next,
) -> Result<Response, StatusCode> {
	let token = auth_header.token();

	// Check if it's an API key or JWT
	let context = if token.starts_with(&crate::config::Config::default().auth.api_key_prefix) {
		// Handle API key authentication
		authenticate_api_key(&pool, &tenant_slug, token).await?
	} else {
		// Handle JWT authentication
		let tenant_repo = crate::repositories::TenantRepository::new(pool.clone());
		authenticate_jwt(&auth_service, &tenant_repo, &tenant_slug, token).await?
	};

	// Store context in request extensions
	req.extensions_mut().insert(Arc::new(context.clone()));

	// Execute the request with the tenant context
	let response = with_tenant_context(context, next.run(req)).await;

	Ok(response)
}

async fn authenticate_jwt<T>(
	auth_service: &AuthService,
	tenant_repo: &T,
	tenant_slug: &str,
	token: &str,
) -> Result<TenantContext, StatusCode>
where
	T: TenantRepositoryTrait,
{
	// Verify JWT
	let claims = auth_service
		.verify_jwt(token)
		.map_err(|_| StatusCode::UNAUTHORIZED)?;

	// Get tenant
	let tenant = tenant_repo
		.get_by_slug(tenant_slug)
		.await
		.map_err(|_| StatusCode::NOT_FOUND)?;

	// Get user's role in this tenant
	let memberships = tenant_repo
		.get_user_tenants(claims.sub)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let membership = memberships
		.iter()
		.find(|(t, _)| t.id == tenant.id)
		.ok_or(StatusCode::FORBIDDEN)?;

	let user = AuthenticatedUser {
		id: claims.sub,
		email: claims.email,
		role: membership.1.clone(),
	};

	Ok(TenantContext::with_user(tenant.id, user))
}

async fn authenticate_api_key(
	pool: &Pool<Postgres>,
	tenant_slug: &str,
	api_key: &str,
) -> Result<TenantContext, StatusCode> {
	// Remove prefix
	let key_without_prefix = api_key
		.strip_prefix(&crate::config::Config::default().auth.api_key_prefix)
		.ok_or(StatusCode::UNAUTHORIZED)?;

	// Hash the key to compare with stored hash
	let auth_service = AuthService::new(crate::config::Config::default().auth.jwt_secret);

	// Look up API key
	let key_record = sqlx::query!(
		r#"
		SELECT 
			ak.id, ak.tenant_id, ak.key_hash, ak.is_active, ak.expires_at,
			t.slug as tenant_slug
		FROM api_keys ak
		INNER JOIN tenants t ON ak.tenant_id = t.id
		WHERE t.slug = $1 AND ak.is_active = true
		"#,
		tenant_slug
	)
	.fetch_all(pool)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	// Find matching key by verifying hash
	let valid_key = key_record
		.into_iter()
		.find(|record| {
			auth_service
				.verify_password(key_without_prefix, &record.key_hash)
				.unwrap_or(false)
		})
		.ok_or(StatusCode::UNAUTHORIZED)?;

	// Check expiration
	if let Some(expires_at) = valid_key.expires_at {
		if expires_at < chrono::Utc::now() {
			return Err(StatusCode::UNAUTHORIZED);
		}
	}

	// Update last used timestamp
	sqlx::query!(
		"UPDATE api_keys SET last_used_at = NOW() WHERE id = $1",
		valid_key.id
	)
	.execute(pool)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	Ok(TenantContext::with_api_key(
		valid_key.tenant_id,
		valid_key.id,
	))
}

// Optional: Rate limiting middleware
pub async fn rate_limit_middleware(
	req: Request<axum::body::Body>,
	next: Next,
) -> Result<Response, StatusCode> {
	// TODO: Implement rate limiting based on tenant quotas
	Ok(next.run(req).await)
}

// Optional: Request logging middleware
pub async fn request_logging_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
	let method = req.method().clone();
	let uri = req.uri().clone();
	let start = std::time::Instant::now();

	let response = next.run(req).await;

	let duration = start.elapsed();
	let status = response.status();

	tracing::info!(
		method = %method,
		uri = %uri,
		status = %status,
		duration_ms = %duration.as_millis(),
		"Request completed"
	);

	response
}
