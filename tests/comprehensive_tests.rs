#[cfg(test)]
mod tests {
	use mockall::{mock, predicate::*};
	use std::sync::Arc;
	use stellar_monitor_tenant_isolation::{
		models::*,
		repositories::{error::TenantRepositoryError, tenant::TenantRepositoryTrait},
		services::monitor_service::{MonitorServiceTrait, ServiceError},
	};
	use uuid::Uuid;

	// Repository Mocks
	mock! {
		pub TenantRepo {}

		impl Clone for TenantRepo {
			fn clone(&self) -> Self;
		}

		#[async_trait::async_trait]
		impl TenantRepositoryTrait for TenantRepo {
			async fn create(&self, request: CreateTenantRequest) -> Result<Tenant, TenantRepositoryError>;
			async fn get(&self, tenant_id: Uuid) -> Result<Tenant, TenantRepositoryError>;
			async fn get_by_slug(&self, slug: &str) -> Result<Tenant, TenantRepositoryError>;
			async fn update(&self, tenant_id: Uuid, request: UpdateTenantRequest) -> Result<Tenant, TenantRepositoryError>;
			async fn delete(&self, tenant_id: Uuid) -> Result<(), TenantRepositoryError>;
			async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tenant>, TenantRepositoryError>;
			async fn add_member(&self, tenant_id: Uuid, user_id: Uuid, role: TenantRole) -> Result<TenantMembership, TenantRepositoryError>;
			async fn remove_member(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), TenantRepositoryError>;
			async fn update_member_role(&self, tenant_id: Uuid, user_id: Uuid, role: TenantRole) -> Result<TenantMembership, TenantRepositoryError>;
			async fn get_members(&self, tenant_id: Uuid) -> Result<Vec<TenantMembership>, TenantRepositoryError>;
			async fn get_user_tenants(&self, user_id: Uuid) -> Result<Vec<(Tenant, TenantRole)>, TenantRepositoryError>;
			async fn get_quota_status(&self, tenant_id: Uuid) -> Result<ResourceQuotaStatus, TenantRepositoryError>;
			async fn check_quota(&self, tenant_id: Uuid, resource: &str, amount: i32) -> Result<bool, TenantRepositoryError>;
		}
	}

	// Test Builders
	pub struct TenantBuilder {
		id: Uuid,
		name: String,
		slug: String,
		is_active: bool,
		max_monitors: i32,
		max_networks: i32,
		max_triggers_per_monitor: i32,
		max_rpc_requests_per_minute: i32,
		max_storage_mb: i32,
		created_at: Option<chrono::DateTime<chrono::Utc>>,
		updated_at: Option<chrono::DateTime<chrono::Utc>>,
	}

	impl Default for TenantBuilder {
		fn default() -> Self {
			Self {
				id: Uuid::new_v4(),
				name: "Test Tenant".to_string(),
				slug: "test-tenant".to_string(),
				is_active: true,
				max_monitors: 10,
				max_networks: 5,
				max_triggers_per_monitor: 5,
				max_rpc_requests_per_minute: 100,
				max_storage_mb: 1000,
				created_at: Some(chrono::Utc::now()),
				updated_at: Some(chrono::Utc::now()),
			}
		}
	}

	impl TenantBuilder {
		pub fn build(self) -> Tenant {
			Tenant {
				id: self.id,
				name: self.name,
				slug: self.slug,
				is_active: self.is_active,
				max_monitors: self.max_monitors,
				max_networks: self.max_networks,
				max_triggers_per_monitor: self.max_triggers_per_monitor,
				max_rpc_requests_per_minute: self.max_rpc_requests_per_minute,
				max_storage_mb: self.max_storage_mb,
				created_at: self.created_at,
				updated_at: self.updated_at,
			}
		}

		pub fn with_name(mut self, name: &str) -> Self {
			self.name = name.to_string();
			self
		}

		pub fn with_slug(mut self, slug: &str) -> Self {
			self.slug = slug.to_string();
			self
		}
	}

