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
		"Network Test Tenant",
		format!("network-test-{}", tenant_id)
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

#[sqlx::test]
async fn test_create_network_success(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;
	let app = test_app(pool).await;

	let request_body = CreateNetworkRequest {
		network_id: "stellar-mainnet".to_string(),
		name: "Stellar Mainnet".to_string(),
		blockchain: "stellar".to_string(),
		configuration: serde_json::json!({
			"rpc_url": "https://horizon.stellar.org",
			"network_passphrase": "Public Global Stellar Network ; September 2015"
		}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/networks")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::CREATED);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let network: TenantNetwork = serde_json::from_slice(&body).unwrap();

	assert_eq!(network.name, "Stellar Mainnet");
	assert_eq!(network.network_id, "stellar-mainnet");
	assert_eq!(network.blockchain, "stellar");
	assert_eq!(network.tenant_id, tenant_id);
}

#[sqlx::test]
async fn test_create_network_quota_exceeded(pool: PgPool) {
	// Setup - create tenant with max networks already reached
	let tenant_id = Uuid::new_v4();
	let api_key = "test-api-key-network-quota";

	sqlx::query!(
		r#"
        INSERT INTO tenants (id, name, slug, is_active, max_monitors, max_networks, 
                           max_triggers_per_monitor, max_rpc_requests_per_minute, max_storage_mb)
        VALUES ($1, $2, $3, true, 10, 1, 5, 100, 1000)
        "#,
		tenant_id,
		"Network Quota Tenant",
		"network-quota-test"
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

	// Create first network to reach quota
	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        Uuid::new_v4(),
        tenant_id,
        "existing-network",
        "Existing Network",
        "stellar",
        serde_json::json!({"rpc_url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let request_body = CreateNetworkRequest {
		network_id: "new-network".to_string(),
		name: "New Network".to_string(),
		blockchain: "stellar".to_string(),
		configuration: serde_json::json!({"rpc_url": "test"}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/networks")
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
async fn test_get_network_by_id(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;

	let network_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        network_id,
        tenant_id,
        "get-test-network",
        "Get Test Network",
        "ethereum",
        serde_json::json!({"rpc_url": "https://eth.test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/networks/get-test-network")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let network: TenantNetwork = serde_json::from_slice(&body).unwrap();

	assert_eq!(network.network_id, "get-test-network");
	assert_eq!(network.name, "Get Test Network");
}

#[sqlx::test]
async fn test_update_network(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;

	let network_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        network_id,
        tenant_id,
        "update-test-network",
        "Original Name",
        "stellar",
        serde_json::json!({"rpc_url": "https://old.url"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let update_request = UpdateNetworkRequest {
		name: Some("Updated Network Name".to_string()),
		configuration: Some(serde_json::json!({"rpc_url": "https://new.url"})),
		is_active: Some(false),
	};

	let request = Request::builder()
		.method("PUT")
		.uri("/api/v1/networks/update-test-network")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&update_request).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let network: TenantNetwork = serde_json::from_slice(&body).unwrap();

	assert_eq!(network.name, "Updated Network Name");
	assert_eq!(network.is_active, Some(false));
}

#[sqlx::test]
async fn test_delete_network(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;

	let network_id = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        network_id,
        tenant_id,
        "delete-test-network",
        "To Delete",
        "stellar",
        serde_json::json!({"rpc_url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool.clone()).await;

	let request = Request::builder()
		.method("DELETE")
		.uri("/api/v1/networks/delete-test-network")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::NO_CONTENT);

	// Verify network is deleted
	let result = sqlx::query!(
		"SELECT id FROM tenant_networks WHERE network_id = $1",
		"delete-test-network"
	)
	.fetch_optional(&pool)
	.await
	.unwrap();

	assert!(result.is_none());
}

#[sqlx::test]
async fn test_delete_network_with_monitors(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;

	// Create network
	let network_uuid = Uuid::new_v4();
	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        network_uuid,
        tenant_id,
        "network-with-monitors",
        "Network with Monitors",
        "stellar",
        serde_json::json!({"rpc_url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	// Create monitor using this network
	sqlx::query!(
        r#"
        INSERT INTO tenant_monitors (id, tenant_id, monitor_id, name, network_id, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        Uuid::new_v4(),
        tenant_id,
        "test-monitor",
        "Test Monitor",
        network_uuid,
        serde_json::json!({"type": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("DELETE")
		.uri("/api/v1/networks/network-with-monitors")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert - should fail due to relationship
	assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn test_list_networks(pool: PgPool) {
	// Setup
	let (tenant_id, api_key) = create_test_tenant(&pool).await;

	// Create multiple networks
	for i in 0..3 {
		sqlx::query!(
            r#"
            INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, true)
            "#,
            Uuid::new_v4(),
            tenant_id,
            format!("network-{}", i),
            format!("Network {}", i),
            "stellar",
            serde_json::json!({"rpc_url": format!("https://network{}.test", i)})
        )
        .execute(&pool)
        .await
        .unwrap();
	}

	let app = test_app(pool).await;

	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/networks?limit=10&offset=0")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::empty())
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::OK);

	let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
	let networks: Vec<TenantNetwork> = serde_json::from_slice(&body).unwrap();

	assert_eq!(networks.len(), 3);
}

#[sqlx::test]
async fn test_network_isolation_between_tenants(pool: PgPool) {
	// Setup - create two tenants
	let (tenant1_id, _) = create_test_tenant(&pool).await;
	let (_tenant2_id, tenant2_api_key) = create_test_tenant(&pool).await;

	// Create network for tenant1
	sqlx::query!(
        r#"
        INSERT INTO tenant_networks (id, tenant_id, network_id, name, blockchain, configuration, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        "#,
        Uuid::new_v4(),
        tenant1_id,
        "tenant1-network",
        "Tenant 1 Network",
        "stellar",
        serde_json::json!({"rpc_url": "test"})
    )
    .execute(&pool)
    .await
    .unwrap();

	let app = test_app(pool).await;

	// Try to access tenant1's network with tenant2's API key
	let request = Request::builder()
		.method("GET")
		.uri("/api/v1/networks/tenant1-network")
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
async fn test_invalid_network_configuration(pool: PgPool) {
	// Setup
	let (_, api_key) = create_test_tenant(&pool).await;
	let app = test_app(pool).await;

	let request_body = CreateNetworkRequest {
		network_id: "".to_string(), // Invalid empty network_id
		name: "Invalid Network".to_string(),
		blockchain: "stellar".to_string(),
		configuration: serde_json::json!({}),
	};

	let request = Request::builder()
		.method("POST")
		.uri("/api/v1/networks")
		.header("content-type", "application/json")
		.header("x-api-key", HeaderValue::from_str(&api_key).unwrap())
		.body(Body::from(serde_json::to_string(&request_body).unwrap()))
		.unwrap();

	// Act
	let response = app.oneshot(request).await.unwrap();

	// Assert
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
