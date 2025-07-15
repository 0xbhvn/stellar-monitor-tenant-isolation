use mockall::predicate::*;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::{error::TenantRepositoryError, tenant::TenantRepositoryTrait},
};
use uuid::Uuid;

use crate::{
	mocks::MockTenantRepository,
	utils::{
		builders::{CreateTenantRequestBuilder, TenantBuilder, UpdateTenantRequestBuilder},
		fixtures::TestIds,
	},
};

#[tokio::test]
async fn test_create_tenant_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let request = CreateTenantRequestBuilder::new()
		.with_name("Acme Corp")
		.with_slug("acme-corp")
		.build();

	let expected_tenant = TenantBuilder::new()
		.with_name("Acme Corp")
		.with_slug("acme-corp")
		.build();

	let expected_tenant_clone = expected_tenant.clone();
	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(move |_| Ok(expected_tenant_clone.clone()));

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_ok());
	let tenant = result.unwrap();
	assert_eq!(tenant.name, "Acme Corp");
	assert_eq!(tenant.slug, "acme-corp");
}

#[tokio::test]
async fn test_create_tenant_duplicate_slug() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let request = CreateTenantRequestBuilder::new()
		.with_slug("existing-slug")
		.build();

	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::AlreadyExists {
				resource_type: "tenant".to_string(),
				resource_id: "existing-slug".to_string(),
			})
		});

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::AlreadyExists { resource_type, .. } => {
			assert_eq!(resource_type, "tenant");
		}
		_ => panic!("Expected AlreadyExists error"),
	}
}

#[tokio::test]
async fn test_get_tenant_by_id_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();
	let expected_tenant = TenantBuilder::new().with_id(test_ids.tenant_1).build();

	let expected_tenant_clone = expected_tenant.clone();
	mock_repo
		.expect_get()
		.with(eq(test_ids.tenant_1))
		.times(1)
		.returning(move |_| Ok(expected_tenant_clone.clone()));

	// Act
	let result = mock_repo.get(test_ids.tenant_1).await;

	// Assert
	assert!(result.is_ok());
	let tenant = result.unwrap();
	assert_eq!(tenant.id, test_ids.tenant_1);
}

#[tokio::test]
async fn test_get_tenant_by_id_not_found() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let unknown_id = Uuid::new_v4();

	mock_repo
		.expect_get()
		.with(eq(unknown_id))
		.times(1)
		.returning(|id| Err(TenantRepositoryError::TenantNotFound(id)));

	// Act
	let result = mock_repo.get(unknown_id).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::TenantNotFound(id) => {
			assert_eq!(id, unknown_id);
		}
		_ => panic!("Expected TenantNotFound error"),
	}
}

#[tokio::test]
async fn test_get_tenant_by_slug_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let expected_tenant = TenantBuilder::new().with_slug("test-slug").build();

	let expected_tenant_clone = expected_tenant.clone();
	mock_repo
		.expect_get_by_slug()
		.with(eq("test-slug"))
		.times(1)
		.returning(move |_| Ok(expected_tenant_clone.clone()));

	// Act
	let result = mock_repo.get_by_slug("test-slug").await;

	// Assert
	assert!(result.is_ok());
	let tenant = result.unwrap();
	assert_eq!(tenant.slug, "test-slug");
}

#[tokio::test]
async fn test_update_tenant_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();
	let update_request = UpdateTenantRequestBuilder::new()
		.with_name("Updated Name")
		.with_max_monitors(20)
		.build();

	let updated_tenant = TenantBuilder::new()
		.with_id(test_ids.tenant_1)
		.with_name("Updated Name")
		.with_max_monitors(20)
		.build();

	let updated_tenant_clone = updated_tenant.clone();
	mock_repo
		.expect_update()
		.with(eq(test_ids.tenant_1), always())
		.times(1)
		.returning(move |_, _| Ok(updated_tenant_clone.clone()));

	// Act
	let result = mock_repo.update(test_ids.tenant_1, update_request).await;

	// Assert
	assert!(result.is_ok());
	let tenant = result.unwrap();
	assert_eq!(tenant.name, "Updated Name");
	assert_eq!(tenant.max_monitors, 20);
}

