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

// Helper to create a test tenant with API key
async fn create_test_tenant(pool: &PgPool) -> (Uuid, String) {
	let tenant_id = Uuid::new_v4();
	let api_key = format!("test-api-key-{}", Uuid::new_v4());

	sqlx::query!(
		r#"
        INSERT INTO tenants (id, name, slug, is_active, max_monitors, max_networks, 
                           max_triggers_per_monitor, max_rpc_requests_per_minute, max_storage_mb)
        VALUES ($1, $2, $3, true, 10, 5, 5, 100, 1000)
        "#,
		tenant_id,
		"Trigger Test Tenant",
		format!("trigger-test-{}", tenant_id)
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
		&api_key,
		"Test API Key"
	)
	.execute(pool)
	.await
	.unwrap();

	(tenant_id, api_key)
}

// Helper to create test network and monitor
async fn create_test_monitor(pool: &PgPool, tenant_id: Uuid) -> (Uuid, Uuid, String) {
	let network_id = Uuid::new_v4();
	let monitor_id = Uuid::new_v4();
	let monitor_external_id = format!("monitor-{}", Uuid::new_v4());

	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        network_id,
        tenant_id,
        "test-network",
        "Test Network",
        "stellar",
        serde_json::json!({"rpc_url": "https://test.stellar.org"})
    )
    .execute(pool)
    .await
    .unwrap();

	sqlx::query!(
        r#"
        INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        monitor_id,
        tenant_id,
        &monitor_external_id,
        "Test Monitor",
        network_id,
        serde_json::json!({"type": "contract_event"})
    )
    .execute(pool)
    .await
    .unwrap();

	(network_id, monitor_id, monitor_external_id)
}

