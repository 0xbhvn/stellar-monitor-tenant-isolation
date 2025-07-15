use std::sync::Arc;
use tokio::task_local;
use uuid::Uuid;

use crate::models::{TenantQuotas, TenantRole};

// Task-local storage for tenant context
task_local! {
	static TENANT_CONTEXT: Arc<TenantContext>;
}

#[derive(Debug, Clone)]
pub struct TenantContext {
	pub tenant_id: Uuid,
	pub user: Option<AuthenticatedUser>,
	pub api_key_id: Option<Uuid>,
	pub quotas: TenantQuotas,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
	pub id: Uuid,
	pub email: String,
	pub role: TenantRole,
}

impl TenantContext {
	pub fn new(tenant_id: Uuid, quotas: TenantQuotas) -> Self {
		Self {
			tenant_id,
			user: None,
			api_key_id: None,
			quotas,
		}
	}

	pub fn with_user(tenant_id: Uuid, user: AuthenticatedUser, quotas: TenantQuotas) -> Self {
		Self {
			tenant_id,
			user: Some(user),
			api_key_id: None,
			quotas,
		}
	}

	pub fn with_api_key(tenant_id: Uuid, api_key_id: Uuid, quotas: TenantQuotas) -> Self {
		Self {
			tenant_id,
			user: None,
			api_key_id: Some(api_key_id),
			quotas,
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

// Get the current tenant context as an Option
pub fn current_tenant_context_option() -> Option<Arc<TenantContext>> {
	TENANT_CONTEXT.try_with(|ctx| ctx.clone()).ok()
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
