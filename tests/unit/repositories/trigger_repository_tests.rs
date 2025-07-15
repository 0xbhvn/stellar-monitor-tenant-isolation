use mockall::predicate::*;
use std::collections::HashMap;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::{error::TenantRepositoryError, trigger::TenantTriggerRepositoryTrait},
};
use uuid::Uuid;

use crate::{
	mocks::MockTenantTriggerRepository,
	utils::{
		builders::TriggerBuilder,
		fixtures::{email_trigger_config, slack_trigger_config, webhook_trigger_config, TestIds},
	},
};

#[tokio::test]
async fn test_create_trigger_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	let request = CreateTriggerRequest {
		trigger_id: "webhook-alert-1".to_string(),
		monitor_id: test_ids.monitor_1,
		name: "Webhook Alert".to_string(),
		trigger_type: "webhook".to_string(),
		configuration: webhook_trigger_config(),
	};

	let expected_trigger = TriggerBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_monitor_id(test_ids.monitor_1)
		.with_name("Webhook Alert")
		.with_trigger_type("webhook")
		.build();

	let expected_trigger_clone = expected_trigger.clone();
	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(move |_| Ok(expected_trigger_clone.clone()));

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.name, "Webhook Alert");
	assert_eq!(trigger.trigger_type, "webhook");
}

#[tokio::test]
async fn test_create_trigger_quota_exceeded() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	let request = CreateTriggerRequest {
		trigger_id: "webhook-alert-2".to_string(),
		monitor_id: test_ids.monitor_1,
		name: "Too Many Triggers".to_string(),
		trigger_type: "webhook".to_string(),
		configuration: webhook_trigger_config(),
	};

	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::QuotaExceeded(
				"Trigger quota exceeded".to_string(),
			))
		});

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::QuotaExceeded(msg) => {
			assert_eq!(msg, "Trigger quota exceeded");
		}
		_ => panic!("Expected QuotaExceeded error"),
	}
}

#[tokio::test]
async fn test_get_trigger_by_id_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();
	let trigger_id = "trigger-123";

	let expected_trigger = TriggerBuilder::new()
		.with_id(test_ids.trigger_1)
		.with_name("Email Alert")
		.with_trigger_type("email")
		.build();

	let expected_trigger_clone = expected_trigger.clone();
	mock_repo
		.expect_get()
		.with(eq(trigger_id))
		.times(1)
		.returning(move |_| Ok(expected_trigger_clone.clone()));

	// Act
	let result = mock_repo.get(trigger_id).await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.name, "Email Alert");
	assert_eq!(trigger.trigger_type, "email");
}

#[tokio::test]
async fn test_get_trigger_by_id_not_found() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();

	mock_repo
		.expect_get()
		.with(eq("unknown-trigger"))
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "resource".to_string(),
				resource_id: "not-found".to_string(),
			})
		});

	// Act
	let result = mock_repo.get("unknown-trigger").await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ResourceNotFound {
			resource_type,
			resource_id,
		} => {
			assert_eq!(resource_type, "trigger");
			assert_eq!(resource_id, "unknown-trigger");
		}
		_ => panic!("Expected ResourceNotFound error"),
	}
}

#[tokio::test]
async fn test_get_trigger_by_uuid_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	let expected_trigger = TriggerBuilder::new().with_id(test_ids.trigger_1).build();

	let expected_trigger_clone = expected_trigger.clone();
	mock_repo
		.expect_get_by_uuid()
		.with(eq(test_ids.trigger_1))
		.times(1)
		.returning(move |_| Ok(expected_trigger_clone.clone()));

	// Act
	let result = mock_repo.get_by_uuid(test_ids.trigger_1).await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.id, test_ids.trigger_1);
}

#[tokio::test]
async fn test_get_all_triggers_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	let trigger1 = TriggerBuilder::new()
		.with_id(test_ids.trigger_1)
		.with_name("Webhook Alert")
		.build();

	let trigger2 = TriggerBuilder::new()
		.with_id(test_ids.trigger_2)
		.with_name("Email Alert")
		.build();

	let mut triggers = HashMap::new();
	triggers.insert(test_ids.trigger_1.to_string(), trigger1);
	triggers.insert(test_ids.trigger_2.to_string(), trigger2);

	let triggers_clone = triggers.clone();
	mock_repo
		.expect_get_all()
		.times(1)
		.returning(move || Ok(triggers_clone.clone()));

	// Act
	let result = mock_repo.get_all().await;

	// Assert
	assert!(result.is_ok());
	let returned_triggers = result.unwrap();
	assert_eq!(returned_triggers.len(), 2);
}

#[tokio::test]
async fn test_get_triggers_by_monitor_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	let triggers = vec![
		TriggerBuilder::new()
			.with_monitor_id(test_ids.monitor_1)
			.with_name("Webhook Alert")
			.build(),
		TriggerBuilder::new()
			.with_monitor_id(test_ids.monitor_1)
			.with_name("Email Alert")
			.build(),
		TriggerBuilder::new()
			.with_monitor_id(test_ids.monitor_1)
			.with_name("Slack Alert")
			.build(),
	];

	let triggers_clone = triggers.clone();
	mock_repo
		.expect_get_by_monitor()
		.with(eq(test_ids.monitor_1))
		.times(1)
		.returning(move |_| Ok(triggers_clone.clone()));

	// Act
	let result = mock_repo.get_by_monitor(test_ids.monitor_1).await;

	// Assert
	assert!(result.is_ok());
	let returned_triggers = result.unwrap();
	assert_eq!(returned_triggers.len(), 3);
	assert!(returned_triggers
		.iter()
		.all(|t| t.monitor_id == test_ids.monitor_1));
}

