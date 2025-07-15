use axum::{
	body::{to_bytes, Body},
	http::{Request, StatusCode},
};
use sqlx::PgPool;
use stellar_monitor_tenant_isolation::{
	api::{create_router, AppState},
	models::*,
	repositories::*,
	services::*,
	utils::AuthService,
};
use tower::ServiceExt;

// Helper function to create test app
async fn test_app(pool: PgPool) -> axum::Router {
	// Initialize repositories
	let tenant_repo = TenantRepository::new(pool.clone());
	let monitor_repo = TenantMonitorRepository::new(pool.clone());
	let network_repo = TenantNetworkRepository::new(pool.clone());
	let trigger_repo = TenantTriggerRepository::new(pool.clone());

	// Initialize services
	let auth_service = AuthService::new("test-secret".to_string());
	let audit_service = AuditService::new(pool.clone());

	let monitor_service = MonitorService::new(
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let network_service = NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let trigger_service = TriggerService::new(
		trigger_repo.clone(),
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create app state
	let app_state = AppState::new(
		monitor_service,
		network_service,
		trigger_service,
		tenant_repo,
		audit_service,
		pool.clone(),
		auth_service,
	);

	// Create router
	create_router(app_state)
}

#[sqlx::test]
async fn test_create_tenant_success(pool: PgPool) {
	// Setup
	let app = test_app(pool).await;

	let request_body = CreateTenantRequest {
		name: "Test Company".to_string(),
		slug: "test-company".to_string(),
		max_monitors: Some(10),
		max_networks: Some(5),
		max_triggers_per_monitor: Some(5),
		max_rpc_requests_per_minute: Some(100),
		max_storage_mb: Some(1000),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::CREATED);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let tenant: Tenant = serde_json::from_slice(&body).unwrap();

	assert_eq!(tenant.name, "Test Company");
	assert_eq!(tenant.slug, "test-company");
}

#[sqlx::test]
async fn test_create_tenant_duplicate_slug(pool: PgPool) {
	// Setup - create first tenant
	let app = test_app(pool.clone()).await;

	let request_body = CreateTenantRequest {
		name: "First Company".to_string(),
		slug: "test-company".to_string(),
		max_monitors: Some(10),
		max_networks: Some(5),
		max_triggers_per_monitor: Some(5),
		max_rpc_requests_per_minute: Some(100),
		max_storage_mb: Some(1000),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	let _ = app.oneshot(request).await.unwrap();

	// Try to create second tenant with same slug
	let app = test_app(pool).await;

	let duplicate_request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(duplicate_request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn test_get_tenant_by_id(pool: PgPool) {
	// Setup - create tenant first
	let app = test_app(pool.clone()).await;

	let create_request = CreateTenantRequest {
		name: "Test Company".to_string(),
		slug: "test-company".to_string(),
		max_monitors: Some(10),
		max_networks: Some(5),
		max_triggers_per_monitor: Some(5),
		max_rpc_requests_per_minute: Some(100),
		max_storage_mb: Some(1000),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&create_request).unwrap()))
		.unwrap();

	let create_response = app.oneshot(request).await.unwrap();
	let body = to_bytes(create_response.into_body(), 1024 * 1024)
		.await
		.unwrap();
	let created_tenant: Tenant = serde_json::from_slice(&body).unwrap();

	// Get tenant by ID
	let app = test_app(pool).await;

	let get_request = Request::builder()
		.method("GET")
		.uri(&format!("/api/v1/tenants/{}", created_tenant.id))
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(get_request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let tenant: Tenant = serde_json::from_slice(&body).unwrap();

	assert_eq!(tenant.id, created_tenant.id);
	assert_eq!(tenant.name, "Test Company");
}

#[sqlx::test]
async fn test_get_tenant_not_found(pool: PgPool) {
	// Setup
	let app = test_app(pool).await;
	let non_existent_id = uuid::Uuid::new_v4();

	let request = Request::builder()
		.method("GET")
		.uri(&format!("/api/v1/tenants/{}", non_existent_id))
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn test_update_tenant(pool: PgPool) {
	// Setup - create tenant first
	let app = test_app(pool.clone()).await;

	let create_request = CreateTenantRequest {
		name: "Old Name".to_string(),
		slug: "old-slug".to_string(),
		max_monitors: Some(10),
		max_networks: Some(5),
		max_triggers_per_monitor: Some(5),
		max_rpc_requests_per_minute: Some(100),
		max_storage_mb: Some(1000),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&create_request).unwrap()))
		.unwrap();

	let create_response = app.oneshot(request).await.unwrap();
	let body = to_bytes(create_response.into_body(), 1024 * 1024)
		.await
		.unwrap();
	let created_tenant: Tenant = serde_json::from_slice(&body).unwrap();

	// Update tenant
	let app = test_app(pool).await;

	let update_request = UpdateTenantRequest {
		name: Some("New Name".to_string()),
		is_active: Some(false),
		max_monitors: Some(20),
		max_networks: None,
		max_triggers_per_monitor: None,
		max_rpc_requests_per_minute: None,
		max_storage_mb: None,
	};

	let request = Request::builder()
		.method("PUT")
		.uri(&format!("/api/v1/tenants/{}", created_tenant.id))
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&update_request).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let tenant: Tenant = serde_json::from_slice(&body).unwrap();

	assert_eq!(tenant.name, "New Name");
	assert_eq!(tenant.is_active, false);
	assert_eq!(tenant.max_monitors, 20);
}

#[sqlx::test]
async fn test_delete_tenant(pool: PgPool) {
	// Setup - create tenant first
	let app = test_app(pool.clone()).await;

	let create_request = CreateTenantRequest {
		name: "To Delete".to_string(),
		slug: "to-delete".to_string(),
		max_monitors: Some(10),
		max_networks: Some(5),
		max_triggers_per_monitor: Some(5),
		max_rpc_requests_per_minute: Some(100),
		max_storage_mb: Some(1000),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from(serde_json::to_string(&create_request).unwrap()))
		.unwrap();

	let create_response = app.oneshot(request).await.unwrap();
	let body = to_bytes(create_response.into_body(), 1024 * 1024)
		.await
		.unwrap();
	let created_tenant: Tenant = serde_json::from_slice(&body).unwrap();

	// Delete tenant
	let app = test_app(pool.clone()).await;

	let delete_request = Request::builder()
		.method("DELETE")
		.uri(&format!("/api/v1/tenants/{}", created_tenant.id))
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(delete_request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::NO_CONTENT);

	// Verify tenant is deleted
	let app = test_app(pool).await;

	let get_request = Request::builder()
		.method("GET")
		.uri(&format!("/api/v1/tenants/{}", created_tenant.id))
		.body(Body::empty())
		.unwrap();

	let get_response = app.oneshot(get_request).await.unwrap();
	assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn test_list_tenants_with_pagination(pool: PgPool) {
	// Setup - create multiple tenants
	let app = test_app(pool.clone()).await;

	for i in 0..5 {
		let request_body = CreateTenantRequest {
			name: format!("Company {}", i),
			slug: format!("company-{}", i),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		};

		let request = Request::builder()
			.method("POST")
			.uri("/api/v1/tenants")
			.header("content-type", "application/json")
			.body(Body::from(serde_json::to_string(&request_body).unwrap()))
			.unwrap();

		let _ = app.clone().oneshot(request).await.unwrap();
	}

	// List with pagination
	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/tenants?limit=2&offset=0")
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let tenants: Vec<Tenant> = serde_json::from_slice(&body).unwrap();

	assert_eq!(tenants.len(), 2);
}

#[sqlx::test]
async fn test_invalid_request_body(pool: PgPool) {
	// Setup
	let app = test_app(pool).await;

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/tenants")
		.header("content-type", "application/json")
		.body(Body::from("{invalid json}"))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
