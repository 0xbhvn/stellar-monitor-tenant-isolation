use axum::{
	body::{to_bytes, Body},
	http::{HeaderValue, Request, StatusCode},
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
use uuid::Uuid;

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

// Helper to create a test tenant
async fn create_test_tenant(pool: &PgPool) -> (Uuid, String) {
	let tenant_id = Uuid::new_v4();
	let api_key = "test-api-key-123";

	sqlx::query!(
		r#"
        INSERT INTO tenants (id, name, slug, is_active, max_monitors, max_networks, 
                           max_triggers_per_monitor, max_rpc_requests_per_minute, max_storage_mb)
        VALUES ($1, $2, $3, true, 10, 5, 5, 100, 1000)
        "#,
		tenant_id,
		"Test Tenant",
		"test-tenant"
	)
	.execute(pool)
	.await
	.unwrap();

	sqlx::query!(
		r#"
        INSERT INTO api_keys (id, tenant_id, key_hash, name, is_active)
        VALUES ($1, $2, $3, $4, true)
        "#,
		Uuid::new_v4(),
		tenant_id,
		api_key,
		"Test API Key"
	)
	.execute(pool)
	.await
	.unwrap();

	(tenant_id, api_key.to_string())
}

// Helper to create a test network
async fn create_test_network(pool: &PgPool, tenant_id: Uuid) -> Uuid {
	let network_id = Uuid::new_v4();

	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        network_id,
        tenant_id,
        "stellar-testnet",
        "Stellar Testnet",
        "stellar",
        serde_json::json!({"rpc_url": "https://horizon-testnet.stellar.org"})
    )
    .execute(pool)
    .await
    .unwrap();

	network_id
}

