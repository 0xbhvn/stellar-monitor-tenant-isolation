use mockall::predicate::*;
use std::collections::HashMap;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::{error::TenantRepositoryError, monitor::TenantMonitorRepositoryTrait},
};

use crate::{
	mocks::MockTenantMonitorRepository,
	utils::{
		builders::{CreateMonitorRequestBuilder, MonitorBuilder},
		fixtures::{evm_monitor_config, stellar_monitor_config, TestIds},
	},
};

#[tokio::test]
async fn test_create_monitor_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let test_ids = TestIds::default();

	let request = CreateMonitorRequestBuilder::new()
		.with_name("Transfer Monitor")
		.with_network_id(test_ids.network_1)
		.with_configuration(stellar_monitor_config())
		.build();

	let expected_monitor = MonitorBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_name("Transfer Monitor")
		.with_network_id(test_ids.network_1)
		.build();

	let expected_monitor_clone = expected_monitor.clone();
	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(move |_| Ok(expected_monitor_clone.clone()));

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.name, "Transfer Monitor");
	assert_eq!(monitor.tenant_id, test_ids.tenant_1);
}

#[tokio::test]
async fn test_create_monitor_quota_exceeded() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let request = CreateMonitorRequestBuilder::new().build();

	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::QuotaExceeded(
				"Monitor quota exceeded".to_string(),
			))
		});

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::QuotaExceeded(msg) => {
			assert_eq!(msg, "Monitor quota exceeded");
		}
		_ => panic!("Expected QuotaExceeded error"),
	}
}

#[tokio::test]
async fn test_get_monitor_by_id_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let test_ids = TestIds::default();
	let monitor_id = "monitor-123";

	let expected_monitor = MonitorBuilder::new()
		.with_id(test_ids.monitor_1)
		.with_monitor_id(monitor_id)
		.build();

	let expected_monitor_clone = expected_monitor.clone();
	mock_repo
		.expect_get()
		.with(eq(monitor_id))
		.times(1)
		.returning(move |_| Ok(expected_monitor_clone.clone()));

	// Act
	let result = mock_repo.get(monitor_id).await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.monitor_id, monitor_id);
}

#[tokio::test]
async fn test_get_monitor_by_id_not_found() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();

	mock_repo
		.expect_get()
		.with(eq("unknown-monitor"))
		.times(1)
		.returning(|id| {
			Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "monitor".to_string(),
				resource_id: id.to_string(),
			})
		});

	// Act
	let result = mock_repo.get("unknown-monitor").await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ResourceNotFound {
			resource_type,
			resource_id,
		} => {
			assert_eq!(resource_type, "monitor");
			assert_eq!(resource_id, "unknown-monitor");
		}
		_ => panic!("Expected ResourceNotFound error"),
	}
}

#[tokio::test]
async fn test_get_monitor_by_uuid_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let test_ids = TestIds::default();

	let expected_monitor = MonitorBuilder::new().with_id(test_ids.monitor_1).build();

	let expected_monitor_clone = expected_monitor.clone();
	mock_repo
		.expect_get_by_uuid()
		.with(eq(test_ids.monitor_1))
		.times(1)
		.returning(move |_| Ok(expected_monitor_clone.clone()));

	// Act
	let result = mock_repo.get_by_uuid(test_ids.monitor_1).await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.id, test_ids.monitor_1);
}

#[tokio::test]
async fn test_get_all_monitors_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let test_ids = TestIds::default();

	let monitor1 = MonitorBuilder::new()
		.with_id(test_ids.monitor_1)
		.with_monitor_id("monitor-1")
		.build();

	let monitor2 = MonitorBuilder::new()
		.with_id(test_ids.monitor_2)
		.with_monitor_id("monitor-2")
		.build();

	let mut monitors = HashMap::new();
	monitors.insert("monitor-1".to_string(), monitor1);
	monitors.insert("monitor-2".to_string(), monitor2);

	let monitors_clone = monitors.clone();
	mock_repo
		.expect_get_all()
		.times(1)
		.returning(move || Ok(monitors_clone.clone()));

	// Act
	let result = mock_repo.get_all().await;

	// Assert
	assert!(result.is_ok());
	let returned_monitors = result.unwrap();
	assert_eq!(returned_monitors.len(), 2);
	assert!(returned_monitors.contains_key("monitor-1"));
	assert!(returned_monitors.contains_key("monitor-2"));
}

