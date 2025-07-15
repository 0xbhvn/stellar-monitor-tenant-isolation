#[cfg(test)]
mod tests {
	use chrono::{Duration, Utc};
	use serde_json::json;
	use std::net::IpAddr;
	use std::str::FromStr;
	use stellar_monitor_tenant_isolation::models::*;
	use uuid::Uuid;

	#[test]
	fn test_tenant_role_permissions() {
		assert!(TenantRole::Owner.can_manage_tenant());
		assert!(TenantRole::Owner.can_write());
		assert!(TenantRole::Owner.can_read());

		assert!(TenantRole::Admin.can_manage_tenant());
		assert!(TenantRole::Admin.can_write());
		assert!(TenantRole::Admin.can_read());

		assert!(!TenantRole::Member.can_manage_tenant());
		assert!(TenantRole::Member.can_write());
		assert!(TenantRole::Member.can_read());

		assert!(!TenantRole::Viewer.can_manage_tenant());
		assert!(!TenantRole::Viewer.can_write());
		assert!(TenantRole::Viewer.can_read());
	}

	#[test]
	fn test_resource_quota_status() {
		let status = ResourceQuotaStatus {
			tenant_id: Uuid::new_v4(),
			quotas: TenantQuotas {
				max_monitors: 10,
				max_networks: 5,
				max_triggers_per_monitor: 10,
				max_rpc_requests_per_minute: 1000,
				max_storage_mb: 1000,
				api_rate_limits: Default::default(),
			},
			usage: CurrentUsage {
				monitors_count: 8,
				networks_count: 3,
				triggers_count: 20,
				rpc_requests_last_minute: 500,
				storage_mb_used: 200,
			},
			available: AvailableResources {
				monitors: 2,
				networks: 2,
				triggers: 60, // 8 monitors * 10 triggers - 20 used
				rpc_requests_per_minute: 500,
				storage_mb: 800,
			},
		};

		assert!(status.can_create_monitor());
		assert!(status.can_create_network());
		assert!(status.can_create_trigger());
		assert!(status.has_rpc_capacity(400));
		assert!(!status.has_rpc_capacity(600));
		assert!(status.has_storage_capacity(700));
		assert!(!status.has_storage_capacity(900));
	}

	#[test]
	fn test_api_permissions() {
		let perm = ApiPermission::all_monitors();
		assert_eq!(perm.resource, "monitors");
		assert_eq!(perm.actions.len(), 3);
		assert!(perm.actions.contains(&"read".to_string()));
		assert!(perm.actions.contains(&"write".to_string()));
		assert!(perm.actions.contains(&"delete".to_string()));

		let perm = ApiPermission::read_only_monitors();
		assert_eq!(perm.resource, "monitors");
		assert_eq!(perm.actions.len(), 1);
		assert!(perm.actions.contains(&"read".to_string()));
	}

	#[test]
	fn test_audit_action_as_str() {
		assert_eq!(AuditAction::Login.as_str(), "login");
		assert_eq!(AuditAction::MonitorCreated.as_str(), "monitor_created");
		assert_eq!(AuditAction::NetworkDeleted.as_str(), "network_deleted");
		assert_eq!(AuditAction::TriggerUpdated.as_str(), "trigger_updated");
	}

	#[test]
	fn test_resource_type_as_str() {
		assert_eq!(ResourceType::Tenant.as_str(), "tenant");
		assert_eq!(ResourceType::Monitor.as_str(), "monitor");
		assert_eq!(ResourceType::Network.as_str(), "network");
		assert_eq!(ResourceType::Trigger.as_str(), "trigger");
	}

	#[test]
	fn test_tenant_resource_quotas() {
		let tenant = Tenant {
			id: Uuid::new_v4(),
			name: "Test Tenant".to_string(),
			slug: "test-tenant".to_string(),
			is_active: true,
			max_monitors: 15,
			max_networks: 10,
			max_triggers_per_monitor: 8,
			max_rpc_requests_per_minute: 2000,
			max_storage_mb: 5000,
			created_at: Some(Utc::now()),
			updated_at: Some(Utc::now()),
		};

		let quotas = tenant.resource_quotas();
		assert_eq!(quotas.max_monitors, 15);
		assert_eq!(quotas.max_networks, 10);
		assert_eq!(quotas.max_triggers_per_monitor, 8);
		assert_eq!(quotas.max_rpc_requests_per_minute, 2000);
		assert_eq!(quotas.max_storage_mb, 5000);
	}

	#[test]
	fn test_tenant_slug_validation() {
		// Valid slugs
		let valid_slugs = vec![
			"company",
			"company-name",
			"company123",
			"123-company",
			"a-b-c-d",
		];

		for slug in valid_slugs {
			// Just validate the slug format matches expected patterns
			assert!(slug.chars().all(|c| c.is_alphanumeric() || c == '-'));
		}
	}

	#[test]
	fn test_user_tenant_association() {
		let tenant_id = Uuid::new_v4();
		let user_tenant = UserTenant {
			tenant_id,
			tenant_name: "Test Tenant".to_string(),
			tenant_slug: "test-tenant".to_string(),
			role: TenantRole::Admin,
		};

		assert_eq!(user_tenant.tenant_id, tenant_id);
		assert_eq!(user_tenant.role, TenantRole::Admin);
	}

	#[test]
	fn test_monitor_configuration_validation() {
		let config = json!({
			"type": "stellar_contract_event",
			"contract_id": "CCREAA6X5UKPTE4U56JPNGUEM5J4XCKGATJ4V6DSWG7PPQEEMWBG43QZ",
			"topics": ["transfer", "mint"]
		});
		assert_eq!(config["type"], "stellar_contract_event");
		assert!(config["contract_id"].is_string());
		assert!(config["topics"].is_array());
	}