#[tokio::test]
async fn test_update_tenant_not_found() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let unknown_id = Uuid::new_v4();
	let update_request = UpdateTenantRequestBuilder::new()
		.with_name("Updated Name")
		.build();

	mock_repo
		.expect_update()
		.with(eq(unknown_id), always())
		.times(1)
		.returning(|id, _| Err(TenantRepositoryError::TenantNotFound(id)));

	// Act
	let result = mock_repo.update(unknown_id, update_request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::TenantNotFound(id) => {
			assert_eq!(id, unknown_id);
		}
		_ => panic!("Expected TenantNotFound error"),
	}
}

#[tokio::test]
async fn test_delete_tenant_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	mock_repo
		.expect_delete()
		.with(eq(test_ids.tenant_1))
		.times(1)
		.returning(|_| Ok(()));

	// Act
	let result = mock_repo.delete(test_ids.tenant_1).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_tenant_not_found() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let unknown_id = Uuid::new_v4();

	mock_repo
		.expect_delete()
		.with(eq(unknown_id))
		.times(1)
		.returning(|id| Err(TenantRepositoryError::TenantNotFound(id)));

	// Act
	let result = mock_repo.delete(unknown_id).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::TenantNotFound(id) => {
			assert_eq!(id, unknown_id);
		}
		_ => panic!("Expected TenantNotFound error"),
	}
}

#[tokio::test]
async fn test_list_tenants_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let tenant1 = TenantBuilder::new().with_name("Tenant 1").build();
	let tenant2 = TenantBuilder::new().with_name("Tenant 2").build();
	let tenants = vec![tenant1, tenant2];

	let tenants_clone = tenants.clone();
	mock_repo
		.expect_list()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(move |_, _| Ok(tenants_clone.clone()));

	// Act
	let result = mock_repo.list(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_tenants = result.unwrap();
	assert_eq!(returned_tenants.len(), 2);
}

#[tokio::test]
async fn test_list_tenants_with_pagination() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let tenant = TenantBuilder::new().with_name("Page 2 Tenant").build();
	let tenants = vec![tenant];

	let tenants_clone = tenants.clone();
	mock_repo
		.expect_list()
		.with(eq(5i64), eq(5i64))
		.times(1)
		.returning(move |_, _| Ok(tenants_clone.clone()));

	// Act
	let result = mock_repo.list(5, 5).await;

	// Assert
	assert!(result.is_ok());
	let returned_tenants = result.unwrap();
	assert_eq!(returned_tenants.len(), 1);
}

// Membership tests
#[tokio::test]
async fn test_add_member_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	let membership = TenantMembership {
		id: Uuid::new_v4(),
		tenant_id: test_ids.tenant_1,
		user_id: test_ids.user_1,
		role: TenantRole::Member,
		created_at: Some(chrono::Utc::now()),
		updated_at: Some(chrono::Utc::now()),
	};

	let membership_clone = membership.clone();
	mock_repo
		.expect_add_member()
		.with(
			eq(test_ids.tenant_1),
			eq(test_ids.user_1),
			eq(TenantRole::Member),
		)
		.times(1)
		.returning(move |_, _, _| Ok(membership_clone.clone()));

	// Act
	let result = mock_repo
		.add_member(test_ids.tenant_1, test_ids.user_1, TenantRole::Member)
		.await;

	// Assert
	assert!(result.is_ok());
	let returned_membership = result.unwrap();
	assert_eq!(returned_membership.tenant_id, test_ids.tenant_1);
	assert_eq!(returned_membership.user_id, test_ids.user_1);
	assert_eq!(returned_membership.role, TenantRole::Member);
}

