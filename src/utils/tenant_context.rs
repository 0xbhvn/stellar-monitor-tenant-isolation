use std::sync::Arc;
use tokio::task_local;
use uuid::Uuid;

use crate::models::TenantRole;

// Task-local storage for tenant context
task_local! {
	static TENANT_CONTEXT: Arc<TenantContext>;
}

#[derive(Debug, Clone)]
pub struct TenantContext {
	pub tenant_id: Uuid,
	pub user: Option<AuthenticatedUser>,
	pub api_key_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
	pub id: Uuid,
	pub email: String,
	pub role: TenantRole,
}

impl TenantContext {
	pub fn new(tenant_id: Uuid) -> Self {
		Self {
			tenant_id,
			user: None,
			api_key_id: None,
		}
	}

	pub fn with_user(tenant_id: Uuid, user: AuthenticatedUser) -> Self {
		Self {
			tenant_id,
			user: Some(user),
			api_key_id: None,
		}
	}

	pub fn with_api_key(tenant_id: Uuid, api_key_id: Uuid) -> Self {
		Self {
			tenant_id,
			user: None,
			api_key_id: Some(api_key_id),
		}
	}

	pub fn can_write(&self) -> bool {
		self.user
			.as_ref()
			.map(|u| u.role.can_write())
			.unwrap_or(true) // API keys can write by default
	}

	pub fn can_manage(&self) -> bool {
		self.user
			.as_ref()
			.map(|u| u.role.can_manage_tenant())
			.unwrap_or(false)
	}
}

// Execute a future with a tenant context
pub async fn with_tenant_context<F, R>(context: TenantContext, f: F) -> R
where
	F: std::future::Future<Output = R>,
{
	let context = Arc::new(context);
	TENANT_CONTEXT.scope(context, f).await
}

// Get the current tenant context
pub fn current_tenant_context() -> Arc<TenantContext> {
	TENANT_CONTEXT
		.try_with(|ctx| ctx.clone())
		.expect("No tenant context set")
}

// Get the current tenant ID
pub fn current_tenant_id() -> Uuid {
	current_tenant_context().tenant_id
}

// Check if we have write permissions in the current context
pub fn can_write() -> bool {
	current_tenant_context().can_write()
}

// Check if we have management permissions in the current context
pub fn can_manage() -> bool {
	current_tenant_context().can_manage()
}

// Middleware helper for extracting tenant context from requests
pub mod middleware {
	use super::*;
	use axum::{
		extract::Path,
		http::{Request, StatusCode},
		middleware::Next,
		response::Response,
	};
	use axum_extra::{
		extract::TypedHeader,
		headers::{authorization::Bearer, Authorization},
	};

	pub async fn tenant_context_middleware(
		Path(_tenant_slug): Path<String>,
		TypedHeader(_auth): TypedHeader<Authorization<Bearer>>,
		mut req: Request<axum::body::Body>,
		next: Next,
	) -> Result<Response, StatusCode> {
		// TODO: Validate JWT token and extract user info
		// TODO: Look up tenant by slug
		// TODO: Verify user has access to tenant

		// For now, create a dummy context
		let context = TenantContext::new(Uuid::new_v4());

		// Store context in request extensions
		req.extensions_mut().insert(Arc::new(context.clone()));

		// Execute the request with the tenant context
		let response = with_tenant_context(context, next.run(req)).await;

		Ok(response)
	}
}