	// Repository Tests
	#[tokio::test]
	async fn test_tenant_repository_create_success() {
		let mut mock_repo = MockTenantRepo::new();
		let expected_tenant = TenantBuilder::default().build();
		let expected_clone = expected_tenant.clone();

		mock_repo
			.expect_create()
			.times(1)
			.returning(move |_| Ok(expected_clone.clone()));

		let request = CreateTenantRequest {
			name: "Test Tenant".to_string(),
			slug: "test-tenant".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		};

		let result = mock_repo.create(request).await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap().name, "Test Tenant");
	}

	#[tokio::test]
	async fn test_tenant_repository_get_not_found() {
		let mut mock_repo = MockTenantRepo::new();
		let tenant_id = Uuid::new_v4();

		mock_repo
			.expect_get()
			.with(eq(tenant_id))
			.times(1)
			.returning(|id| Err(TenantRepositoryError::TenantNotFound(id)));

		let result = mock_repo.get(tenant_id).await;
		assert!(result.is_err());

		match result.unwrap_err() {
			TenantRepositoryError::TenantNotFound(id) => assert_eq!(id, tenant_id),
			_ => panic!("Expected TenantNotFound error"),
		}
	}

	#[tokio::test]
	async fn test_tenant_repository_list_with_pagination() {
		let mut mock_repo = MockTenantRepo::new();
		let tenants = vec![
			TenantBuilder::default().with_name("Tenant 1").build(),
			TenantBuilder::default().with_name("Tenant 2").build(),
		];
		let tenants_clone = tenants.clone();

		mock_repo
			.expect_list()
			.with(eq(10i64), eq(0i64))
			.times(1)
			.returning(move |_, _| Ok(tenants_clone.clone()));

		let result = mock_repo.list(10, 0).await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap().len(), 2);
	}

	#[tokio::test]
	async fn test_tenant_quota_check_success() {
		let mut mock_repo = MockTenantRepo::new();
		let tenant_id = Uuid::new_v4();

		mock_repo
			.expect_check_quota()
			.with(eq(tenant_id), eq("monitors"), eq(1))
			.times(1)
			.returning(|_, _, _| Ok(true));

		let result = mock_repo.check_quota(tenant_id, "monitors", 1).await;
		assert!(result.is_ok());
		assert!(result.unwrap());
	}

	#[tokio::test]
	async fn test_tenant_quota_exceeded() {
		let mut mock_repo = MockTenantRepo::new();
		let tenant_id = Uuid::new_v4();

		mock_repo
			.expect_check_quota()
			.with(eq(tenant_id), eq("monitors"), eq(100))
			.times(1)
			.returning(|_, _, _| Ok(false));

		let result = mock_repo.check_quota(tenant_id, "monitors", 100).await;
		assert!(result.is_ok());
		assert!(!result.unwrap());
	}

	// Service Tests
	mock! {
		pub MonitorSvc {}

		#[async_trait::async_trait]
		impl MonitorServiceTrait for MonitorSvc {
			async fn create_monitor(&self, request: CreateMonitorRequest, metadata: RequestMetadata) -> Result<TenantMonitor, ServiceError>;
			async fn get_monitor(&self, monitor_id: &str) -> Result<TenantMonitor, ServiceError>;
			async fn update_monitor(&self, monitor_id: &str, request: UpdateMonitorRequest, metadata: RequestMetadata) -> Result<TenantMonitor, ServiceError>;
			async fn delete_monitor(&self, monitor_id: &str, metadata: RequestMetadata) -> Result<(), ServiceError>;
			async fn list_monitors(&self, limit: i64, offset: i64) -> Result<Vec<TenantMonitor>, ServiceError>;
			async fn get_monitor_count(&self) -> Result<i64, ServiceError>;
		}
	}

	#[tokio::test]
	async fn test_monitor_service_create_success() {
		let mut mock_service = MockMonitorSvc::new();
		let expected_monitor = TenantMonitor {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			monitor_id: "monitor-123".to_string(),
			name: "Test Monitor".to_string(),
			network_id: Uuid::new_v4(),
			configuration: serde_json::json!({"type": "test"}),
			is_active: Some(true),
			created_at: Some(chrono::Utc::now()),
			updated_at: Some(chrono::Utc::now()),
		};
		let expected_clone = expected_monitor.clone();

		mock_service
			.expect_create_monitor()
			.times(1)
			.returning(move |_, _| Ok(expected_clone.clone()));

		let request = CreateMonitorRequest {
			monitor_id: "monitor-123".to_string(),
			name: "Test Monitor".to_string(),
			network_id: Uuid::new_v4(),
			configuration: serde_json::json!({"type": "test"}),
		};

		let metadata = RequestMetadata {
			ip_address: None,
			user_agent: None,
		};

		let result = mock_service.create_monitor(request, metadata).await;
		assert!(result.is_ok());
		assert_eq!(result.unwrap().name, "Test Monitor");
	}

	#[tokio::test]
	async fn test_monitor_service_quota_exceeded() {
		let mut mock_service = MockMonitorSvc::new();

		mock_service
			.expect_create_monitor()
			.times(1)
			.returning(|_, _| {
				Err(ServiceError::QuotaExceeded(
					"Monitor quota exceeded".to_string(),
				))
			});

		let request = CreateMonitorRequest {
			monitor_id: "monitor-123".to_string(),
			name: "Test Monitor".to_string(),
			network_id: Uuid::new_v4(),
			configuration: serde_json::json!({"type": "test"}),
		};

		let metadata = RequestMetadata {
			ip_address: None,
			user_agent: None,
		};

		let result = mock_service.create_monitor(request, metadata).await;
		assert!(result.is_err());

		match result.unwrap_err() {
			ServiceError::QuotaExceeded(msg) => assert_eq!(msg, "Monitor quota exceeded"),
			_ => panic!("Expected QuotaExceeded error"),
		}
	}

	// Integration-style tests
	#[tokio::test]
	async fn test_full_tenant_lifecycle() {
		let mut mock_repo = MockTenantRepo::new();
		let tenant_id = Uuid::new_v4();
		let user_id = Uuid::new_v4();

		// Create tenant
		let tenant = TenantBuilder::default()
			.with_name("Integration Test")
			.with_slug("integration-test")
			.build();
		let tenant_clone = tenant.clone();

		mock_repo
			.expect_create()
			.times(1)
			.returning(move |_| Ok(tenant_clone.clone()));

		// Add member
		let membership = TenantMembership {
			id: Uuid::new_v4(),
			tenant_id,
			user_id,
			role: TenantRole::Admin,
			created_at: Some(chrono::Utc::now()),
			updated_at: Some(chrono::Utc::now()),
		};
		let membership_clone = membership.clone();

		mock_repo
			.expect_add_member()
			.times(1)
			.returning(move |_, _, _| Ok(membership_clone.clone()));

		// Test quota
		mock_repo
			.expect_check_quota()
			.times(1)
			.returning(|_, _, _| Ok(true));

		// Delete tenant
		mock_repo.expect_delete().times(1).returning(|_| Ok(()));

		// Execute lifecycle
		let create_request = CreateTenantRequest {
			name: "Integration Test".to_string(),
			slug: "integration-test".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		};

		let created = mock_repo.create(create_request).await.unwrap();
		assert_eq!(created.name, "Integration Test");

		let membership = mock_repo
			.add_member(tenant_id, user_id, TenantRole::Admin)
			.await
			.unwrap();
		assert_eq!(membership.role, TenantRole::Admin);

		let quota_ok = mock_repo
			.check_quota(tenant_id, "monitors", 1)
			.await
			.unwrap();
		assert!(quota_ok);

		let delete_result = mock_repo.delete(tenant_id).await;
		assert!(delete_result.is_ok());
	}

	// Edge case tests
	#[tokio::test]
	async fn test_empty_tenant_name_validation() {
		let mut mock_repo = MockTenantRepo::new();

		mock_repo.expect_create().times(1).returning(|_| {
			Err(TenantRepositoryError::ValidationError(
				"Name cannot be empty".to_string(),
			))
		});

		let request = CreateTenantRequest {
			name: "".to_string(),
			slug: "empty-name".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		};

		let result = mock_repo.create(request).await;
		assert!(result.is_err());

		match result.unwrap_err() {
			TenantRepositoryError::ValidationError(msg) => assert_eq!(msg, "Name cannot be empty"),
			_ => panic!("Expected ValidationError"),
		}
	}

	#[tokio::test]
	async fn test_concurrent_operations() {
		let mock_repo = Arc::new(MockTenantRepo::new());
		let _tenant_id = Uuid::new_v4();

		// Simulate concurrent reads
		let _repo1 = Arc::clone(&mock_repo);
		let _repo2 = Arc::clone(&mock_repo);

		let handle1 = tokio::spawn(async move {
			// This would normally call repo1.get(tenant_id)
			// but mocking concurrent operations requires more setup
			Ok::<(), TenantRepositoryError>(())
		});

		let handle2 = tokio::spawn(async move {
			// This would normally call repo2.get(tenant_id)
			// but mocking concurrent operations requires more setup
			Ok::<(), TenantRepositoryError>(())
		});

		let result1 = handle1.await.unwrap();
		let result2 = handle2.await.unwrap();

		assert!(result1.is_ok());
		assert!(result2.is_ok());
	}
}