#[tokio::test]
async fn test_update_monitor_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let test_ids = TestIds::default();
	let monitor_id = "monitor-123";

	let update_request = UpdateMonitorRequest {
		name: Some("Updated Monitor".to_string()),
		configuration: Some(evm_monitor_config()),
		is_active: Some(false),
	};

	let updated_monitor = MonitorBuilder::new()
		.with_id(test_ids.monitor_1)
		.with_monitor_id(monitor_id)
		.with_name("Updated Monitor")
		.with_active(false)
		.build();

	let updated_monitor_clone = updated_monitor.clone();
	mock_repo
		.expect_update()
		.with(eq(monitor_id), always())
		.times(1)
		.returning(move |_, _| Ok(updated_monitor_clone.clone()));

	// Act
	let result = mock_repo.update(monitor_id, update_request).await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.name, "Updated Monitor");
	assert!(!monitor.is_active.unwrap_or(true));
}

#[tokio::test]
async fn test_delete_monitor_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let monitor_id = "monitor-123";

	mock_repo
		.expect_delete()
		.with(eq(monitor_id))
		.times(1)
		.returning(|_| Ok(()));

	// Act
	let result = mock_repo.delete(monitor_id).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_monitor_not_found() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();

	mock_repo
		.expect_delete()
		.with(eq("unknown-monitor"))
		.times(1)
		.returning(|id| {
			Err(TenantRepositoryError::ResourceNotFound {
				resource_type: "monitor".to_string(),
				resource_id: id.to_string(),
			})
		});

	// Act
	let result = mock_repo.delete("unknown-monitor").await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ResourceNotFound {
			resource_type,
			resource_id,
		} => {
			assert_eq!(resource_type, "monitor");
			assert_eq!(resource_id, "unknown-monitor");
		}
		_ => panic!("Expected ResourceNotFound error"),
	}
}

#[tokio::test]
async fn test_list_monitors_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let monitors = vec![
		MonitorBuilder::new().with_name("Monitor 1").build(),
		MonitorBuilder::new().with_name("Monitor 2").build(),
		MonitorBuilder::new().with_name("Monitor 3").build(),
	];

	let monitors_clone = monitors.clone();
	mock_repo
		.expect_list()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(move |_, _| Ok(monitors_clone.clone()));

	// Act
	let result = mock_repo.list(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_monitors = result.unwrap();
	assert_eq!(returned_monitors.len(), 3);
}

#[tokio::test]
async fn test_list_monitors_with_pagination() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let monitors = vec![MonitorBuilder::new().with_name("Page 2 Monitor").build()];

	let monitors_clone = monitors.clone();
	mock_repo
		.expect_list()
		.with(eq(5i64), eq(5i64))
		.times(1)
		.returning(move |_, _| Ok(monitors_clone.clone()));

	// Act
	let result = mock_repo.list(5, 5).await;

	// Assert
	assert!(result.is_ok());
	let returned_monitors = result.unwrap();
	assert_eq!(returned_monitors.len(), 1);
}

#[tokio::test]
async fn test_check_quota_success() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();

	mock_repo
		.expect_check_quota()
		.times(1)
		.returning(|| Ok(true));

	// Act
	let result = mock_repo.check_quota().await;

	// Assert
	assert!(result.is_ok());
	assert!(result.unwrap());
}

#[tokio::test]
async fn test_check_quota_exceeded() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();

	mock_repo
		.expect_check_quota()
		.times(1)
		.returning(|| Ok(false));

	// Act
	let result = mock_repo.check_quota().await;

	// Assert
	assert!(result.is_ok());
	assert!(!result.unwrap());
}

// Edge cases and error scenarios
#[tokio::test]
async fn test_create_monitor_with_invalid_data() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();
	let request = CreateMonitorRequestBuilder::new()
        .with_name("") // Empty name
        .build();

	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::ValidationError(
				"Name cannot be empty".to_string(),
			))
		});

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ValidationError(msg) => {
			assert_eq!(msg, "Name cannot be empty");
		}
		_ => panic!("Expected ValidationError"),
	}
}

#[tokio::test]
async fn test_database_connection_error() {
	// Arrange
	let mut mock_repo = MockTenantMonitorRepository::new();

	mock_repo.expect_get_all().times(1).returning(|| {
		Err(TenantRepositoryError::Internal(
			"Connection timeout".to_string(),
		))
	});

	// Act
	let result = mock_repo.get_all().await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::Internal(msg) => {
			assert_eq!(msg, "Connection timeout");
		}
		_ => panic!("Expected Internal error"),
	}
}
