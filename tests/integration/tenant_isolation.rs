#[cfg(test)]
mod tests {
	use stellar_monitor_tenant_isolation::*;
	use sqlx::postgres::{PgPoolOptions, PgPool};
	use uuid::Uuid;
	
	// Test helper to create a test database pool
	async fn create_test_pool() -> PgPool {
		dotenvy::dotenv().ok();
		
		let database_url = std::env::var("TEST_DATABASE_URL")
			.unwrap_or_else(|_| "postgres://postgres:password@localhost/stellar_monitor_test".to_string());
		
		PgPoolOptions::new()
			.max_connections(5)
			.connect(&database_url)
			.await
			.expect("Failed to connect to test database")
	}
	
	// Test helper to run migrations
	async fn run_migrations(pool: &PgPool) {
		sqlx::migrate!("./migrations")
			.run(pool)
			.await
			.expect("Failed to run migrations");
	}
	
	// Test helper to clean up database
	async fn cleanup_database(pool: &PgPool) {
		// Delete all data in reverse order of foreign key dependencies
		sqlx::query!("DELETE FROM audit_logs").execute(pool).await.ok();
		sqlx::query!("DELETE FROM resource_usage").execute(pool).await.ok();
		sqlx::query!("DELETE FROM tenant_triggers").execute(pool).await.ok();
		sqlx::query!("DELETE FROM tenant_monitors").execute(pool).await.ok();
		sqlx::query!("DELETE FROM tenant_networks").execute(pool).await.ok();
		sqlx::query!("DELETE FROM api_keys").execute(pool).await.ok();
		sqlx::query!("DELETE FROM tenant_memberships").execute(pool).await.ok();
		sqlx::query!("DELETE FROM users").execute(pool).await.ok();
		sqlx::query!("DELETE FROM tenants").execute(pool).await.ok();
	}
	
	#[tokio::test]
	async fn test_tenant_isolation_repositories() {
		let pool = create_test_pool().await;
		run_migrations(&pool).await;
		cleanup_database(&pool).await;
		
		// Create repositories
		let tenant_repo = repositories::TenantRepository::new(pool.clone());
		let monitor_repo = repositories::TenantMonitorRepository::new(pool.clone());
		
		// Create two tenants
		let tenant1 = tenant_repo.create(models::CreateTenantRequest {
			name: "Tenant 1".to_string(),
			slug: "tenant-1".to_string(),
			max_monitors: Some(5),
			max_networks: Some(3),
			max_triggers_per_monitor: Some(10),
			max_rpc_requests_per_minute: Some(1000),
			max_storage_mb: Some(500),
		}).await.unwrap();
		
		let tenant2 = tenant_repo.create(models::CreateTenantRequest {
			name: "Tenant 2".to_string(),
			slug: "tenant-2".to_string(),
			max_monitors: Some(5),
			max_networks: Some(3),
			max_triggers_per_monitor: Some(10),
			max_rpc_requests_per_minute: Some(1000),
			max_storage_mb: Some(500),
		}).await.unwrap();
		
		// Create monitor in tenant 1's context
		let monitor1 = utils::with_tenant_context(
			utils::TenantContext::new(tenant1.id),
			async {
				monitor_repo.create(models::CreateMonitorRequest {
					monitor_id: "monitor-1".to_string(),
					name: "Tenant 1 Monitor".to_string(),
					network_id: Uuid::new_v4(), // Would normally be a real network
					configuration: serde_json::json!({
						"test": true
					}),
				}).await
			}
		).await.unwrap();
		
		// Try to access monitor from tenant 2's context - should fail
		let result = utils::with_tenant_context(
			utils::TenantContext::new(tenant2.id),
			async {
				monitor_repo.get("monitor-1").await
			}
		).await;
		
		assert!(result.is_err());
		assert!(matches!(
			result.unwrap_err(),
			repositories::TenantRepositoryError::ResourceNotFound { .. }
		));
		
		// Access monitor from correct tenant - should succeed
		let retrieved_monitor = utils::with_tenant_context(
			utils::TenantContext::new(tenant1.id),
			async {
				monitor_repo.get("monitor-1").await
			}
		).await.unwrap();
		
		assert_eq!(retrieved_monitor.id, monitor1.id);
		assert_eq!(retrieved_monitor.name, "Tenant 1 Monitor");
		
		cleanup_database(&pool).await;
	}
	
	#[tokio::test]
	async fn test_quota_enforcement() {
		let pool = create_test_pool().await;
		run_migrations(&pool).await;
		cleanup_database(&pool).await;
		
		// Create repositories
		let tenant_repo = repositories::TenantRepository::new(pool.clone());
		let monitor_repo = repositories::TenantMonitorRepository::new(pool.clone());
		
		// Create tenant with low quota
		let tenant = tenant_repo.create(models::CreateTenantRequest {
			name: "Limited Tenant".to_string(),
			slug: "limited-tenant".to_string(),
			max_monitors: Some(2), // Only allow 2 monitors
			max_networks: Some(3),
			max_triggers_per_monitor: Some(10),
			max_rpc_requests_per_minute: Some(1000),
			max_storage_mb: Some(500),
		}).await.unwrap();
		
		// Create monitors up to the limit
		let context = utils::TenantContext::new(tenant.id);
		
		for i in 0..2 {
			let result = utils::with_tenant_context(
				context.clone(),
				async {
					monitor_repo.create(models::CreateMonitorRequest {
						monitor_id: format!("monitor-{}", i),
						name: format!("Monitor {}", i),
						network_id: Uuid::new_v4(),
						configuration: serde_json::json!({}),
					}).await
				}
			).await;
			
			assert!(result.is_ok());
		}
		
		// Try to create one more - should fail
		let result = utils::with_tenant_context(
			context,
			async {
				monitor_repo.create(models::CreateMonitorRequest {
					monitor_id: "monitor-exceed".to_string(),
					name: "Exceeding Monitor".to_string(),
					network_id: Uuid::new_v4(),
					configuration: serde_json::json!({}),
				}).await
			}
		).await;
		
		assert!(result.is_err());
		assert!(matches!(
			result.unwrap_err(),
			repositories::TenantRepositoryError::QuotaExceeded(_)
		));
		
		cleanup_database(&pool).await;
	}
	
	#[tokio::test]
	async fn test_user_authentication_and_roles() {
		let pool = create_test_pool().await;
		run_migrations(&pool).await;
		cleanup_database(&pool).await;
		
		let auth_service = utils::AuthService::new("test-secret".to_string());
		
		// Test password hashing and verification
		let password = "secure-password-123";
		let hash = auth_service.hash_password(password).unwrap();
		
		assert!(auth_service.verify_password(password, &hash).unwrap());
		assert!(!auth_service.verify_password("wrong-password", &hash).unwrap());
		
		// Test JWT generation and verification
		let user = models::User {
			id: Uuid::new_v4(),
			email: "test@example.com".to_string(),
			password_hash: hash,
			is_active: true,
			created_at: chrono::Utc::now(),
			updated_at: chrono::Utc::now(),
		};
		
		let token = auth_service.generate_jwt(&user).unwrap();
		let claims = auth_service.verify_jwt(&token).unwrap();
		
		assert_eq!(claims.sub, user.id);
		assert_eq!(claims.email, user.email);
		
		cleanup_database(&pool).await;
	}
}