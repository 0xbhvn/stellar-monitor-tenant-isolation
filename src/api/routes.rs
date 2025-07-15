use axum::{
	routing::{delete, get, post, put},
	Router,
};
use tower::ServiceBuilder;
use tower_http::{
	cors::{Any, CorsLayer},
	trace::TraceLayer,
};

use super::auth;
use super::handlers;
use crate::repositories::*;
use crate::services::*;

#[derive(Clone)]
pub struct AppState<M, N, T, TR, A>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	pub monitor_service: M,
	pub network_service: N,
	pub trigger_service: T,
	pub tenant_repo: TR,
	pub audit_service: A,
	pub pool: sqlx::PgPool,
	pub auth_service: crate::utils::AuthService,
}

pub fn create_router<M, N, T, TR, A>(state: AppState<M, N, T, TR, A>) -> Router
where
	M: MonitorServiceTrait + Clone + Send + Sync + 'static,
	N: NetworkServiceTrait + Clone + Send + Sync + 'static,
	T: TriggerServiceTrait + Clone + Send + Sync + 'static,
	TR: TenantRepositoryTrait + Clone + Send + Sync + 'static,
	A: AuditServiceTrait + Clone + Send + Sync + 'static,
{
	// Public routes (no auth required)
	let public_routes = Router::new()
		.route("/health", get(handlers::health_check))
		.route(
			"/api/v1/auth/register",
			post(auth::register::<M, N, T, TR, A>),
		)
		.route("/api/v1/auth/login", post(auth::login::<M, N, T, TR, A>));

	// Tenant-scoped routes (require auth and tenant context)
	let tenant_routes = Router::new()
		// Monitor routes
		.route("/monitors", post(handlers::create_monitor))
		.route("/monitors", get(handlers::list_monitors))
		.route("/monitors/:monitor_id", get(handlers::get_monitor))
		.route("/monitors/:monitor_id", put(handlers::update_monitor))
		.route("/monitors/:monitor_id", delete(handlers::delete_monitor))
		// Network routes
		.route("/networks", post(handlers::create_network))
		.route("/networks", get(handlers::list_networks))
		.route("/networks/:network_id", get(handlers::get_network))
		.route("/networks/:network_id", put(handlers::update_network))
		.route("/networks/:network_id", delete(handlers::delete_network))
		// Trigger routes
		.route("/triggers", post(handlers::create_trigger))
		.route("/triggers", get(handlers::list_triggers))
		.route("/triggers/:trigger_id", get(handlers::get_trigger))
		.route("/triggers/:trigger_id", put(handlers::update_trigger))
		.route("/triggers/:trigger_id", delete(handlers::delete_trigger))
		.route("/monitors/:monitor_id/triggers", get(handlers::list_triggers_by_monitor))
		// API key routes
		.route("/api-keys", post(auth::create_api_key))
		.route("/api-keys", get(auth::list_api_keys))
		.route("/api-keys/:key_id", delete(auth::revoke_api_key));
	// Apply tenant middleware
	// TODO: Fix middleware compilation issue
	// .layer(middleware::from_fn_with_state(
	// 	(state.pool.clone(), state.auth_service.clone()),
	// 	api_middleware::tenant_auth_middleware::<axum::body::Body>,
	// ));

	// Combine all routes
	Router::new()
		.merge(public_routes)
		.nest("/api/v1/tenants/:tenant_slug", tenant_routes)
		.layer(
			ServiceBuilder::new()
				.layer(TraceLayer::new_for_http())
				.layer(
					CorsLayer::new()
						.allow_origin(Any)
						.allow_methods(Any)
						.allow_headers(Any),
				),
		)
		.with_state(state)
}

// Helper function to create app state
impl<M, N, T, TR, A> AppState<M, N, T, TR, A>
where
	M: MonitorServiceTrait,
	N: NetworkServiceTrait,
	T: TriggerServiceTrait,
	TR: TenantRepositoryTrait,
	A: AuditServiceTrait,
{
	pub fn new(
		monitor_service: M,
		network_service: N,
		trigger_service: T,
		tenant_repo: TR,
		audit_service: A,
		pool: sqlx::PgPool,
		auth_service: crate::utils::AuthService,
	) -> Self {
		Self {
			monitor_service,
			network_service,
			trigger_service,
			tenant_repo,
			audit_service,
			pool,
			auth_service,
		}
	}
}
