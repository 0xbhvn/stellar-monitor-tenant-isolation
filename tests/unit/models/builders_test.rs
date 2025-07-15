#[cfg(test)]
mod tests {
	use chrono::{Duration, Utc};
	#[allow(unused_imports)]
	use stellar_monitor_tenant_isolation::models::*;
	use uuid::Uuid;

	// Test utilities are accessible here since we're in the tests/ directory
	use crate::utils::builders::*;
	use crate::utils::fixtures::*;

	#[test]
	fn test_tenant_builder() {
		let tenant = TenantBuilder::new()
			.with_name("Test Corp")
			.with_slug("test-corp")
			.with_max_monitors(20)
			.build();

		assert_eq!(tenant.name, "Test Corp");
		assert_eq!(tenant.slug, "test-corp");
		assert_eq!(tenant.max_monitors, 20);
		assert!(tenant.is_active);
	}

	#[test]
	fn test_user_builder() {
		let user = UserBuilder::new()
			.with_email("alice@example.com")
			.with_active(false)
			.build();

		assert_eq!(user.email, "alice@example.com");
		assert!(!user.is_active);
		assert!(user.created_at.is_some());
	}

	#[test]
	fn test_monitor_builder_with_fixtures() {
		let monitor = MonitorBuilder::new()
			.with_name("Transfer Events Monitor")
			.with_monitor_id("mon-123")
			.with_configuration(stellar_monitor_config())
			.build();

		assert_eq!(monitor.name, "Transfer Events Monitor");
		assert_eq!(monitor.monitor_id, "mon-123");
		assert!(monitor.is_active.unwrap_or(false));

		// Verify the configuration from fixtures
		let config = &monitor.configuration;
		assert_eq!(config["type"], "stellar_contract_event");
		assert!(config["contract_id"].is_string());
		assert!(config["topics"].is_array());
	}

	#[test]
	fn test_network_builder_with_fixtures() {
		let network = NetworkBuilder::new()
			.with_name("Ethereum Mainnet")
			.with_blockchain("evm")
			.with_network_id("eth-mainnet")
			.with_configuration(evm_network_config())
			.build();

		assert_eq!(network.name, "Ethereum Mainnet");
		assert_eq!(network.blockchain, "evm");
		assert_eq!(network.network_id, "eth-mainnet");

		// Verify the configuration from fixtures
		let config = &network.configuration;
		assert!(config["rpc_url"].is_string());
		assert_eq!(config["chain_id"], 1);
		assert_eq!(config["block_time"], 12);
	}

	#[test]
	fn test_trigger_builder_with_fixtures() {
		let trigger = TriggerBuilder::new()
			.with_name("Slack Alert")
			.with_trigger_type("slack")
			.with_configuration(slack_trigger_config())
			.build();

		assert_eq!(trigger.name, "Slack Alert");
		assert_eq!(trigger.trigger_type, "slack");

		// Verify the configuration from fixtures
		let config = &trigger.configuration;
		assert!(config["webhook_url"].is_string());
		assert_eq!(config["channel"], "#alerts");
		assert_eq!(config["username"], "Monitor Bot");
	}

	#[test]
	fn test_api_key_builder_with_expiry() {
		let expires_at = Utc::now() + Duration::days(30);
		let api_key = ApiKeyBuilder::new()
			.with_name("Production API Key")
			.with_expires_at(expires_at)
			.with_permissions(full_permissions())
			.build();

		assert_eq!(api_key.name, "Production API Key");
		assert_eq!(api_key.expires_at.unwrap(), expires_at);
		assert!(api_key.is_active);

		// Verify permissions from fixtures
		let perms = &api_key.permissions;
		assert!(perms.is_array());
		assert!(perms.as_array().unwrap().len() > 0);
	}

	#[test]
	fn test_test_ids_consistency() {
		let ids = TestIds::default();

		// Verify all IDs are unique
		let all_ids = vec![
			ids.tenant_1,
			ids.tenant_2,
			ids.user_1,
			ids.user_2,
			ids.network_1,
			ids.network_2,
			ids.monitor_1,
			ids.monitor_2,
			ids.trigger_1,
			ids.trigger_2,
			ids.api_key_1,
			ids.api_key_2,
		];

		// Check uniqueness
		use std::collections::HashSet;
		let unique_ids: HashSet<_> = all_ids.iter().collect();
		assert_eq!(unique_ids.len(), all_ids.len());

		// Verify they match expected patterns
		assert_eq!(
			ids.tenant_1.to_string(),
			"11111111-1111-1111-1111-111111111111"
		);
		assert_eq!(
			ids.tenant_2.to_string(),
			"22222222-2222-2222-2222-222222222222"
		);
	}

	#[test]
	fn test_builder_chain_methods() {
		// Test that all builder methods can be chained
		let tenant = TenantBuilder::new()
			.with_id(Uuid::new_v4())
			.with_name("Chain Test")
			.with_slug("chain-test")
			.with_active(true)
			.with_max_monitors(100)
			.with_max_networks(50)
			.with_max_triggers_per_monitor(20)
			.with_max_rpc_requests_per_minute(5000)
			.with_max_storage_mb(10000)
			.with_created_at(Utc::now())
			.with_updated_at(Utc::now())
			.build();

		assert_eq!(tenant.name, "Chain Test");
		assert_eq!(tenant.max_monitors, 100);
		assert_eq!(tenant.max_networks, 50);
	}
}
