use mockall::predicate::*;
use std::collections::HashMap;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::{error::TenantRepositoryError, network::TenantNetworkRepositoryTrait},
};

use crate::{
	mocks::MockTenantNetworkRepository,
	utils::{
		builders::NetworkBuilder,
		fixtures::{evm_network_config, stellar_network_config, TestIds},
	},
};

#[tokio::test]
async fn test_create_network_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let test_ids = TestIds::default();

	let request = CreateNetworkRequest {
		network_id: "stellar-testnet".to_string(),
		name: "Stellar Testnet".to_string(),
		blockchain: "stellar".to_string(),
		configuration: stellar_network_config(),
	};

	let expected_network = NetworkBuilder::new()
		.with_tenant_id(test_ids.tenant_1)
		.with_original_id("stellar-testnet")
		.with_name("Stellar Testnet")
		.build();

	let expected_network_clone = expected_network.clone();
	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(move |_| Ok(expected_network_clone.clone()));

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.name, "Stellar Testnet");
	assert_eq!(network.network_id, "stellar-testnet");
}

#[tokio::test]
async fn test_create_network_quota_exceeded() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let _test_ids = TestIds::default();

	let request = CreateNetworkRequest {
		network_id: "network-id".to_string(),
		name: "Test Network".to_string(),
		blockchain: "evm".to_string(),
		configuration: evm_network_config(),
	};

	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::QuotaExceeded(
				"Network quota exceeded".to_string(),
			))
		});

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::QuotaExceeded(msg) => {
			assert_eq!(msg, "Network quota exceeded");
		}
		_ => panic!("Expected QuotaExceeded error"),
	}
}

#[tokio::test]
async fn test_get_network_by_id_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let test_ids = TestIds::default();
	let network_id = "stellar-testnet";

	let expected_network = NetworkBuilder::new()
		.with_id(test_ids.network_1)
		.with_original_id(network_id)
		.with_name("Stellar Testnet")
		.build();

	let expected_network_clone = expected_network.clone();
	mock_repo
		.expect_get()
		.with(eq(network_id))
		.times(1)
		.returning(move |_| Ok(expected_network_clone.clone()));

	// Act
	let result = mock_repo.get(network_id).await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.network_id, network_id);
	assert_eq!(network.name, "Stellar Testnet");
}

// #[tokio::test]
// async fn test_get_network_by_id_not_found() {
// 	// Arrange
// 	let mut mock_repo = MockTenantNetworkRepository::new();

// 	mock_repo
// 		.expect_get()
// 		.with(eq("unknown-network"))
// 		.times(1)
// 		.returning(|_| {
// 			Err(TenantRepositoryError::ResourceNotFound {
// 				resource_type: "resource".to_string(),
// 				resource_id: "not-found".to_string(),
// 			})
// 		});

// 	// Act
// 	let result = mock_repo.get("unknown-network").await;

// 	// Assert
// 	assert!(result.is_err());
// 	match result.unwrap_err() {
// 		TenantRepositoryError::ResourceNotFound {
// 			resource_type,
// 			resource_id,
// 		} => {
// 			assert_eq!(resource_type, "network");
// 			assert_eq!(resource_id, "unknown-network");
// 		}
// 		_ => panic!("Expected ResourceNotFound error"),
// 	}
// }

#[tokio::test]
async fn test_get_network_by_uuid_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let test_ids = TestIds::default();

	let expected_network = NetworkBuilder::new().with_id(test_ids.network_1).build();

	let expected_network_clone = expected_network.clone();
	mock_repo
		.expect_get_by_uuid()
		.with(eq(test_ids.network_1))
		.times(1)
		.returning(move |_| Ok(expected_network_clone.clone()));

	// Act
	let result = mock_repo.get_by_uuid(test_ids.network_1).await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.id, test_ids.network_1);
}

#[tokio::test]
async fn test_get_all_networks_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let test_ids = TestIds::default();

	let network1 = NetworkBuilder::new()
		.with_id(test_ids.network_1)
		.with_original_id("stellar-testnet")
		.with_name("Stellar Testnet")
		.build();

	let network2 = NetworkBuilder::new()
		.with_id(test_ids.network_2)
		.with_original_id("ethereum-mainnet")
		.with_name("Ethereum Mainnet")
		.build();

	let mut networks = HashMap::new();
	networks.insert("stellar-testnet".to_string(), network1);
	networks.insert("ethereum-mainnet".to_string(), network2);

	let networks_clone = networks.clone();
	mock_repo
		.expect_get_all()
		.times(1)
		.returning(move || Ok(networks_clone.clone()));

	// Act
	let result = mock_repo.get_all().await;

	// Assert
	assert!(result.is_ok());
	let returned_networks = result.unwrap();
	assert_eq!(returned_networks.len(), 2);
	assert!(returned_networks.contains_key("stellar-testnet"));
	assert!(returned_networks.contains_key("ethereum-mainnet"));
}

