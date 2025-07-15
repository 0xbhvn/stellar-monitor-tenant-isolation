use mockall::predicate::*;
use stellar_monitor_tenant_isolation::{
	models::*,
	services::{AuditServiceTrait, ServiceError},
};
use uuid::Uuid;

use crate::{
	mocks::MockAuditService,
	utils::{builders::CreateAuditLogRequestBuilder, fixtures::TestIds},
};

#[tokio::test]
async fn test_log_success() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let request = CreateAuditLogRequestBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_user_id(test_ids.user_1)
		.with_action(AuditAction::MonitorCreated)
		.with_resource_type(ResourceType::Monitor)
		.with_resource_id(test_ids.monitor_1)
		.build();

	mock_service
		.expect_log()
		.with(always())
		.times(1)
		.returning(move |_| Ok(()));

	// Act
	let result = mock_service.log(request).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_log_different_actions() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	// Test various audit actions
	let actions = vec![
		AuditAction::TenantCreated,
		AuditAction::TenantUpdated,
		AuditAction::TenantDeleted,
		AuditAction::MonitorCreated,
		AuditAction::MonitorUpdated,
		AuditAction::MonitorDeleted,
		AuditAction::NetworkCreated,
		AuditAction::NetworkUpdated,
		AuditAction::NetworkDeleted,
		AuditAction::TriggerCreated,
		AuditAction::TriggerUpdated,
		AuditAction::TriggerDeleted,
		AuditAction::UserInvited,
		AuditAction::UserRemoved,
		AuditAction::UserRoleChanged,
	];

	for action in actions {
		let request = CreateAuditLogRequestBuilder::new()
			.with_tenant_id(test_ids.tenant_1)
			.with_user_id(test_ids.user_1)
			.with_action(action.clone())
			.build();

		mock_service
			.expect_log()
			.with(always())
			.times(1)
			.returning(move |_| Ok(()));

		let result = mock_service.log(request).await;
		assert!(result.is_ok());
	}
}

#[tokio::test]
async fn test_audit_log_with_metadata() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let request = CreateAuditLogRequestBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_user_id(test_ids.user_1)
		.with_action(AuditAction::MonitorCreated)
		.with_ip_address("192.168.1.100")
		.with_changes(serde_json::json!({
			"old_value": null,
			"new_value": "monitor-123"
		}))
		.build();

	mock_service
		.expect_log()
		.with(always())
		.times(1)
		.returning(move |_| Ok(()));

	// Act
	let result = mock_service.log(request).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_audit_service_error() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let request = CreateAuditLogRequestBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_user_id(test_ids.user_1)
		.with_action(AuditAction::MonitorCreated)
		.build();

	mock_service
		.expect_log()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(ServiceError::Internal(
				"Database connection failed".to_string(),
			))
		});

	// Act
	let result = mock_service.log(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::Internal(msg) => assert_eq!(msg, "Database connection failed"),
		_ => panic!("Expected Internal error"),
	}
}

#[tokio::test]
async fn test_log_with_api_key() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let request = CreateAuditLogRequestBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_api_key_id(Uuid::new_v4())
		.with_action(AuditAction::MonitorCreated)
		.with_resource_type(ResourceType::Monitor)
		.with_resource_id(test_ids.monitor_1)
		.build();

	mock_service
		.expect_log()
		.with(always())
		.times(1)
		.returning(move |_| Ok(()));

	// Act
	let result = mock_service.log(request).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_log_network_actions() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let network_actions = vec![
		AuditAction::NetworkCreated,
		AuditAction::NetworkUpdated,
		AuditAction::NetworkDeleted,
	];

	for action in network_actions {
		let request = CreateAuditLogRequestBuilder::new()
			.with_tenant_id(test_ids.tenant_1)
			.with_user_id(test_ids.user_1)
			.with_action(action)
			.with_resource_type(ResourceType::Network)
			.with_resource_id(test_ids.network_1)
			.build();

		mock_service
			.expect_log()
			.with(always())
			.times(1)
			.returning(move |_| Ok(()));

		let result = mock_service.log(request).await;
		assert!(result.is_ok());
	}
}

#[tokio::test]
async fn test_log_trigger_actions() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let trigger_actions = vec![
		AuditAction::TriggerCreated,
		AuditAction::TriggerUpdated,
		AuditAction::TriggerDeleted,
	];

	for action in trigger_actions {
		let request = CreateAuditLogRequestBuilder::new()
			.with_tenant_id(test_ids.tenant_1)
			.with_user_id(test_ids.user_1)
			.with_action(action)
			.with_resource_type(ResourceType::Trigger)
			.with_resource_id(test_ids.trigger_1)
			.build();

		mock_service
			.expect_log()
			.with(always())
			.times(1)
			.returning(move |_| Ok(()));

		let result = mock_service.log(request).await;
		assert!(result.is_ok());
	}
}

#[tokio::test]
async fn test_log_member_actions() {
	// Arrange
	let mut mock_service = MockAuditService::new();
	let test_ids = TestIds::default();

	let member_actions = vec![
		AuditAction::UserInvited,
		AuditAction::UserRemoved,
		AuditAction::UserRoleChanged,
	];

	for action in member_actions {
		let request = CreateAuditLogRequestBuilder::new()
			.with_tenant_id(test_ids.tenant_1)
			.with_user_id(test_ids.user_1)
			.with_action(action)
			.with_resource_type(ResourceType::User)
			.with_resource_id(test_ids.user_2)
			.build();

		mock_service
			.expect_log()
			.with(always())
			.times(1)
			.returning(move |_| Ok(()));

		let result = mock_service.log(request).await;
		assert!(result.is_ok());
	}
}
