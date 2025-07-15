use sqlx::PgPool;
use std::sync::Arc;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::tenant::TenantRepositoryTrait,
	services::{
		AuditService, MonitorService, MonitorServiceTrait, NetworkService, NetworkServiceTrait,
		ServiceError, TriggerService, TriggerServiceTrait,
	},
};

/// Test monitor quota enforcement
#[sqlx::test]
async fn test_monitor_quota_enforcement(pool: PgPool) {
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

	// Create tenant with limited monitor quota
	let tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Limited Monitors Tenant".to_string(),
			slug: "limited-monitors".to_string(),
			max_monitors: Some(2), // Only allow 2 monitors
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

	// Create a network
	let network = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "quota-test-network".to_string(),
				name: "Quota Test Network".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create first monitor - should succeed
	let monitor1 = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "monitor-1".to_string(),
				name: "Monitor 1".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create second monitor - should succeed
	let _monitor2 = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "monitor-2".to_string(),
				name: "Monitor 2".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create third monitor - should fail due to quota
	let monitor3_result = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "monitor-3".to_string(),
				name: "Monitor 3".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await;

	assert!(monitor3_result.is_err());
	match monitor3_result.unwrap_err() {
		ServiceError::QuotaExceeded(_) => (), // Expected
		_ => panic!("Expected QuotaExceeded error"),
	}

	// Check quota status
	let quota_status = tenant_repo.get_quota_status(tenant.id).await.unwrap();
	assert_eq!(quota_status.usage.monitors_count, 2);
	assert_eq!(quota_status.quotas.max_monitors, 2);
	assert_eq!(quota_status.available.monitors, 0);

	// Delete one monitor
	monitor_service
		.delete_monitor(&monitor1.monitor_id, metadata.clone())
		.await
		.unwrap();

	// Now creating a new monitor should succeed
	let monitor3 = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "monitor-3".to_string(),
				name: "Monitor 3".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata,
		)
		.await
		.unwrap();

	assert_eq!(monitor3.name, "Monitor 3");
}

/// Test network quota enforcement
#[sqlx::test]
async fn test_network_quota_enforcement(pool: PgPool) {
	let audit_service = Arc::new(AuditService::new(pool.clone()));

	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);

	let network_service = NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create tenant with limited network quota
	let _tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Limited Networks Tenant".to_string(),
			slug: "limited-networks".to_string(),
			max_monitors: Some(10),
			max_networks: Some(1), // Only allow 1 network
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

	// Create first network - should succeed
	let _network1 = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "network-1".to_string(),
				name: "Network 1".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test1.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create second network - should fail due to quota
	let network2_result = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "network-2".to_string(),
				name: "Network 2".to_string(),
				blockchain: "ethereum".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test2.ethereum.org"}),
			},
			metadata,
		)
		.await;

	assert!(network2_result.is_err());
	match network2_result.unwrap_err() {
		ServiceError::QuotaExceeded(_) => (), // Expected
		_ => panic!("Expected QuotaExceeded error"),
	}
}

