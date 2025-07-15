use mockall::predicate::*;
use stellar_monitor_tenant_isolation::{
	models::*,
	services::{ServiceError, TriggerServiceTrait},
};

use crate::{
	mocks::MockTriggerService,
	utils::{
		builders::{CreateTriggerRequestBuilder, TriggerBuilder},
		fixtures::{email_trigger_config, webhook_trigger_config, TestIds},
	},
};

#[tokio::test]
async fn test_create_trigger_success() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let test_ids = TestIds::default();

	let request = CreateTriggerRequestBuilder::new()
		.with_monitor_id(test_ids.monitor_1)
		.with_name("Webhook Alert")
		.with_trigger_type("webhook")
		.with_configuration(webhook_trigger_config())
		.build();

	let expected_trigger = TriggerBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_monitor_id(test_ids.monitor_1)
		.with_name("Webhook Alert")
		.with_trigger_type("webhook")
		.build();

	let expected_trigger_clone = expected_trigger.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_trigger()
		.with(always(), always())
		.times(1)
		.returning(move |_, _| Ok(expected_trigger_clone.clone()));

	// Act
	let result = mock_service.create_trigger(request, metadata).await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.name, "Webhook Alert");
	assert_eq!(trigger.trigger_type, "webhook");
}

#[tokio::test]
async fn test_create_trigger_quota_exceeded() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let test_ids = TestIds::default();

	let request = CreateTriggerRequestBuilder::new()
		.with_monitor_id(test_ids.monitor_1)
		.build();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_trigger()
		.with(always(), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::QuotaExceeded(
				"Trigger quota exceeded for monitor".to_string(),
			))
		});

	// Act
	let result = mock_service.create_trigger(request, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::QuotaExceeded(msg) => assert_eq!(msg, "Trigger quota exceeded for monitor"),
		_ => panic!("Expected QuotaExceeded error"),
	}
}

#[tokio::test]
async fn test_get_trigger_success() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let trigger_id = "trigger-123";

	let expected_trigger = TriggerBuilder::new().with_trigger_id(trigger_id).build();

	let expected_trigger_clone = expected_trigger.clone();
	mock_service
		.expect_get_trigger()
		.with(eq(trigger_id))
		.times(1)
		.returning(move |_| Ok(expected_trigger_clone.clone()));

	// Act
	let result = mock_service.get_trigger(trigger_id).await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.trigger_id, trigger_id);
}

#[tokio::test]
async fn test_update_trigger_success() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let trigger_id = "trigger-123";

	let update_request = UpdateTriggerRequest {
		name: Some("Updated Trigger".to_string()),
		configuration: Some(email_trigger_config()),
		is_active: None,
	};

	let updated_trigger = TriggerBuilder::new()
		.with_trigger_id(trigger_id)
		.with_name("Updated Trigger")
		.with_trigger_type("email")
		.build();

	let updated_trigger_clone = updated_trigger.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_update_trigger()
		.with(eq(trigger_id), always(), always())
		.times(1)
		.returning(move |_, _, _| Ok(updated_trigger_clone.clone()));

	// Act
	let result = mock_service
		.update_trigger(trigger_id, update_request, metadata)
		.await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.name, "Updated Trigger");
}

#[tokio::test]
async fn test_delete_trigger_success() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let trigger_id = "trigger-123";
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_delete_trigger()
		.with(eq(trigger_id), always())
		.times(1)
		.returning(|_, _| Ok(()));

	// Act
	let result = mock_service.delete_trigger(trigger_id, metadata).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_triggers_by_monitor() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let test_ids = TestIds::default();

	let triggers = vec![
		TriggerBuilder::new()
			.with_monitor_id(test_ids.monitor_1)
			.with_name("Webhook 1")
			.with_trigger_type("webhook")
			.build(),
		TriggerBuilder::new()
			.with_monitor_id(test_ids.monitor_1)
			.with_name("Email 1")
			.with_trigger_type("email")
			.build(),
	];

	let triggers_clone = triggers.clone();
	mock_service
		.expect_list_triggers_by_monitor()
		.with(eq(test_ids.monitor_1))
		.times(1)
		.returning(move |_| Ok(triggers_clone.clone()));

	// Act
	let result = mock_service
		.list_triggers_by_monitor(test_ids.monitor_1)
		.await;

	// Assert
	assert!(result.is_ok());
	let returned_triggers = result.unwrap();
	assert_eq!(returned_triggers.len(), 2);
}

#[tokio::test]
async fn test_trigger_validation_error() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let test_ids = TestIds::default();

	let request = CreateTriggerRequestBuilder::new()
        .with_monitor_id(test_ids.monitor_1)
        .with_name("")  // Empty name
        .build();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_trigger()
		.with(always(), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::ValidationError(
				"Trigger name cannot be empty".to_string(),
			))
		});

	// Act
	let result = mock_service.create_trigger(request, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::ValidationError(msg) => assert_eq!(msg, "Trigger name cannot be empty"),
		_ => panic!("Expected ValidationError"),
	}
}

#[tokio::test]
async fn test_trigger_type_validation() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let test_ids = TestIds::default();

	let request = CreateTriggerRequestBuilder::new()
		.with_monitor_id(test_ids.monitor_1)
		.with_trigger_type("invalid_type")
		.build();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_trigger()
		.with(always(), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::ValidationError(
				"Invalid trigger type: invalid_type".to_string(),
			))
		});

	// Act
	let result = mock_service.create_trigger(request, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::ValidationError(msg) => assert_eq!(msg, "Invalid trigger type: invalid_type"),
		_ => panic!("Expected ValidationError"),
	}
}

#[tokio::test]
async fn test_unauthorized_trigger_access() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let trigger_id = "private-trigger";

	mock_service
		.expect_get_trigger()
		.with(eq(trigger_id))
		.times(1)
		.returning(|_| {
			Err(ServiceError::AccessDenied(
				"Unauthorized access to trigger".to_string(),
			))
		});

	// Act
	let result = mock_service.get_trigger(trigger_id).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::AccessDenied(msg) => assert_eq!(msg, "Unauthorized access to trigger"),
		_ => panic!("Expected AccessDenied error"),
	}
}

#[tokio::test]
async fn test_trigger_activation_deactivation() {
	// Arrange
	let mut mock_service = MockTriggerService::new();
	let trigger_id = "trigger-123";

	// Test deactivation
	let deactivate_request = UpdateTriggerRequest {
		name: None,
		configuration: None,
		is_active: Some(false),
	};

	let deactivated_trigger = TriggerBuilder::new()
		.with_trigger_id(trigger_id)
		.with_active(false)
		.build();

	let deactivated_trigger_clone = deactivated_trigger.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_update_trigger()
		.with(eq(trigger_id), always(), always())
		.times(1)
		.returning(move |_, _, _| Ok(deactivated_trigger_clone.clone()));

	// Act
	let result = mock_service
		.update_trigger(trigger_id, deactivate_request, metadata.clone())
		.await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.is_active, Some(false));
}
