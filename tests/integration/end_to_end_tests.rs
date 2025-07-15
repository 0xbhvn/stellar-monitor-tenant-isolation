use sqlx::PgPool;
use std::sync::Arc;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::tenant::TenantRepositoryTrait,
	services::{
		AuditService, MonitorService, MonitorServiceTrait, NetworkService, NetworkServiceTrait,
		TriggerService, TriggerServiceTrait,
	},
};

/// Test the complete lifecycle of creating a tenant and all its resources
#[sqlx::test]
async fn test_complete_tenant_lifecycle(pool: PgPool) {
	let audit_service = Arc::new(AuditService::new(pool.clone()));

	// Step 1: Create a tenant
	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());

	let create_tenant_req = CreateTenantRequest {
		name: "Integration Test Company".to_string(),
		slug: "integration-test".to_string(),
		max_monitors: Some(5),
		max_networks: Some(3),
		max_triggers_per_monitor: Some(3),
		max_rpc_requests_per_minute: Some(100),
		max_storage_mb: Some(500),
	};

	let tenant = tenant_repo.create(create_tenant_req).await.unwrap();
	assert_eq!(tenant.name, "Integration Test Company");

	// Step 2: Create a network
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);
	let network_service = NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let create_network_req = CreateNetworkRequest {
		network_id: "stellar-testnet".to_string(),
		name: "Stellar Testnet".to_string(),
		blockchain: "stellar".to_string(),
		configuration: serde_json::json!({
			"rpc_url": "https://horizon-testnet.stellar.org",
			"network_passphrase": "Test SDF Network ; September 2015"
		}),
	};

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	let network = network_service
		.create_network(create_network_req, metadata.clone())
		.await
		.unwrap();
	assert_eq!(network.name, "Stellar Testnet");

	// Step 3: Create a monitor
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);
	let monitor_service = MonitorService::new(
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let create_monitor_req = CreateMonitorRequest {
		monitor_id: "test-monitor-1".to_string(),
		name: "Payment Monitor".to_string(),
		network_id: network.id,
		configuration: serde_json::json!({
			"type": "contract_event",
			"contract_address": "GBVOL67TMUQBGL4TZYNMY3ZQ5WGQYFPFD5VJRWXR72VA33VFNL225PL5",
			"event_name": "Transfer",
			"filters": []
		}),
	};

	let monitor = monitor_service
		.create_monitor(create_monitor_req, metadata.clone())
		.await
		.unwrap();
	assert_eq!(monitor.name, "Payment Monitor");

	// Step 4: Create triggers
	let trigger_repo =
		stellar_monitor_tenant_isolation::repositories::trigger::TenantTriggerRepository::new(
			pool.clone(),
		);
	let trigger_service = TriggerService::new(
		trigger_repo.clone(),
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let create_webhook_trigger = CreateTriggerRequest {
		trigger_id: "webhook-trigger-1".to_string(),
		monitor_id: monitor.id,
		name: "Webhook Notification".to_string(),
		trigger_type: "webhook".to_string(),
		configuration: serde_json::json!({
			"url": "https://webhook.site/test",
			"method": "POST",
			"headers": {
				"Authorization": "Bearer test-token"
			}
		}),
	};

	let webhook_trigger = trigger_service
		.create_trigger(create_webhook_trigger, metadata.clone())
		.await
		.unwrap();
	assert_eq!(webhook_trigger.name, "Webhook Notification");

	// Step 5: List all resources
	let monitors = monitor_service.list_monitors(10, 0).await.unwrap();
	assert_eq!(monitors.len(), 1);

	let networks = network_service.list_networks(10, 0).await.unwrap();
	assert_eq!(networks.len(), 1);

	let triggers = trigger_service
		.list_triggers_by_monitor(monitor.id)
		.await
		.unwrap();
	assert_eq!(triggers.len(), 1);

	// Step 6: Check quota status
	let quota_status = tenant_repo.get_quota_status(tenant.id).await.unwrap();
	assert_eq!(quota_status.usage.monitors_count, 1);
	assert_eq!(quota_status.usage.networks_count, 1);
	assert_eq!(quota_status.usage.triggers_count, 1);

	// Step 7: Clean up - delete everything
	trigger_service
		.delete_trigger(&webhook_trigger.trigger_id, metadata.clone())
		.await
		.unwrap();
	monitor_service
		.delete_monitor(&monitor.monitor_id, metadata.clone())
		.await
		.unwrap();
	network_service
		.delete_network(&network.network_id, metadata.clone())
		.await
		.unwrap();
	tenant_repo.delete(tenant.id).await.unwrap();

	// Verify deletion
	let deleted_tenant = tenant_repo.get(tenant.id).await;
	assert!(deleted_tenant.is_err());
}

/// Test creating resources across multiple tenants
#[sqlx::test]
async fn test_multi_tenant_resource_creation(pool: PgPool) {
	let audit_service = Arc::new(AuditService::new(pool.clone()));

	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);

	let network_service = NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);
	let monitor_service = MonitorService::new(
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create two tenants
	let tenant1 = tenant_repo
		.create(CreateTenantRequest {
			name: "Tenant 1".to_string(),
			slug: "tenant-1".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		})
		.await
		.unwrap();

	let tenant2 = tenant_repo
		.create(CreateTenantRequest {
			name: "Tenant 2".to_string(),
			slug: "tenant-2".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		})
		.await
		.unwrap();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	// Create networks for each tenant
	let network1 = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "tenant1-network".to_string(),
				name: "Tenant 1 Network".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test1.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	let network2 = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "tenant2-network".to_string(),
				name: "Tenant 2 Network".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test2.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create monitors for each tenant
	let monitor1 = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "tenant1-monitor".to_string(),
				name: "Tenant 1 Monitor".to_string(),
				network_id: network1.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	let monitor2 = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "tenant2-monitor".to_string(),
				name: "Tenant 2 Monitor".to_string(),
				network_id: network2.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Verify each tenant has their own resources
	assert_eq!(monitor1.tenant_id, tenant1.id);
	assert_eq!(monitor2.tenant_id, tenant2.id);
	assert_ne!(monitor1.tenant_id, monitor2.tenant_id);

	// Check quotas for both tenants
	let quota1 = tenant_repo.get_quota_status(tenant1.id).await.unwrap();
	let quota2 = tenant_repo.get_quota_status(tenant2.id).await.unwrap();

	assert_eq!(quota1.usage.monitors_count, 1);
	assert_eq!(quota2.usage.monitors_count, 1);
}

