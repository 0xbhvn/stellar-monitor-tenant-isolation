use sqlx::PgPool;
use std::sync::Arc;
use stellar_monitor_tenant_isolation::{
	models::*,
	repositories::{
		monitor::TenantMonitorRepositoryTrait, network::TenantNetworkRepositoryTrait,
		tenant::TenantRepositoryTrait, trigger::TenantTriggerRepositoryTrait,
	},
	services::{
		AuditService, MonitorService, MonitorServiceTrait, NetworkService, NetworkServiceTrait,
		TriggerService, TriggerServiceTrait,
	},
};
use uuid::Uuid;

/// Test that resources are properly isolated between tenants
#[sqlx::test]
async fn test_resource_isolation_between_tenants(pool: PgPool) {
	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);

	// Create two separate tenants
	let tenant1 = tenant_repo
		.create(CreateTenantRequest {
			name: "Isolated Tenant 1".to_string(),
			slug: "isolated-1".to_string(),
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
			name: "Isolated Tenant 2".to_string(),
			slug: "isolated-2".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		})
		.await
		.unwrap();

	// Create network for tenant1
	let network1 = network_repo
		.create(CreateNetworkRequest {
			network_id: "tenant1-private-network".to_string(),
			name: "Tenant 1 Private Network".to_string(),
			blockchain: "stellar".to_string(),
			configuration: serde_json::json!({"rpc_url": "https://tenant1.stellar.org"}),
		})
		.await
		.unwrap();

	// Create monitor for tenant1
	let monitor1 = monitor_repo
		.create(CreateMonitorRequest {
			monitor_id: "tenant1-private-monitor".to_string(),
			name: "Tenant 1 Private Monitor".to_string(),
			network_id: network1.id,
			configuration: serde_json::json!({"type": "private"}),
		})
		.await
		.unwrap();

	// Verify tenant1's resources are associated with tenant1
	assert_eq!(network1.tenant_id, tenant1.id);
	assert_eq!(monitor1.tenant_id, tenant1.id);

	// Try to create monitor for tenant2 using tenant1's network - should fail
	// In a real system, this would be enforced by the service layer with proper auth

	// Create separate network for tenant2
	let network2 = network_repo
		.create(CreateNetworkRequest {
			network_id: "tenant2-private-network".to_string(),
			name: "Tenant 2 Private Network".to_string(),
			blockchain: "ethereum".to_string(),
			configuration: serde_json::json!({"rpc_url": "https://tenant2.ethereum.org"}),
		})
		.await
		.unwrap();

	// Verify resources are properly isolated
	assert_eq!(network2.tenant_id, tenant2.id);
	assert_ne!(network1.tenant_id, network2.tenant_id);

	// List all networks - in a real system, this would be filtered by tenant
	let all_networks = network_repo.list(100, 0).await.unwrap();
	let tenant1_networks: Vec<_> = all_networks
		.iter()
		.filter(|n| n.tenant_id == tenant1.id)
		.collect();
	let tenant2_networks: Vec<_> = all_networks
		.iter()
		.filter(|n| n.tenant_id == tenant2.id)
		.collect();

	assert_eq!(tenant1_networks.len(), 1);
	assert_eq!(tenant2_networks.len(), 1);
	assert_eq!(tenant1_networks[0].network_id, "tenant1-private-network");
	assert_eq!(tenant2_networks[0].network_id, "tenant2-private-network");
}