#[tokio::test]
async fn test_update_network_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let test_ids = TestIds::default();
	let network_id = "stellar-testnet";

	let update_request = UpdateNetworkRequest {
		name: Some("Updated Stellar Network".to_string()),
		configuration: Some(stellar_network_config()),
		is_active: Some(false),
	};

	let updated_network = NetworkBuilder::new()
		.with_id(test_ids.network_1)
		.with_original_id(network_id)
		.with_name("Updated Stellar Network")
		.with_active(false)
		.build();

	let updated_network_clone = updated_network.clone();
	mock_repo
		.expect_update()
		.with(eq(network_id), always())
		.times(1)
		.returning(move |_, _| Ok(updated_network_clone.clone()));

	// Act
	let result = mock_repo.update(network_id, update_request).await;

	// Assert
	assert!(result.is_ok());
	let network = result.unwrap();
	assert_eq!(network.name, "Updated Stellar Network");
	assert!(!network.is_active.unwrap_or(true));
}

#[tokio::test]
async fn test_delete_network_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let network_id = "stellar-testnet";

	mock_repo
		.expect_delete()
		.with(eq(network_id))
		.times(1)
		.returning(|_| Ok(()));

	// Act
	let result = mock_repo.delete(network_id).await;

	// Assert
	assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_network_with_monitors() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let network_id = "stellar-testnet";

	mock_repo
		.expect_delete()
		.with(eq(network_id))
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::ValidationError(
				"Cannot delete network with active monitors".to_string(),
			))
		});

	// Act
	let result = mock_repo.delete(network_id).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ValidationError(msg) => {
			assert_eq!(msg, "Cannot delete network with active monitors");
		}
		_ => panic!("Expected ValidationError error"),
	}
}

#[tokio::test]
async fn test_list_networks_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let networks = vec![
		NetworkBuilder::new().with_name("Network 1").build(),
		NetworkBuilder::new().with_name("Network 2").build(),
	];

	let networks_clone = networks.clone();
	mock_repo
		.expect_list()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(move |_, _| Ok(networks_clone.clone()));

	// Act
	let result = mock_repo.list(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_networks = result.unwrap();
	assert_eq!(returned_networks.len(), 2);
}

#[tokio::test]
async fn test_list_networks_empty() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();

	mock_repo
		.expect_list()
		.with(eq(10i64), eq(0i64))
		.times(1)
		.returning(|_, _| Ok(vec![]));

	// Act
	let result = mock_repo.list(10, 0).await;

	// Assert
	assert!(result.is_ok());
	let returned_networks = result.unwrap();
	assert!(returned_networks.is_empty());
}

#[tokio::test]
async fn test_check_quota_success() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();

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
	let mut mock_repo = MockTenantNetworkRepository::new();

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

// Edge cases
#[tokio::test]
async fn test_create_network_duplicate_original_id() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let _test_ids = TestIds::default();

	let request = CreateNetworkRequest {
		network_id: "existing-network".to_string(),
		name: "Test Network".to_string(),
		blockchain: "stellar".to_string(),
		configuration: stellar_network_config(),
	};

	mock_repo
		.expect_create()
		.with(always())
		.times(1)
		.returning(|_| {
			Err(TenantRepositoryError::AlreadyExists {
				resource_type: "Network".to_string(),
				resource_id: "stellar-testnet".to_string(),
			})
		});

	// Act
	let result = mock_repo.create(request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::AlreadyExists {
			resource_type,
			resource_id,
		} => {
			assert_eq!(resource_type, "Network");
			assert_eq!(resource_id, "stellar-testnet");
		}
		_ => panic!("Expected DuplicateResource error"),
	}
}

#[tokio::test]
async fn test_update_network_invalid_config() {
	// Arrange
	let mut mock_repo = MockTenantNetworkRepository::new();
	let network_id = "stellar-testnet";

	let update_request = UpdateNetworkRequest {
		name: None,
		configuration: Some(serde_json::json!({"invalid": "config"})),
		is_active: None,
	};

	mock_repo
		.expect_update()
		.with(eq(network_id), always())
		.times(1)
		.returning(|_, _| {
			Err(TenantRepositoryError::ValidationError(
				"Invalid network configuration".to_string(),
			))
		});

	// Act
	let result = mock_repo.update(network_id, update_request).await;

	// Assert
	assert!(result.is_err());
	match result.unwrap_err() {
		TenantRepositoryError::ValidationError(msg) => {
			assert_eq!(msg, "Invalid network configuration");
		}
		_ => panic!("Expected ValidationError"),
	}
}