#[tokio::test]
async fn test_get_triggers_by_monitor_empty() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let monitor_id = Uuid::new_v4();

	mock_repo
		.expect_get_by_monitor()
		.with(eq(monitor_id))
		.times(1)
		.returning(|_| Ok(vec![]));

	// Act
	let result = mock_repo.get_by_monitor(monitor_id).await;

	// Assert
	assert!(result.is_ok());
	let returned_triggers = result.unwrap();
	assert!(returned_triggers.is_empty());
}

#[tokio::test]
async fn test_update_trigger_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();
	let trigger_id = "trigger-123";

	let update_request = UpdateTriggerRequest {
		name: Some("Updated Webhook Alert".to_string()),
		configuration: Some(webhook_trigger_config()),
		is_active: None,
	};

	let updated_trigger = TriggerBuilder::new()
		.with_id(test_ids.trigger_1)
		.with_name("Updated Webhook Alert")
		.with_active(false)
		.build();

	let updated_trigger_clone = updated_trigger.clone();
	mock_repo
		.expect_update()
		.with(eq(trigger_id), always())
		.times(1)
		.returning(move |_, _| Ok(updated_trigger_clone.clone()));

	// Act
	let result = mock_repo.update(trigger_id, update_request).await;

	// Assert
	assert!(result.is_ok());
	let trigger = result.unwrap();
	assert_eq!(trigger.name, "Updated Webhook Alert");
	assert_eq!(trigger.is_active, Some(false));
}

#[tokio::test]
async fn test_delete_trigger_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let trigger_id = "trigger-123";

	mock_repo
		.expect_delete()
		.with(eq(trigger_id))
		.times(1)
		.returning(|_| Ok(()));

	// Act
	let result = mock_repo.delete(trigger_id).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_triggers_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let triggers = vec![
		TriggerBuilder::new().with_name("Trigger 1").build(),
		TriggerBuilder::new().with_name("Trigger 2").build(),
	];

	let triggers_clone = triggers.clone();
	mock_repo
		.expect_list()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(move |_, _| Ok(triggers_clone.clone()));

	// Act
	let result = mock_repo.list(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_triggers = result.unwrap();
	assert_eq!(returned_triggers.len(), 2);
}

#[tokio::test]
async fn test_count_triggers_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();

	mock_repo.expect_count().times(1).returning(|| Ok(42));

	// Act
	let result = mock_repo.count().await;

	// Assert
	assert!(result.is_ok());
	assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_check_quota_for_monitor_success() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	mock_repo
		.expect_check_quota()
		.with(eq(test_ids.monitor_1))
		.times(1)
		.returning(|_| Ok(true));

	// Act
	let result = mock_repo.check_quota(test_ids.monitor_1).await;

	// Assert
	assert!(result.is_ok());
	assert!(result.unwrap());
}

#[tokio::test]
async fn test_check_quota_for_monitor_exceeded() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	mock_repo
		.expect_check_quota()
		.with(eq(test_ids.monitor_1))
		.times(1)
		.returning(|_| Ok(false));

	// Act
	let result = mock_repo.check_quota(test_ids.monitor_1).await;

	// Assert
	assert!(result.is_ok());
	assert!(!result.unwrap());
}

// Edge cases
#[tokio::test]
async fn test_create_multiple_trigger_types() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let test_ids = TestIds::default();

	// Test webhook trigger
	let webhook_request = CreateTriggerRequest {
		trigger_id: "trigger-webhook".to_string(),
		monitor_id: test_ids.monitor_1,
		name: "Webhook".to_string(),
		trigger_type: "webhook".to_string(),
		configuration: webhook_trigger_config(),
	};

	// Test email trigger
	let email_request = CreateTriggerRequest {
		trigger_id: "trigger-email".to_string(),
		monitor_id: test_ids.monitor_1,
		name: "Email".to_string(),
		trigger_type: "email".to_string(),
		configuration: email_trigger_config(),
	};

	// Test slack trigger
	let slack_request = CreateTriggerRequest {
		trigger_id: "trigger-slack".to_string(),
		monitor_id: test_ids.monitor_1,
		name: "Slack".to_string(),
		trigger_type: "slack".to_string(),
		configuration: slack_trigger_config(),
	};

	mock_repo.expect_create().times(3).returning(|req| {
		let trigger = TriggerBuilder::new()
			.with_name(&req.name)
			.with_trigger_type(&req.trigger_type)
			.build();
		Ok(trigger)
	});

	// Act
	let webhook_result = mock_repo.create(webhook_request).await;
	let email_result = mock_repo.create(email_request).await;
	let slack_result = mock_repo.create(slack_request).await;

	// Assert
	assert!(webhook_result.is_ok());
	assert!(email_result.is_ok());
	assert!(slack_result.is_ok());

	assert_eq!(webhook_result.unwrap().trigger_type, "webhook");
	assert_eq!(email_result.unwrap().trigger_type, "email");
	assert_eq!(slack_result.unwrap().trigger_type, "slack");
}

#[tokio::test]
async fn test_update_trigger_invalid_config() {
	// Arrange
	let mut mock_repo = MockTenantTriggerRepository::new();
	let trigger_id = "trigger-123";

	let update_request = UpdateTriggerRequest {
		name: None,
		configuration: Some(serde_json::json!({"invalid": "config"})),
		is_active: None,
	};

	mock_repo
		.expect_update()
		.with(eq(trigger_id), always())
		.times(1)
		.returning(|_, _| {
			Err(TenantRepositoryError::ValidationError(
				"Invalid trigger configuration".to_string(),
			))
		});

	// Act
	let result = mock_repo.update(trigger_id, update_request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ValidationError(msg) => {
			assert_eq!(msg, "Invalid trigger configuration");
		}
		_ => panic!("Expected ValidationError"),
	}
}
