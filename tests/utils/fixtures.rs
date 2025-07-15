use serde_json::json;
use uuid::Uuid;

/// Common test UUIDs for consistent testing
pub struct TestIds {
	pub tenant_1: Uuid,
	pub tenant_2: Uuid,
	pub user_1: Uuid,
	pub user_2: Uuid,
	pub network_1: Uuid,
	pub network_2: Uuid,
	pub monitor_1: Uuid,
	pub monitor_2: Uuid,
	pub trigger_1: Uuid,
	pub trigger_2: Uuid,
	pub api_key_1: Uuid,
	pub api_key_2: Uuid,
}

impl Default for TestIds {
	fn default() -> Self {
		Self {
			tenant_1: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
			tenant_2: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
			user_1: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
			user_2: Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap(),
			network_1: Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap(),
			network_2: Uuid::parse_str("66666666-6666-6666-6666-666666666666").unwrap(),
			monitor_1: Uuid::parse_str("77777777-7777-7777-7777-777777777777").unwrap(),
			monitor_2: Uuid::parse_str("88888888-8888-8888-8888-888888888888").unwrap(),
			trigger_1: Uuid::parse_str("99999999-9999-9999-9999-999999999999").unwrap(),
			trigger_2: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
			api_key_1: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
			api_key_2: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
		}
	}
}

/// Sample monitor configurations
pub fn stellar_monitor_config() -> serde_json::Value {
	json!({
		"type": "stellar_contract_event",
		"contract_id": "CCREAA6X5UKPTE4U56JPNGUEM5J4XCKGATJ4V6DSWG7PPQEEMWBG43QZ",
		"topics": ["transfer", "mint"],
		"filters": {
			"amount": {
				"gte": "1000000"
			}
		}
	})
}

pub fn evm_monitor_config() -> serde_json::Value {
	json!({
		"type": "evm_contract_event",
		"contract_address": "0x1234567890abcdef1234567890abcdef12345678",
		"event_signature": "Transfer(address,address,uint256)",
		"filters": {
			"from": "0x0000000000000000000000000000000000000000"
		}
	})
}

/// Sample network configurations
pub fn stellar_network_config() -> serde_json::Value {
	json!({
		"rpc_url": "https://horizon-testnet.stellar.org",
		"chain_id": "testnet",
		"block_time": 5,
		"max_retries": 3,
		"timeout": 30
	})
}

pub fn evm_network_config() -> serde_json::Value {
	json!({
		"rpc_url": "https://eth-mainnet.g.alchemy.com/v2/your-api-key",
		"chain_id": 1,
		"block_time": 12,
		"max_retries": 3,
		"timeout": 30
	})
}

/// Sample trigger configurations
pub fn webhook_trigger_config() -> serde_json::Value {
	json!({
		"url": "https://example.com/webhook",
		"method": "POST",
		"headers": {
			"Content-Type": "application/json",
			"X-API-Key": "secret-key"
		},
		"timeout": 30,
		"retry_count": 3
	})
}

pub fn email_trigger_config() -> serde_json::Value {
	json!({
		"to": ["alerts@example.com"],
		"cc": [],
		"subject": "Monitor Alert: {{monitor_name}}",
		"template": "default",
		"smtp_config": {
			"host": "smtp.example.com",
			"port": 587,
			"username": "alerts@example.com",
			"password": "smtp-password"
		}
	})
}

pub fn slack_trigger_config() -> serde_json::Value {
	json!({
		"webhook_url": "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX",
		"channel": "#alerts",
		"username": "Monitor Bot",
		"icon_emoji": ":robot_face:",
		"template": "default"
	})
}

/// API permission sets
pub fn read_only_permissions() -> serde_json::Value {
	json!([
		{
			"resource": "monitors",
			"actions": ["read"]
		},
		{
			"resource": "networks",
			"actions": ["read"]
		},
		{
			"resource": "triggers",
			"actions": ["read"]
		}
	])
}

pub fn full_permissions() -> serde_json::Value {
	json!([
		{
			"resource": "monitors",
			"actions": ["read", "write", "delete"]
		},
		{
			"resource": "networks",
			"actions": ["read", "write", "delete"]
		},
		{
			"resource": "triggers",
			"actions": ["read", "write", "delete"]
		},
		{
			"resource": "api_keys",
			"actions": ["read", "write", "delete"]
		}
	])
}
