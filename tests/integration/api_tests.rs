#[cfg(test)]
mod tests {
	use axum::http::{Request, StatusCode};
	use axum::body::Body;
	use tower::ServiceExt;
	use stellar_monitor_tenant_isolation::*;
	use sqlx::PgPool;
	
	async fn setup_test_app() -> (axum::Router, PgPool) {
		dotenvy::dotenv().ok();
		
		// Create test database pool
		let database_url = std::env::var("TEST_DATABASE_URL")
			.unwrap_or_else(|_| "postgres://postgres:password@localhost/stellar_monitor_test".to_string());
		
		let pool = sqlx::postgres::PgPoolOptions::new()
			.max_connections(5)
			.connect(&database_url)
			.await
			.expect("Failed to connect to test database");
		
		// Run migrations
		sqlx::migrate!("./migrations")
			.run(&pool)
			.await
			.expect("Failed to run migrations");
		
		// Initialize repositories and services
		let tenant_repo = repositories::TenantRepository::new(pool.clone());
		let monitor_repo = repositories::TenantMonitorRepository::new(pool.clone());
		let network_repo = repositories::TenantNetworkRepository::new(pool.clone());
		let trigger_repo = repositories::TenantTriggerRepository::new(pool.clone());
		
		let auth_service = utils::AuthService::new("test-secret".to_string());
		let audit_service = services::AuditService::new(pool.clone());
		
		let monitor_service = services::MonitorService::new(
			monitor_repo.clone(),
			tenant_repo.clone(),
			audit_service.clone(),
		);
		
		let network_service = services::NetworkService::new(
			network_repo.clone(),
			tenant_repo.clone(),
			audit_service.clone(),
		);
		
		let trigger_service = services::TriggerService::new(
			trigger_repo.clone(),
			monitor_repo.clone(),
			tenant_repo.clone(),
			audit_service.clone(),
		);
		
		// Create app state
		let app_state = api::AppState::new(
			monitor_service,
			network_service,
			trigger_service,
			tenant_repo,
			audit_service,
			pool.clone(),
			auth_service,
		);
		
		// Create router
		let app = api::create_router(app_state);
		
		(app, pool)
	}
	
