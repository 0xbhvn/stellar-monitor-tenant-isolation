use mockall::predicate::*;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::error::TenantRepositoryError,
	services::{MonitorServiceTrait, ServiceError},
};

use crate::{
	mocks::MockMonitorService,
	utils::{
		builders::{CreateMonitorRequestBuilder, MonitorBuilder},
		fixtures::{stellar_monitor_config, TestIds},
	},
};

#[tokio::test]
async fn test_create_monitor_success() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let test_ids = TestIds::default();

	let request = CreateMonitorRequestBuilder::new()
		.with_name("New Monitor")
		.with_network_id(test_ids.network_1)
		.with_configuration(stellar_monitor_config())
		.build();

	let expected_monitor = MonitorBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_name("New Monitor")
		.with_network_id(test_ids.network_1)
		.build();

	let expected_monitor_clone = expected_monitor.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_monitor()
		.with(always(), always())
		.times(1)
		.returning(move |_, _| Ok(expected_monitor_clone.clone()));

	// Act
	let result = mock_service.create_monitor(request, metadata).await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.name, "New Monitor");
}

#[tokio::test]
async fn test_create_monitor_quota_exceeded() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let request = CreateMonitorRequestBuilder::new().build();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_monitor()
		.with(always(), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::QuotaExceeded(
				"Monitor quota exceeded".to_string(),
			))
		});

	// Act
	let result = mock_service.create_monitor(request, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::QuotaExceeded(msg) => assert_eq!(msg, "Monitor quota exceeded"),
		_ => panic!("Expected QuotaExceeded error"),
	}
}

#[tokio::test]
async fn test_get_monitor_success() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let monitor_id = "monitor-123";

	let expected_monitor = MonitorBuilder::new().with_monitor_id(monitor_id).build();

	let expected_monitor_clone = expected_monitor.clone();
	mock_service
		.expect_get_monitor()
		.with(eq(monitor_id))
		.times(1)
		.returning(move |_| Ok(expected_monitor_clone.clone()));

	// Act
	let result = mock_service.get_monitor(monitor_id).await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.monitor_id, monitor_id);
}

#[tokio::test]
async fn test_get_monitor_not_found() {
	// Arrange
	let mut mock_service = MockMonitorService::new();

	mock_service
		.expect_get_monitor()
		.with(eq("unknown-monitor"))
		.times(1)
		.returning(|_| {
			Err(ServiceError::Repository(
				TenantRepositoryError::ResourceNotFound {
					resource_type: "monitor".to_string(),
					resource_id: "unknown-monitor".to_string(),
				},
			))
		});

	// Act
	let result = mock_service.get_monitor("unknown-monitor").await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::Repository(TenantRepositoryError::ResourceNotFound {
			resource_type,
			resource_id,
		}) => {
			assert_eq!(resource_type, "monitor");
			assert_eq!(resource_id, "unknown-monitor");
		}
		_ => panic!("Expected Repository(ResourceNotFound) error"),
	}
}

#[tokio::test]
async fn test_update_monitor_success() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let monitor_id = "monitor-123";

	let update_request = UpdateMonitorRequest {
		name: Some("Updated Monitor".to_string()),
		configuration: None,
		is_active: Some(false),
	};

	let updated_monitor = MonitorBuilder::new()
		.with_monitor_id(monitor_id)
		.with_name("Updated Monitor")
		.with_active(false)
		.build();

	let updated_monitor_clone = updated_monitor.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_update_monitor()
		.with(eq(monitor_id), always(), always())
		.times(1)
		.returning(move |_, _, _| Ok(updated_monitor_clone.clone()));

	// Act
	let result = mock_service
		.update_monitor(monitor_id, update_request, metadata)
		.await;

	// Assert
	assert!(result.is_ok());
	let monitor = result.unwrap();
	assert_eq!(monitor.name, "Updated Monitor");
	assert_eq!(monitor.is_active, Some(false));
}

#[tokio::test]
async fn test_delete_monitor_success() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let monitor_id = "monitor-123";
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_delete_monitor()
		.with(eq(monitor_id), always())
		.times(1)
		.returning(|_, _| Ok(()));

	// Act
	let result = mock_service.delete_monitor(monitor_id, metadata).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_monitor_with_triggers() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let monitor_id = "monitor-with-triggers";
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_delete_monitor()
		.with(eq(monitor_id), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::ValidationError(
				"Cannot delete monitor with active triggers".to_string(),
			))
		});

	// Act
	let result = mock_service.delete_monitor(monitor_id, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::ValidationError(msg) => {
			assert_eq!(msg, "Cannot delete monitor with active triggers")
		}
		_ => panic!("Expected ValidationError"),
	}
}

#[tokio::test]
async fn test_list_monitors_with_pagination() {
	// Arrange
	let mut mock_service = MockMonitorService::new();

	let monitors = vec![
		MonitorBuilder::new().with_name("Monitor 1").build(),
		MonitorBuilder::new().with_name("Monitor 2").build(),
	];

	let monitors_clone = monitors.clone();
	mock_service
		.expect_list_monitors()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(move |_, _| Ok(monitors_clone.clone()));

	// Act
	let result = mock_service.list_monitors(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_monitors = result.unwrap();
	assert_eq!(returned_monitors.len(), 2);
}

#[tokio::test]
async fn test_unauthorized_access() {
	// Arrange
	let mut mock_service = MockMonitorService::new();
	let monitor_id = "monitor-123";

	mock_service
		.expect_get_monitor()
		.with(eq(monitor_id))
		.times(1)
		.returning(|_| {
			Err(ServiceError::AccessDenied(
				"Unauthorized access to monitor".to_string(),
			))
		});

	// Act
	let result = mock_service.get_monitor(monitor_id).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::AccessDenied(msg) => assert_eq!(msg, "Unauthorized access to monitor"),
		_ => panic!("Expected AccessDenied error"),
	}
}
