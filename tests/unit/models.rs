#[cfg(test)]
mod tests {
	use stellar_monitor_tenant_isolation::models::*;
	use uuid::Uuid;
	use chrono::Utc;
	
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
}