	#[tokio::test]
	async fn test_health_check() {
		let (app, _pool) = setup_test_app().await;
		
		let response = app
			.oneshot(
				Request::builder()
					.uri("/health")
					.body(Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();
		
		assert_eq!(response.status(), StatusCode::OK);
		
		let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
			.await
			.unwrap();
		let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
		
		assert_eq!(json["status"], "healthy");
		assert!(json["timestamp"].is_string());
	}
	
	#[tokio::test]
	async fn test_user_registration_and_login() {
		let (app, pool) = setup_test_app().await;
		
		// Clean up database
		sqlx::query!("DELETE FROM tenant_memberships").execute(&pool).await.ok();
		sqlx::query!("DELETE FROM users").execute(&pool).await.ok();
		sqlx::query!("DELETE FROM tenants").execute(&pool).await.ok();
		
		// Test registration
		let register_request = serde_json::json!({
			"email": "newuser@example.com",
			"password": "secure-password-123",
			"tenant_name": "Test Company",
			"tenant_slug": "test-company"
		});
		
		let response = app.clone()
			.oneshot(
				Request::builder()
					.method("POST")
					.uri("/api/v1/auth/register")
					.header("content-type", "application/json")
					.body(Body::from(register_request.to_string()))
					.unwrap(),
			)
			.await
			.unwrap();
		
		assert_eq!(response.status(), StatusCode::CREATED);
		
		let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
			.await
			.unwrap();
		let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
		
		assert!(json["data"]["access_token"].is_string());
		assert!(json["data"]["user"]["email"].as_str().unwrap() == "newuser@example.com");
		assert!(json["data"]["tenant"]["slug"].as_str().unwrap() == "test-company");
		
		// Test login
		let login_request = serde_json::json!({
			"email": "newuser@example.com",
			"password": "secure-password-123"
		});
		
		let response = app
			.oneshot(
				Request::builder()
					.method("POST")
					.uri("/api/v1/auth/login")
					.header("content-type", "application/json")
					.body(Body::from(login_request.to_string()))
					.unwrap(),
			)
			.await
			.unwrap();
		
		assert_eq!(response.status(), StatusCode::OK);
		
		let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
			.await
			.unwrap();
		let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
		
		assert!(json["data"]["access_token"].is_string());
		assert_eq!(json["data"]["user"]["email"], "newuser@example.com");
		assert_eq!(json["data"]["user"]["tenants"][0]["tenant_slug"], "test-company");
		assert_eq!(json["data"]["user"]["tenants"][0]["role"], "owner");
	}
	
	#[tokio::test]
	async fn test_monitor_crud_with_auth() {
		let (app, pool) = setup_test_app().await;
		
		// Clean up database
		sqlx::query!("DELETE FROM tenant_monitors").execute(&pool).await.ok();
		sqlx::query!("DELETE FROM tenant_networks").execute(&pool).await.ok();
		sqlx::query!("DELETE FROM tenant_memberships").execute(&pool).await.ok();
		sqlx::query!("DELETE FROM users").execute(&pool).await.ok();
		sqlx::query!("DELETE FROM tenants").execute(&pool).await.ok();
		
		// Create a test user and tenant
		let auth_service = utils::AuthService::new("test-secret".to_string());
		
		let tenant = sqlx::query_as!(
			models::Tenant,
			"INSERT INTO tenants (name, slug) VALUES ($1, $2) RETURNING *",
			"Test Tenant",
			"test-tenant"
		)
		.fetch_one(&pool)
		.await
		.unwrap();
		
		let user = sqlx::query_as!(
			models::User,
			"INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
			"test@example.com",
			auth_service.hash_password("password").unwrap()
		)
		.fetch_one(&pool)
		.await
		.unwrap();
		
		sqlx::query!(
			"INSERT INTO tenant_memberships (tenant_id, user_id, role) VALUES ($1, $2, 'owner')",
			tenant.id,
			user.id
		)
		.execute(&pool)
		.await
		.unwrap();
		
		// Create a network for the monitor
		let network_id = sqlx::query_scalar!(
			"INSERT INTO tenant_networks (tenant_id, network_id, name, blockchain, configuration) 
			 VALUES ($1, $2, $3, $4, $5) RETURNING id",
			tenant.id,
			"stellar-mainnet",
			"Stellar Mainnet",
			"stellar",
			serde_json::json!({})
		)
		.fetch_one(&pool)
		.await
		.unwrap();
		
		// Generate JWT token
		let token = auth_service.generate_jwt(&user).unwrap();
		
		// Test creating a monitor
		let create_request = serde_json::json!({
			"monitor_id": "test-monitor",
			"name": "Test Monitor",
			"network_id": network_id,
			"configuration": {
				"test": true
			}
		});
		
		let response = app.clone()
			.oneshot(
				Request::builder()
					.method("POST")
					.uri("/api/v1/tenants/test-tenant/monitors")
					.header("authorization", format!("Bearer {}", token))
					.header("content-type", "application/json")
					.body(Body::from(create_request.to_string()))
					.unwrap(),
			)
			.await
			.unwrap();
		
		assert_eq!(response.status(), StatusCode::CREATED);
		
		// Test listing monitors
		let response = app.clone()
			.oneshot(
				Request::builder()
					.uri("/api/v1/tenants/test-tenant/monitors")
					.header("authorization", format!("Bearer {}", token))
					.body(Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();
		
		assert_eq!(response.status(), StatusCode::OK);
		
		let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
			.await
			.unwrap();
		let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
		
		assert_eq!(json["data"].as_array().unwrap().len(), 1);
		assert_eq!(json["data"][0]["name"], "Test Monitor");
		assert_eq!(json["meta"]["total"], 1);
	}
}