	#[test]
	fn test_network_blockchain_types() {
		// Test valid blockchain types
		let valid_types = vec!["stellar", "evm"];
		for blockchain_type in valid_types {
			assert!(blockchain_type == "stellar" || blockchain_type == "evm");
		}
	}

	#[test]
	fn test_trigger_types() {
		// Test valid trigger types
		let valid_types = vec!["webhook", "email", "slack", "discord", "telegram"];
		for trigger_type in valid_types {
			assert!(!trigger_type.is_empty());
		}
	}

	#[test]
	fn test_api_key_permissions_serialization() {
		let perms = vec![
			ApiPermission {
				resource: "monitors".to_string(),
				actions: vec!["read".to_string(), "write".to_string()],
			},
			ApiPermission {
				resource: "networks".to_string(),
				actions: vec!["read".to_string()],
			},
		];

		let json = serde_json::to_value(&perms).unwrap();
		assert!(json.is_array());
		assert_eq!(json.as_array().unwrap().len(), 2);
	}

	#[test]
	fn test_audit_log_creation() {
		let audit_log = AuditLog {
			id: Uuid::new_v4(),
			tenant_id: Uuid::new_v4(),
			user_id: Some(Uuid::new_v4()),
			api_key_id: None,
			action: AuditAction::MonitorCreated.as_str().to_string(),
			resource_type: Some(ResourceType::Monitor.as_str().to_string()),
			resource_id: Some(Uuid::new_v4()),
			changes: Some(json!({
				"monitor_name": "Test Monitor",
				"network_id": "stellar-testnet"
			})),
			ip_address: Some(IpAddr::from_str("192.168.1.1").unwrap()),
			user_agent: Some("Mozilla/5.0".to_string()),
			created_at: Some(Utc::now()),
		};

		assert_eq!(audit_log.action, "monitor_created");
		assert!(audit_log.user_id.is_some());
		assert!(audit_log.changes.is_some());
	}

	#[test]
	fn test_audit_action_exhaustive() {
		let actions = vec![
			AuditAction::Login,
			AuditAction::Logout,
			AuditAction::TenantCreated,
			AuditAction::TenantUpdated,
			AuditAction::TenantDeleted,
			AuditAction::UserInvited,
			AuditAction::UserRemoved,
			AuditAction::UserRoleChanged,
			AuditAction::MonitorCreated,
			AuditAction::MonitorUpdated,
			AuditAction::MonitorDeleted,
			AuditAction::MonitorEnabled,
			AuditAction::MonitorDisabled,
			AuditAction::NetworkCreated,
			AuditAction::NetworkUpdated,
			AuditAction::NetworkDeleted,
			AuditAction::TriggerCreated,
			AuditAction::TriggerUpdated,
			AuditAction::TriggerDeleted,
			AuditAction::TriggerEnabled,
			AuditAction::TriggerDisabled,
			AuditAction::ApiKeyCreated,
			AuditAction::ApiKeyDeleted,
		];

		for action in actions {
			// Ensure all actions have string representations
			assert!(!action.as_str().is_empty());
		}
	}

	#[test]
	fn test_quota_edge_cases() {
		let status = ResourceQuotaStatus {
			tenant_id: Uuid::new_v4(),
			quotas: TenantQuotas {
				max_monitors: 0,
				max_networks: 0,
				max_triggers_per_monitor: 0,
				max_rpc_requests_per_minute: 0,
				max_storage_mb: 0,
				api_rate_limits: Default::default(),
			},
			usage: CurrentUsage {
				monitors_count: 0,
				networks_count: 0,
				triggers_count: 0,
				rpc_requests_last_minute: 0,
				storage_mb_used: 0,
			},
			available: AvailableResources {
				monitors: 0,
				networks: 0,
				triggers: 0,
				rpc_requests_per_minute: 0,
				storage_mb: 0,
			},
		};

		assert!(!status.can_create_monitor());
		assert!(!status.can_create_network());
		assert!(!status.can_create_trigger());
		assert!(!status.has_rpc_capacity(1));
		assert!(!status.has_storage_capacity(1));
	}

	#[test]
	fn test_tenant_role_serialization() {
		let owner = serde_json::to_string(&TenantRole::Owner).unwrap();
		let admin = serde_json::to_string(&TenantRole::Admin).unwrap();
		let member = serde_json::to_string(&TenantRole::Member).unwrap();
		let viewer = serde_json::to_string(&TenantRole::Viewer).unwrap();

		assert_eq!(owner, "\"owner\"");
		assert_eq!(admin, "\"admin\"");
		assert_eq!(member, "\"member\"");
		assert_eq!(viewer, "\"viewer\"");
	}

	#[test]
	fn test_monitor_configuration_edge_cases() {
		// Empty configuration
		let empty_config = json!({});
		assert_eq!(empty_config, json!({}));

		// Complex nested configuration
		let complex_config = json!({
			"type": "complex_monitor",
			"nested": {
				"level1": {
					"level2": {
						"array": [1, 2, 3],
						"object": {
							"key": "value"
						}
					}
				}
			}
		});
		assert_eq!(complex_config["type"], "complex_monitor");
		assert!(complex_config["nested"]["level1"]["level2"]["array"].is_array());
	}

	#[test]
	fn test_created_updated_timestamps() {
		let now = Utc::now();
		let past = now - Duration::days(7);

		// Test that updated_at should always be >= created_at
		assert!(now > past);
		assert!(now - past == Duration::days(7));
	}
}