/// Test monitor lifecycle with state changes
#[sqlx::test]
async fn test_monitor_state_management(pool: PgPool) {
	let audit_service = Arc::new(AuditService::new(pool.clone()));

	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);

	let network_service = NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);
	let monitor_service = MonitorService::new(
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create tenant and network
	let _tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "State Test Tenant".to_string(),
			slug: "state-test".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		})
		.await
		.unwrap();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	let network = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "state-test-network".to_string(),
				name: "State Test Network".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create monitor (starts active by default)
	let monitor = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "state-test-monitor".to_string(),
				name: "State Test Monitor".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	assert_eq!(monitor.is_active, Some(true));

	// Deactivate monitor
	let updated_monitor = monitor_service
		.update_monitor(
			&monitor.monitor_id,
			UpdateMonitorRequest {
				name: None,
				configuration: None,
				is_active: Some(false),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	assert_eq!(updated_monitor.is_active, Some(false));

	// Reactivate monitor
	let reactivated_monitor = monitor_service
		.update_monitor(
			&monitor.monitor_id,
			UpdateMonitorRequest {
				name: None,
				configuration: None,
				is_active: Some(true),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	assert_eq!(reactivated_monitor.is_active, Some(true));

	// Update configuration
	let reconfigured_monitor = monitor_service
		.update_monitor(
			&monitor.monitor_id,
			UpdateMonitorRequest {
				name: Some("Updated Monitor Name".to_string()),
				configuration: Some(serde_json::json!({"type": "updated", "version": 2})),
				is_active: None,
			},
			metadata,
		)
		.await
		.unwrap();

	assert_eq!(reconfigured_monitor.name, "Updated Monitor Name");
	assert_eq!(reconfigured_monitor.configuration["version"], 2);
}