/// Test trigger quota enforcement per monitor
#[sqlx::test]
async fn test_trigger_quota_per_monitor(pool: PgPool) {
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
	let trigger_repo =
		stellar_monitor_tenant_isolation::repositories::trigger::TenantTriggerRepository::new(
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
	let trigger_service = TriggerService::new(
		trigger_repo.clone(),
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create tenant with limited triggers per monitor
	let _tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Limited Triggers Tenant".to_string(),
			slug: "limited-triggers".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(2), // Only 2 triggers per monitor
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		})
		.await
		.unwrap();

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	// Create network and monitor
	let network = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "trigger-quota-network".to_string(),
				name: "Trigger Quota Network".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	let monitor = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "trigger-quota-monitor".to_string(),
				name: "Trigger Quota Monitor".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create first trigger - should succeed
	let trigger1 = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "trigger-1".to_string(),
				monitor_id: monitor.id,
				name: "Trigger 1".to_string(),
				trigger_type: "webhook".to_string(),
				configuration: serde_json::json!({"url": "https://webhook1.test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create second trigger - should succeed
	let _trigger2 = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "trigger-2".to_string(),
				monitor_id: monitor.id,
				name: "Trigger 2".to_string(),
				trigger_type: "email".to_string(),
				configuration: serde_json::json!({"to": "test@example.com"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create third trigger - should fail due to quota
	let trigger3_result = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "trigger-3".to_string(),
				monitor_id: monitor.id,
				name: "Trigger 3".to_string(),
				trigger_type: "slack".to_string(),
				configuration: serde_json::json!({"webhook_url": "https://slack.test"}),
			},
			metadata.clone(),
		)
		.await;

	assert!(trigger3_result.is_err());
	match trigger3_result.unwrap_err() {
		ServiceError::QuotaExceeded(_) => (), // Expected
		_ => panic!("Expected QuotaExceeded error"),
	}

	// Delete one trigger
	trigger_service
		.delete_trigger(&trigger1.trigger_id, metadata.clone())
		.await
		.unwrap();

	// Now creating a new trigger should succeed
	let trigger3 = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "trigger-3".to_string(),
				monitor_id: monitor.id,
				name: "Trigger 3".to_string(),
				trigger_type: "slack".to_string(),
				configuration: serde_json::json!({"webhook_url": "https://slack.test"}),
			},
			metadata,
		)
		.await
		.unwrap();

	assert_eq!(trigger3.name, "Trigger 3");
}

/// Test storage quota enforcement
#[sqlx::test]
async fn test_storage_quota_tracking(pool: PgPool) {
	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());

	// Create tenant with limited storage
	let tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Limited Storage Tenant".to_string(),
			slug: "limited-storage".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(100), // Only 100 MB storage
		})
		.await
		.unwrap();

	// Check initial storage usage
	let quota_status = tenant_repo.get_quota_status(tenant.id).await.unwrap();
	assert_eq!(quota_status.quotas.max_storage_mb, 100);
	assert!(quota_status.usage.storage_mb_used < 10); // Should be minimal initially

	// Simulate storage usage (in a real system, this would be tracked as data is stored)
	// For now, we just verify the quota structure is correct
	assert!(quota_status.available.storage_mb > 90);
}

/// Test quota status aggregation
#[sqlx::test]
async fn test_quota_status_aggregation(pool: PgPool) {
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
	let trigger_repo =
		stellar_monitor_tenant_isolation::repositories::trigger::TenantTriggerRepository::new(
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
	let trigger_service = TriggerService::new(
		trigger_repo.clone(),
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create tenant
	let tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Quota Aggregation Tenant".to_string(),
			slug: "quota-aggregation".to_string(),
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

	// Create multiple resources
	let network1 = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "agg-network-1".to_string(),
				name: "Aggregation Network 1".to_string(),
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
				network_id: "agg-network-2".to_string(),
				name: "Aggregation Network 2".to_string(),
				blockchain: "ethereum".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://test2.ethereum.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	let monitor1 = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "agg-monitor-1".to_string(),
				name: "Aggregation Monitor 1".to_string(),
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
				monitor_id: "agg-monitor-2".to_string(),
				name: "Aggregation Monitor 2".to_string(),
				network_id: network2.id,
				configuration: serde_json::json!({"type": "test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	let _trigger1 = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "agg-trigger-1".to_string(),
				monitor_id: monitor1.id,
				name: "Aggregation Trigger 1".to_string(),
				trigger_type: "webhook".to_string(),
				configuration: serde_json::json!({"url": "https://webhook.test"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	let _trigger2 = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "agg-trigger-2".to_string(),
				monitor_id: monitor2.id,
				name: "Aggregation Trigger 2".to_string(),
				trigger_type: "email".to_string(),
				configuration: serde_json::json!({"to": "test@example.com"}),
			},
			metadata,
		)
		.await
		.unwrap();

	// Check aggregated quota status
	let quota_status = tenant_repo.get_quota_status(tenant.id).await.unwrap();

	assert_eq!(quota_status.usage.networks_count, 2);
	assert_eq!(quota_status.usage.monitors_count, 2);
	assert_eq!(quota_status.usage.triggers_count, 2);

	assert_eq!(quota_status.available.networks, 3); // 5 - 2
	assert_eq!(quota_status.available.monitors, 8); // 10 - 2
	assert_eq!(quota_status.available.triggers, 8); // (5 * 2) - 2
}