#[sqlx::test]
async fn test_create_trigger_success(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;
	let app = test_app(pool).await;

	let request_body = CreateTriggerRequest {
		trigger_id: "trigger-123".to_string(),
		monitor_id,
		name: "Webhook Alert".to_string(),
		trigger_type: "webhook".to_string(),
		configuration: serde_json::json!({
			"url": "https://webhook.site/test",
			"method": "POST",
			"headers": {
				"Authorization": "Bearer test-token"
			}
		}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/triggers")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::CREATED);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let trigger: TenantTrigger = serde_json::from_slice(&body).unwrap();

	assert_eq!(trigger.name, "Webhook Alert");
	assert_eq!(trigger.trigger_id, "trigger-123");
	assert_eq!(trigger.trigger_type, "webhook");
	assert_eq!(trigger.tenant_id, tenant_id);
}

#[sqlx::test]
async fn test_create_trigger_quota_exceeded(pool: PgPool) {
	// Setup - create tenant with low trigger quota
	let tenant_id = Uuid::new_v4();
	let api_key = "test-api-key-trigger-quota";

	sqlx::query!(
		r#"
        INSERT INTO tenants (id, name, slug, is_active, max_monitors, max_networks, 
                           max_triggers_per_monitor, max_rpc_requests_per_minute, max_storage_mb)
        VALUES ($1, $2, $3, true, 10, 5, 1, 100, 1000)
        "#,
		tenant_id,
		"Trigger Quota Tenant",
		"trigger-quota-test"
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

	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;

	// Create first trigger to reach quota
	sqlx::query!(
        r#"
        INSERT INTO tenant_triggers (id, tenant_id, trigger_id, monitor_id, name, type, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        "#,
        Uuid::new_v4(),
        tenant_id,
        "existing-trigger",
        monitor_id,
        "Existing Trigger",
        "webhook",
        serde_json::json!({"url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let request_body = CreateTriggerRequest {
		trigger_id: "new-trigger".to_string(),
		monitor_id,
		name: "New Trigger".to_string(),
		trigger_type: "webhook".to_string(),
		configuration: serde_json::json!({"url": "test"}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/triggers")
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
async fn test_get_trigger_by_id(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;

	let trigger_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_triggers (id, tenant_id, trigger_id, monitor_id, name, type, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        "#,
        trigger_id,
        tenant_id,
        "get-test-trigger",
        monitor_id,
        "Get Test Trigger",
        "email",
        serde_json::json!({"to": "test@example.com"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/triggers/get-test-trigger")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let trigger: TenantTrigger = serde_json::from_slice(&body).unwrap();

	assert_eq!(trigger.trigger_id, "get-test-trigger");
	assert_eq!(trigger.name, "Get Test Trigger");
	assert_eq!(trigger.trigger_type, "email");
}

#[sqlx::test]
async fn test_update_trigger(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;

	let trigger_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_triggers (id, tenant_id, trigger_id, monitor_id, name, type, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        "#,
        trigger_id,
        tenant_id,
        "update-test-trigger",
        monitor_id,
        "Original Name",
        "webhook",
        serde_json::json!({"url": "https://old.webhook"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let update_request = UpdateTriggerRequest {
		name: Some("Updated Trigger Name".to_string()),
		configuration: Some(serde_json::json!({"url": "https://new.webhook"})),
		is_active: Some(false),
	};

	let request = Request::builder()
		.method("PUT")
		.uri("/api/v1/triggers/update-test-trigger")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&update_request).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let trigger: TenantTrigger = serde_json::from_slice(&body).unwrap();

	assert_eq!(trigger.name, "Updated Trigger Name");
	assert_eq!(trigger.is_active, Some(false));
}

#[sqlx::test]
async fn test_delete_trigger(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;

	let trigger_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_triggers (id, tenant_id, trigger_id, monitor_id, name, type, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        "#,
        trigger_id,
        tenant_id,
        "delete-test-trigger",
        monitor_id,
        "To Delete",
        "webhook",
        serde_json::json!({"url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool.clone()).await;

	let request = Request::builder()
		.method("DELETE")
		.uri("/api/v1/triggers/delete-test-trigger")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::NO_CONTENT);

	// Verify trigger is deleted
	let result = sqlx::query!(
		"SELECT id FROM tenant_triggers WHERE trigger_id = $1",
		"delete-test-trigger"
	)
	.fetch_optional(&pool)
	.await
	.unwrap();

	assert!(result.is_none());
}

#[sqlx::test]
async fn test_list_triggers_by_monitor(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, monitor_external_id) = create_test_monitor(&pool, tenant_id).await;

	// Create multiple triggers for the monitor
	for i in 0..3 {
		sqlx::query!(
            r#"
            INSERT INTO tenant_triggers (id, tenant_id, trigger_id, monitor_id, name, type, configuration, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            "#,
            Uuid::new_v4(),
            tenant_id,
            format!("trigger-{}", i),
            monitor_id,
            format!("Trigger {}", i),
            "webhook",
            serde_json::json!({"url": format!("https://webhook{}.test", i)})
        )
        .execute(&pool)
        .await
        .unwrap();
	}

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri(&format!(
			"/api/v1/monitors/{}/triggers",
			monitor_external_id
		))
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let triggers: Vec<TenantTrigger> = serde_json::from_slice(&body).unwrap();

	assert_eq!(triggers.len(), 3);
	assert!(triggers.iter().all(|t| t.monitor_id == monitor_id));
}

#[sqlx::test]
async fn test_trigger_types(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;
	let app = test_app(pool.clone()).await;

	// Test different trigger types
	let trigger_types = vec![
		(
			"webhook",
			serde_json::json!({"url": "https://webhook.test"}),
		),
		(
			"email",
			serde_json::json!({"to": "test@example.com", "subject": "Alert"}),
		),
		(
			"slack",
			serde_json::json!({"webhook_url": "https://hooks.slack.com/test", "channel": "#alerts"}),
		),
	];

	for (trigger_type, config) in trigger_types {
		let request_body = CreateTriggerRequest {
			trigger_id: format!("trigger-{}", trigger_type),
			monitor_id,
			name: format!("{} Trigger", trigger_type),
			trigger_type: trigger_type.to_string(),
			configuration: config,
		};

		let request = Request::builder()
			.method("POST")
			.uri("/api/v1/triggers")
			.header("content-type", "application/json")
			.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
			.body(Body::from(serde_json::to_string(&request_body).unwrap()))
			.unwrap();

		let response = app.clone().oneshot(request).await.unwrap();
		assert_eq!(response.status(), StatusCode::CREATED);
	}
}

#[sqlx::test]
async fn test_trigger_isolation_between_tenants(pool: PgPool) {
	// Setup - create two tenants
	let (tenant1_id, _) = create_test_tenant(&pool).await;
	let (_tenant2_id, tenant2_api_key) = create_test_tenant(&pool).await;

	let (_, monitor1_id, _) = create_test_monitor(&pool, tenant1_id).await;

	// Create trigger for tenant1
	sqlx::query!(
        r#"
        INSERT INTO tenant_triggers (id, tenant_id, trigger_id, monitor_id, name, type, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        "#,
        Uuid::new_v4(),
        tenant1_id,
        "tenant1-trigger",
        monitor1_id,
        "Tenant 1 Trigger",
        "webhook",
        serde_json::json!({"url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	// Try to access tenant1's trigger with tenant2's API key
	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/triggers/tenant1-trigger")
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

#[sqlx::test]
async fn test_invalid_trigger_type(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let (_, monitor_id, _) = create_test_monitor(&pool, tenant_id).await;
	let app = test_app(pool).await;

	let request_body = CreateTriggerRequest {
		trigger_id: "invalid-trigger".to_string(),
		monitor_id,
		name: "Invalid Trigger".to_string(),
		trigger_type: "invalid_type".to_string(), // Invalid trigger type
		configuration: serde_json::json!({}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/triggers")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