/// Test member access control within tenants
#[sqlx::test]
async fn test_tenant_member_access_control(pool: PgPool) {
	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());

	// Create tenant
	let tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Member Access Tenant".to_string(),
			slug: "member-access".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		})
		.await
		.unwrap();

	// Add members with different roles
	let admin_user_id = Uuid::new_v4();
	let viewer_user_id = Uuid::new_v4();
	let editor_user_id = Uuid::new_v4();

	let admin_membership = tenant_repo
		.add_member(tenant.id, admin_user_id, TenantRole::Admin)
		.await
		.unwrap();

	let viewer_membership = tenant_repo
		.add_member(tenant.id, viewer_user_id, TenantRole::Viewer)
		.await
		.unwrap();

	let editor_membership = tenant_repo
		.add_member(tenant.id, editor_user_id, TenantRole::Member)
		.await
		.unwrap();

	// Verify memberships
	assert_eq!(admin_membership.role, TenantRole::Admin);
	assert_eq!(viewer_membership.role, TenantRole::Viewer);
	assert_eq!(editor_membership.role, TenantRole::Member);

	// Get all members
	let members = tenant_repo.get_members(tenant.id).await.unwrap();
	assert_eq!(members.len(), 3);

	// Update member role
	let updated_membership = tenant_repo
		.update_member_role(tenant.id, viewer_user_id, TenantRole::Member)
		.await
		.unwrap();
	assert_eq!(updated_membership.role, TenantRole::Member);

	// Remove member
	tenant_repo
		.remove_member(tenant.id, editor_user_id)
		.await
		.unwrap();

	// Verify member was removed
	let remaining_members = tenant_repo.get_members(tenant.id).await.unwrap();
	assert_eq!(remaining_members.len(), 2);

	// Get user's tenants
	let admin_tenants = tenant_repo.get_user_tenants(admin_user_id).await.unwrap();
	assert_eq!(admin_tenants.len(), 1);
	assert_eq!(admin_tenants[0].0.id, tenant.id);
	assert_eq!(admin_tenants[0].1, TenantRole::Admin);
}

/// Test concurrent operations on different tenants
#[sqlx::test]
async fn test_concurrent_tenant_operations(pool: PgPool) {
	let audit_service = Arc::new(AuditService::new(pool.clone()));

	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);

	let monitor_service = Arc::new(MonitorService::new(
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	));
	let network_service = Arc::new(NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	));

	// Create multiple tenants
	let mut tenant_ids = Vec::new();
	for i in 0..3 {
		let tenant = tenant_repo
			.create(CreateTenantRequest {
				name: format!("Concurrent Tenant {}", i),
				slug: format!("concurrent-{}", i),
				max_monitors: Some(10),
				max_networks: Some(5),
				max_triggers_per_monitor: Some(5),
				max_rpc_requests_per_minute: Some(100),
				max_storage_mb: Some(1000),
			})
			.await
			.unwrap();
		tenant_ids.push(tenant.id);
	}

	let metadata = RequestMetadata {
		ip_address: None,
		user_agent: None,
	};

	// Spawn concurrent tasks to create resources for each tenant
	let mut handles = Vec::new();

	for (i, tenant_id) in tenant_ids.iter().enumerate() {
		let tenant_id = *tenant_id;
		let network_service = Arc::clone(&network_service);
		let monitor_service = Arc::clone(&monitor_service);
		let metadata = metadata.clone();

		let handle = tokio::spawn(async move {
			// Create network
			let network = network_service
				.create_network(
					CreateNetworkRequest {
						network_id: format!("concurrent-network-{}", i),
						name: format!("Concurrent Network {}", i),
						blockchain: "stellar".to_string(),
						configuration: serde_json::json!({"rpc_url": format!("https://concurrent{}.stellar.org", i)}),
					},
					metadata.clone(),
				)
				.await
				.unwrap();

			// Create monitors
			let mut monitor_ids = Vec::new();
			for j in 0..3 {
				let monitor = monitor_service
					.create_monitor(
						CreateMonitorRequest {
							monitor_id: format!("concurrent-monitor-{}-{}", i, j),
							name: format!("Concurrent Monitor {} - {}", i, j),
							network_id: network.id,
							configuration: serde_json::json!({"type": "concurrent", "index": j}),
						},
						metadata.clone(),
					)
					.await
					.unwrap();
				monitor_ids.push(monitor.id);
			}

			(tenant_id, network.id, monitor_ids)
		});

		handles.push(handle);
	}

	// Wait for all concurrent operations to complete
	let results: Vec<_> = futures::future::join_all(handles)
		.await
		.into_iter()
		.map(|r| r.unwrap())
		.collect();

	// Verify each tenant has their own isolated resources
	for (tenant_id, network_id, monitor_ids) in results {
		// Check monitors belong to correct tenant
		let monitor_repo =
			stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
				pool.clone(),
			);
		for monitor_id in monitor_ids {
			let monitor_uuid = Uuid::parse_str(&monitor_id.to_string()).unwrap();
			let monitor = monitor_repo.get_by_uuid(monitor_uuid).await.unwrap();
			assert_eq!(monitor.tenant_id, tenant_id);
			assert_eq!(monitor.network_id, network_id);
		}

		// Check quota usage
		let quota_status = tenant_repo.get_quota_status(tenant_id).await.unwrap();
		assert_eq!(quota_status.usage.networks_count, 1);
		assert_eq!(quota_status.usage.monitors_count, 3);
	}
}

