use mockall::predicate::*;
use stellar_monitor_tenant_isolation::{
	models::*,
	services::{NetworkServiceTrait, ServiceError},
};

use crate::{
	mocks::MockNetworkService,
	utils::{
		builders::{CreateNetworkRequestBuilder, NetworkBuilder},
		fixtures::{stellar_network_config, TestIds},
	},
};

#[tokio::test]
async fn test_create_network_success() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let test_ids = TestIds::default();

	let request = CreateNetworkRequestBuilder::new()
		.with_name("Stellar Mainnet")
		.with_blockchain("stellar")
		.with_configuration(stellar_network_config())
		.build();

	let expected_network = NetworkBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_name("Stellar Mainnet")
		.build();

	let expected_network_clone = expected_network.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_network()
		.with(always(), always())
		.times(1)
		.returning(move |_, _| Ok(expected_network_clone.clone()));

	// Act
	let result = mock_service.create_network(request, metadata).await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.name, "Stellar Mainnet");
}

#[tokio::test]
async fn test_create_network_quota_exceeded() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let request = CreateNetworkRequestBuilder::new().build();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_network()
		.with(always(), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::QuotaExceeded(
				"Network quota exceeded".to_string(),
			))
		});

	// Act
	let result = mock_service.create_network(request, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::QuotaExceeded(msg) => assert_eq!(msg, "Network quota exceeded"),
		_ => panic!("Expected QuotaExceeded error"),
	}
}

#[tokio::test]
async fn test_get_network_success() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let network_id = "stellar-mainnet";

	let expected_network = NetworkBuilder::new().with_network_id(network_id).build();

	let expected_network_clone = expected_network.clone();
	mock_service
		.expect_get_network()
		.with(eq(network_id))
		.times(1)
		.returning(move |_| Ok(expected_network_clone.clone()));

	// Act
	let result = mock_service.get_network(network_id).await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.network_id, network_id);
}

#[tokio::test]
async fn test_update_network_success() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let network_id = "stellar-mainnet";

	let update_request = UpdateNetworkRequest {
		name: Some("Updated Network".to_string()),
		configuration: None,
		is_active: Some(false),
	};

	let updated_network = NetworkBuilder::new()
		.with_network_id(network_id)
		.with_name("Updated Network")
		.with_active(false)
		.build();

	let updated_network_clone = updated_network.clone();
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_update_network()
		.with(eq(network_id), always(), always())
		.times(1)
		.returning(move |_, _, _| Ok(updated_network_clone.clone()));

	// Act
	let result = mock_service
		.update_network(network_id, update_request, metadata)
		.await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.name, "Updated Network");
	assert_eq!(network.is_active, Some(false));
}

#[tokio::test]
async fn test_delete_network_success() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let network_id = "stellar-mainnet";
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_delete_network()
		.with(eq(network_id), always())
		.times(1)
		.returning(|_, _| Ok(()));

	// Act
	let result = mock_service.delete_network(network_id, metadata).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_network_with_monitors() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let network_id = "network-with-monitors";
	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_delete_network()
		.with(eq(network_id), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::ValidationError(
				"Cannot delete network with active monitors".to_string(),
			))
		});

	// Act
	let result = mock_service.delete_network(network_id, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::ValidationError(msg) => {
			assert_eq!(msg, "Cannot delete network with active monitors")
		}
		_ => panic!("Expected ValidationError"),
	}
}

#[tokio::test]
async fn test_list_networks_success() {
	// Arrange
	let mut mock_service = MockNetworkService::new();

	let networks = vec![
		NetworkBuilder::new().with_name("Network 1").build(),
		NetworkBuilder::new().with_name("Network 2").build(),
	];

	let networks_clone = networks.clone();
	mock_service
		.expect_list_networks()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(move |_, _| Ok(networks_clone.clone()));

	// Act
	let result = mock_service.list_networks(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_networks = result.unwrap();
	assert_eq!(returned_networks.len(), 2);
}

#[tokio::test]
async fn test_network_validation_error() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let request = CreateNetworkRequestBuilder::new()
        .with_name("")  // Empty name
        .build();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	mock_service
		.expect_create_network()
		.with(always(), always())
		.times(1)
		.returning(|_, _| {
			Err(ServiceError::ValidationError(
				"Network name cannot be empty".to_string(),
			))
		});

	// Act
	let result = mock_service.create_network(request, metadata).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::ValidationError(msg) => assert_eq!(msg, "Network name cannot be empty"),
		_ => panic!("Expected ValidationError"),
	}
}

#[tokio::test]
async fn test_unauthorized_network_access() {
	// Arrange
	let mut mock_service = MockNetworkService::new();
	let network_id = "private-network";

	mock_service
		.expect_get_network()
		.with(eq(network_id))
		.times(1)
		.returning(|_| {
			Err(ServiceError::AccessDenied(
				"Unauthorized access to network".to_string(),
			))
		});

	// Act
	let result = mock_service.get_network(network_id).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		ServiceError::AccessDenied(msg) => assert_eq!(msg, "Unauthorized access to network"),
		_ => panic!("Expected AccessDenied error"),
	}
}