#[tokio::test]
async fn test_remove_member_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	mock_repo
		.expect_remove_member()
		.with(eq(test_ids.tenant_1), eq(test_ids.user_1))
		.times(1)
		.returning(|_, _| Ok(()));

	// Act
	let result = mock_repo
		.remove_member(test_ids.tenant_1, test_ids.user_1)
		.await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_member_role_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	let updated_membership = TenantMembership {
		id: Uuid::new_v4(),
		tenant_id: test_ids.tenant_1,
		user_id: test_ids.user_1,
		role: TenantRole::Admin,
		created_at: Some(chrono::Utc::now()),
		updated_at: Some(chrono::Utc::now()),
	};

	let membership_clone = updated_membership.clone();
	mock_repo
		.expect_update_member_role()
		.with(
			eq(test_ids.tenant_1),
			eq(test_ids.user_1),
			eq(TenantRole::Admin),
		)
		.times(1)
		.returning(move |_, _, _| Ok(membership_clone.clone()));

	// Act
	let result = mock_repo
		.update_member_role(test_ids.tenant_1, test_ids.user_1, TenantRole::Admin)
		.await;

	// Assert
	assert!(result.is_ok());
	let returned_membership = result.unwrap();
	assert_eq!(returned_membership.role, TenantRole::Admin);
}

#[tokio::test]
async fn test_get_members_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	let members = vec![
		TenantMembership {
			id: Uuid::new_v4(),
			tenant_id: test_ids.tenant_1,
			user_id: test_ids.user_1,
			role: TenantRole::Owner,
			created_at: Some(chrono::Utc::now()),
			updated_at: Some(chrono::Utc::now()),
		},
		TenantMembership {
			id: Uuid::new_v4(),
			tenant_id: test_ids.tenant_1,
			user_id: test_ids.user_2,
			role: TenantRole::Member,
			created_at: Some(chrono::Utc::now()),
			updated_at: Some(chrono::Utc::now()),
		},
	];

	let members_clone = members.clone();
	mock_repo
		.expect_get_members()
		.with(eq(test_ids.tenant_1))
		.times(1)
		.returning(move |_| Ok(members_clone.clone()));

	// Act
	let result = mock_repo.get_members(test_ids.tenant_1).await;

	// Assert
	assert!(result.is_ok());
	let returned_members = result.unwrap();
	assert_eq!(returned_members.len(), 2);
}

// Quota tests
#[tokio::test]
async fn test_check_quota_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	mock_repo
		.expect_check_quota()
		.with(eq(test_ids.tenant_1), eq("monitors"), eq(1))
		.times(1)
		.returning(|_, _, _| Ok(true)); // true means quota is available

	// Act
	let result = mock_repo
		.check_quota(test_ids.tenant_1, "monitors", 1)
		.await;

	// Assert
	assert!(result.is_ok());
	assert!(result.unwrap());
}

#[tokio::test]
async fn test_check_quota_exceeded() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	mock_repo
		.expect_check_quota()
		.with(eq(test_ids.tenant_1), eq("monitors"), eq(100))
		.times(1)
		.returning(|_, _, _| Ok(false)); // false means quota exceeded

	// Act
	let result = mock_repo
		.check_quota(test_ids.tenant_1, "monitors", 100)
		.await;

	// Assert
	assert!(result.is_ok());
	assert!(!result.unwrap());
}

#[tokio::test]
async fn test_get_quota_status_success() {
	// Arrange
	let mut mock_repo = MockTenantRepository::new();
	let test_ids = TestIds::default();

	let quota_status = ResourceQuotaStatus {
		tenant_id: test_ids.tenant_1,
		quotas: TenantQuotas {
			max_monitors: 10,
			max_networks: 5,
			max_triggers_per_monitor: 10,
			max_rpc_requests_per_minute: 6000,
			max_storage_mb: 1000,
			api_rate_limits: Default::default(),
		},
		usage: CurrentUsage {
			monitors_count: 5,
			networks_count: 2,
			triggers_count: 15,
			rpc_requests_last_minute: 500,
			storage_mb_used: 100,
		},
		available: AvailableResources {
			monitors: 5,
			networks: 3,
			triggers: 35,
			rpc_requests_per_minute: 5500,
			storage_mb: 900,
		},
	};

	let quota_status_clone = quota_status.clone();
	mock_repo
		.expect_get_quota_status()
		.with(eq(test_ids.tenant_1))
		.times(1)
		.returning(move |_| Ok(quota_status_clone.clone()));

	// Act
	let result = mock_repo.get_quota_status(test_ids.tenant_1).await;

	// Assert
	assert!(result.is_ok());
	let status = result.unwrap();
	assert_eq!(status.usage.monitors_count, 5);
	assert_eq!(status.quotas.max_monitors, 10);
	assert_eq!(status.available.monitors, 5);
}