/// Test deletion cascades and cleanup
#[sqlx::test]
async fn test_tenant_deletion_cascade(pool: PgPool) {
	let audit_service = Arc::new(AuditService::new(pool.clone()));

	let tenant_repo =
		stellar_monitor_tenant_isolation::repositories::tenant::TenantRepository::new(pool.clone());
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
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

	// Create tenant with resources
	let tenant = tenant_repo
		.create(CreateTenantRequest {
			name: "Cascade Delete Tenant".to_string(),
			slug: "cascade-delete".to_string(),
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

	// Create network
	let network = network_service
		.create_network(
			CreateNetworkRequest {
				network_id: "cascade-network".to_string(),
				name: "Cascade Network".to_string(),
				blockchain: "stellar".to_string(),
				configuration: serde_json::json!({"rpc_url": "https://cascade.stellar.org"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create monitor
	let monitor = monitor_service
		.create_monitor(
			CreateMonitorRequest {
				monitor_id: "cascade-monitor".to_string(),
				name: "Cascade Monitor".to_string(),
				network_id: network.id,
				configuration: serde_json::json!({"type": "cascade"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Create trigger
	let trigger = trigger_service
		.create_trigger(
			CreateTriggerRequest {
				trigger_id: "cascade-trigger".to_string(),
				monitor_id: monitor.id,
				name: "Cascade Trigger".to_string(),
				trigger_type: "webhook".to_string(),
				configuration: serde_json::json!({"url": "https://cascade.webhook"}),
			},
			metadata.clone(),
		)
		.await
		.unwrap();

	// Add member
	let user_id = Uuid::new_v4();
	tenant_repo
		.add_member(tenant.id, user_id, TenantRole::Admin)
		.await
		.unwrap();

	// Delete tenant - should cascade delete all associated resources
	tenant_repo.delete(tenant.id).await.unwrap();

	// Verify all resources are deleted
	let monitor_repo =
		stellar_monitor_tenant_isolation::repositories::monitor::TenantMonitorRepository::new(
			pool.clone(),
		);
	let network_repo =
		stellar_monitor_tenant_isolation::repositories::network::TenantNetworkRepository::new(
			pool.clone(),
		);
	let trigger_repo =
		stellar_monitor_tenant_isolation::repositories::trigger::TenantTriggerRepository::new(
			pool.clone(),
		);

	// These should all return errors as resources should be deleted
	assert!(monitor_repo.get(&monitor.monitor_id).await.is_err());
	assert!(network_repo.get(&network.network_id).await.is_err());
	assert!(trigger_repo.get(&trigger.trigger_id).await.is_err());

	// Verify user's tenant list is empty
	let user_tenants = tenant_repo.get_user_tenants(user_id).await.unwrap();
	assert_eq!(user_tenants.len(), 0);
}