#[sqlx::test]
async fn test_create_monitor_success(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let network_id = create_test_network(&pool, tenant_id).await;
	let app = test_app(pool).await;

	let request_body = CreateMonitorRequest {
		monitor_id: "monitor-123".to_string(),
		name: "Test Monitor".to_string(),
		network_id,
		configuration: serde_json::json!({
			"type": "contract_event",
			"contract_address": "GBVOL67TMUQBGL4TZYNMY3ZQ5WGQYFPFD5VJRWXR72VA33VFNL225PL5",
			"event_name": "Transfer"
		}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/monitors")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::CREATED);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let monitor: TenantMonitor = serde_json::from_slice(&body).unwrap();

	assert_eq!(monitor.name, "Test Monitor");
	assert_eq!(monitor.monitor_id, "monitor-123");
	assert_eq!(monitor.tenant_id, tenant_id);
}

#[sqlx::test]
async fn test_create_monitor_quota_exceeded(pool: PgPool) {
	// Setup - create tenant with low quota
	let tenant_id = Uuid::new_v4();
	let api_key = "test-api-key-quota";

	sqlx::query!(
		r#"
        INSERT INTO tenants (id, name, slug, is_active, max_monitors, max_networks, 
                           max_triggers_per_monitor, max_rpc_requests_per_minute, max_storage_mb)
        VALUES ($1, $2, $3, true, 0, 5, 5, 100, 1000)
        "#,
		tenant_id,
		"Quota Test Tenant",
		"quota-test"
	)
	.execute(&pool)
	.await
	.unwrap();

	sqlx::query!(
		r#"
        INSERT INTO api_keys (id, tenant_id, key_hash, name, is_active)
        VALUES ($1, $2, $3, $4, true)
        "#,
		Uuid::new_v4(),
		tenant_id,
		api_key,
		"Test API Key"
	)
	.execute(&pool)
	.await
	.unwrap();

	let network_id = create_test_network(&pool, tenant_id).await;
	let app = test_app(pool).await;

	let request_body = CreateMonitorRequest {
		monitor_id: "monitor-quota".to_string(),
		name: "Quota Test Monitor".to_string(),
		network_id,
		configuration: serde_json::json!({"type": "test"}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/monitors")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(api_key).unwrap())
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn test_get_monitor_by_id(pool: PgPool) {
	// Setup - create monitor first
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let network_id = create_test_network(&pool, tenant_id).await;

	let monitor_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        monitor_id,
        tenant_id,
        "monitor-get-test",
        "Get Test Monitor",
        network_id,
        serde_json::json!({"type": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri(&format!("/api/v1/monitors/{}", "monitor-get-test"))
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let monitor: TenantMonitor = serde_json::from_slice(&body).unwrap();

	assert_eq!(monitor.monitor_id, "monitor-get-test");
	assert_eq!(monitor.name, "Get Test Monitor");
}

#[sqlx::test]
async fn test_get_monitor_unauthorized(pool: PgPool) {
	// Setup
	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/monitors/some-monitor")
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn test_update_monitor(pool: PgPool) {
	// Setup - create monitor first
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let network_id = create_test_network(&pool, tenant_id).await;

	let monitor_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        monitor_id,
        tenant_id,
        "monitor-update-test",
        "Original Name",
        network_id,
        serde_json::json!({"type": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let update_request = UpdateMonitorRequest {
		name: Some("Updated Name".to_string()),
		configuration: Some(serde_json::json!({"type": "updated"})),
		is_active: Some(false),
	};

	let request = Request::builder()
		.method("PUT")
		.uri(&format!("/api/v1/monitors/{}", "monitor-update-test"))
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&update_request).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let monitor: TenantMonitor = serde_json::from_slice(&body).unwrap();

	assert_eq!(monitor.name, "Updated Name");
	assert_eq!(monitor.is_active, Some(false));
}

#[sqlx::test]
async fn test_delete_monitor(pool: PgPool) {
	// Setup - create monitor first
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let network_id = create_test_network(&pool, tenant_id).await;

	let monitor_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        monitor_id,
        tenant_id,
        "monitor-delete-test",
        "To Delete",
        network_id,
        serde_json::json!({"type": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool.clone()).await;

	let request = Request::builder()
		.method("DELETE")
		.uri(&format!("/api/v1/monitors/{}", "monitor-delete-test"))
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::NO_CONTENT);

	// Verify monitor is deleted
	let result = sqlx::query!(
		"SELECT id FROM tenant_monitors WHERE monitor_id = $1",
		"monitor-delete-test"
	)
	.fetch_optional(&pool)
	.await
	.unwrap();

	assert!(result.is_none());
}

#[sqlx::test]
async fn test_list_monitors_with_pagination(pool: PgPool) {
	// Setup - create multiple monitors
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let network_id = create_test_network(&pool, tenant_id).await;

	for i in 0..5 {
		sqlx::query!(
            r#"
            INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, true)
            "#,
            Uuid::new_v4(),
            tenant_id,
            format!("monitor-list-{}", i),
            format!("Monitor {}", i),
            network_id,
            serde_json::json!({"type": "test"})
        )
        .execute(&pool)
        .await
        .unwrap();
	}

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/monitors?limit=3&offset=0")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let monitors: Vec<TenantMonitor> = serde_json::from_slice(&body).unwrap();

	assert_eq!(monitors.len(), 3);
}

#[sqlx::test]
async fn test_monitor_access_control(pool: PgPool) {
	// Setup - create two tenants with monitors
	let (tenant1_id, _) = create_test_tenant(&pool).await;
	let (_tenant2_id, tenant2_api_key) = create_test_tenant(&pool).await;

	let network_id = create_test_network(&pool, tenant1_id).await;

	// Create monitor for tenant1
	sqlx::query!(
        r#"
        INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        Uuid::new_v4(),
        tenant1_id,
        "tenant1-monitor",
        "Tenant 1 Monitor",
        network_id,
        serde_json::json!({"type": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	// Try to access tenant1's monitor with tenant2's API key
	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/monitors/tenant1-monitor")
		.header(
			"x-api-key",
			HeaderValue::from_str(&tenant2_api_key).unwrap(),
		)
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
