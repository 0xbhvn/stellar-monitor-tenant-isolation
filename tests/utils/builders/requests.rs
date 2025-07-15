use serde_json::{json, Value as JsonValue};
use std::net::IpAddr;
use std::str::FromStr;
use stellar_monitor_tenant_isolation::models::*;
use uuid::Uuid;

// Builder for CreateTenantRequest
pub struct CreateTenantRequestBuilder {
	name: String,
	slug: String,
	max_monitors: Option<i32>,
	max_networks: Option<i32>,
	max_triggers_per_monitor: Option<i32>,
	max_rpc_requests_per_minute: Option<i32>,
	max_storage_mb: Option<i32>,
}

impl Default for CreateTenantRequestBuilder {
	fn default() -> Self {
		Self {
			name: "Test Tenant".to_string(),
			slug: "test-tenant".to_string(),
			max_monitors: Some(10),
			max_networks: Some(5),
			max_triggers_per_monitor: Some(5),
			max_rpc_requests_per_minute: Some(100),
			max_storage_mb: Some(1000),
		}
	}
}

impl CreateTenantRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_slug(mut self, slug: impl Into<String>) -> Self {
		self.slug = slug.into();
		self
	}

	pub fn with_max_monitors(mut self, max: i32) -> Self {
		self.max_monitors = Some(max);
		self
	}

	pub fn with_no_monitor_limit(mut self) -> Self {
		self.max_monitors = None;
		self
	}

	pub fn build(self) -> CreateTenantRequest {
		CreateTenantRequest {
			name: self.name,
			slug: self.slug,
			max_monitors: self.max_monitors,
			max_networks: self.max_networks,
			max_triggers_per_monitor: self.max_triggers_per_monitor,
			max_rpc_requests_per_minute: self.max_rpc_requests_per_minute,
			max_storage_mb: self.max_storage_mb,
		}
	}
}

// Builder for UpdateTenantRequest
pub struct UpdateTenantRequestBuilder {
	name: Option<String>,
	is_active: Option<bool>,
	max_monitors: Option<i32>,
	max_networks: Option<i32>,
	max_triggers_per_monitor: Option<i32>,
	max_rpc_requests_per_minute: Option<i32>,
	max_storage_mb: Option<i32>,
}

impl Default for UpdateTenantRequestBuilder {
	fn default() -> Self {
		Self {
			name: None,
			is_active: None,
			max_monitors: None,
			max_networks: None,
			max_triggers_per_monitor: None,
			max_rpc_requests_per_minute: None,
			max_storage_mb: None,
		}
	}
}

impl UpdateTenantRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = Some(is_active);
		self
	}

	pub fn with_max_monitors(mut self, max: i32) -> Self {
		self.max_monitors = Some(max);
		self
	}

	pub fn build(self) -> UpdateTenantRequest {
		UpdateTenantRequest {
			name: self.name,
			is_active: self.is_active,
			max_monitors: self.max_monitors,
			max_networks: self.max_networks,
			max_triggers_per_monitor: self.max_triggers_per_monitor,
			max_rpc_requests_per_minute: self.max_rpc_requests_per_minute,
			max_storage_mb: self.max_storage_mb,
		}
	}
}

// Builder for CreateMonitorRequest
pub struct CreateMonitorRequestBuilder {
	monitor_id: String,
	name: String,
	network_id: Uuid,
	configuration: JsonValue,
}

impl Default for CreateMonitorRequestBuilder {
	fn default() -> Self {
		Self {
			monitor_id: format!("monitor-{}", Uuid::new_v4()),
			name: "Test Monitor".to_string(),
			network_id: Uuid::new_v4(),
			configuration: json!({
				"type": "transaction",
				"filters": {}
			}),
		}
	}
}

impl CreateMonitorRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_monitor_id(mut self, id: impl Into<String>) -> Self {
		self.monitor_id = id.into();
		self
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_network_id(mut self, id: Uuid) -> Self {
		self.network_id = id;
		self
	}

	pub fn with_configuration(mut self, config: JsonValue) -> Self {
		self.configuration = config;
		self
	}

	pub fn build(self) -> CreateMonitorRequest {
		CreateMonitorRequest {
			monitor_id: self.monitor_id,
			name: self.name,
			network_id: self.network_id,
			configuration: self.configuration,
		}
	}
}

// Builder for UpdateMonitorRequest
pub struct UpdateMonitorRequestBuilder {
	name: Option<String>,
	configuration: Option<JsonValue>,
	is_active: Option<bool>,
}

impl Default for UpdateMonitorRequestBuilder {
	fn default() -> Self {
		Self {
			name: None,
			configuration: None,
			is_active: None,
		}
	}
}

impl UpdateMonitorRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	pub fn with_configuration(mut self, config: JsonValue) -> Self {
		self.configuration = Some(config);
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = Some(is_active);
		self
	}

	pub fn build(self) -> UpdateMonitorRequest {
		UpdateMonitorRequest {
			name: self.name,
			configuration: self.configuration,
			is_active: self.is_active,
		}
	}
}

// Builder for CreateNetworkRequest
pub struct CreateNetworkRequestBuilder {
	network_id: String,
	name: String,
	blockchain: String,
	configuration: JsonValue,
}

impl Default for CreateNetworkRequestBuilder {
	fn default() -> Self {
		Self {
			network_id: format!("network-{}", Uuid::new_v4()),
			name: "Test Network".to_string(),
			blockchain: "stellar".to_string(),
			configuration: json!({
				"rpc_url": "https://example.com/rpc",
				"chain_id": 1
			}),
		}
	}
}

impl CreateNetworkRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_network_id(mut self, id: impl Into<String>) -> Self {
		self.network_id = id.into();
		self
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_blockchain(mut self, blockchain: impl Into<String>) -> Self {
		self.blockchain = blockchain.into();
		self
	}

	pub fn with_configuration(mut self, config: JsonValue) -> Self {
		self.configuration = config;
		self
	}

	pub fn build(self) -> CreateNetworkRequest {
		CreateNetworkRequest {
			network_id: self.network_id,
			name: self.name,
			blockchain: self.blockchain,
			configuration: self.configuration,
		}
	}
}

// Builder for UpdateNetworkRequest
pub struct UpdateNetworkRequestBuilder {
	name: Option<String>,
	configuration: Option<JsonValue>,
	is_active: Option<bool>,
}

impl Default for UpdateNetworkRequestBuilder {
	fn default() -> Self {
		Self {
			name: None,
			configuration: None,
			is_active: None,
		}
	}
}

impl UpdateNetworkRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	pub fn with_configuration(mut self, config: JsonValue) -> Self {
		self.configuration = Some(config);
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = Some(is_active);
		self
	}

	pub fn build(self) -> UpdateNetworkRequest {
		UpdateNetworkRequest {
			name: self.name,
			configuration: self.configuration,
			is_active: self.is_active,
		}
	}
}

// Builder for CreateTriggerRequest
pub struct CreateTriggerRequestBuilder {
	trigger_id: String,
	monitor_id: Uuid,
	name: String,
	trigger_type: String,
	configuration: JsonValue,
}

impl Default for CreateTriggerRequestBuilder {
	fn default() -> Self {
		Self {
			trigger_id: format!("trigger-{}", Uuid::new_v4()),
			monitor_id: Uuid::new_v4(),
			name: "Test Trigger".to_string(),
			trigger_type: "webhook".to_string(),
			configuration: json!({
				"url": "https://example.com/webhook",
				"method": "POST"
			}),
		}
	}
}

impl CreateTriggerRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_trigger_id(mut self, id: impl Into<String>) -> Self {
		self.trigger_id = id.into();
		self
	}

	pub fn with_monitor_id(mut self, id: Uuid) -> Self {
		self.monitor_id = id;
		self
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = name.into();
		self
	}

	pub fn with_trigger_type(mut self, trigger_type: impl Into<String>) -> Self {
		self.trigger_type = trigger_type.into();
		self
	}

	pub fn with_configuration(mut self, config: JsonValue) -> Self {
		self.configuration = config;
		self
	}

	pub fn build(self) -> CreateTriggerRequest {
		CreateTriggerRequest {
			trigger_id: self.trigger_id,
			monitor_id: self.monitor_id,
			name: self.name,
			trigger_type: self.trigger_type,
			configuration: self.configuration,
		}
	}
}

// Builder for UpdateTriggerRequest
pub struct UpdateTriggerRequestBuilder {
	name: Option<String>,
	configuration: Option<JsonValue>,
	is_active: Option<bool>,
}

impl Default for UpdateTriggerRequestBuilder {
	fn default() -> Self {
		Self {
			name: None,
			configuration: None,
			is_active: None,
		}
	}
}

impl UpdateTriggerRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_name(mut self, name: impl Into<String>) -> Self {
		self.name = Some(name.into());
		self
	}

	pub fn with_configuration(mut self, config: JsonValue) -> Self {
		self.configuration = Some(config);
		self
	}

	pub fn with_active(mut self, is_active: bool) -> Self {
		self.is_active = Some(is_active);
		self
	}

	pub fn build(self) -> UpdateTriggerRequest {
		UpdateTriggerRequest {
			name: self.name,
			configuration: self.configuration,
			is_active: self.is_active,
		}
	}
}

// Builder for RequestMetadata
pub struct RequestMetadataBuilder {
	ip_address: Option<IpAddr>,
	user_agent: Option<String>,
}

impl Default for RequestMetadataBuilder {
	fn default() -> Self {
		Self {
			ip_address: IpAddr::from_str("127.0.0.1").ok(),
			user_agent: Some("Test Agent".to_string()),
		}
	}
}

impl RequestMetadataBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
		self.ip_address = IpAddr::from_str(&ip.into()).ok();
		self
	}

	pub fn with_ip_addr(mut self, ip: IpAddr) -> Self {
		self.ip_address = Some(ip);
		self
	}

	pub fn with_user_agent(mut self, agent: impl Into<String>) -> Self {
		self.user_agent = Some(agent.into());
		self
	}

	pub fn build(self) -> RequestMetadata {
		RequestMetadata {
			ip_address: self.ip_address,
			user_agent: self.user_agent,
		}
	}
}

// Builder for CreateAuditLogRequest
pub struct CreateAuditLogRequestBuilder {
	tenant_id: Uuid,
	user_id: Option<Uuid>,
	api_key_id: Option<Uuid>,
	action: AuditAction,
	resource_type: Option<ResourceType>,
	resource_id: Option<Uuid>,
	changes: Option<JsonValue>,
	ip_address: Option<IpAddr>,
	user_agent: Option<String>,
}

impl Default for CreateAuditLogRequestBuilder {
	fn default() -> Self {
		Self {
			tenant_id: Uuid::new_v4(),
			user_id: Some(Uuid::new_v4()),
			api_key_id: None,
			action: AuditAction::MonitorCreated,
			resource_type: Some(ResourceType::Monitor),
			resource_id: Some(Uuid::new_v4()),
			changes: None,
			ip_address: IpAddr::from_str("127.0.0.1").ok(),
			user_agent: Some("Test Agent".to_string()),
		}
	}
}

impl CreateAuditLogRequestBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_tenant_id(mut self, id: Uuid) -> Self {
		self.tenant_id = id;
		self
	}

	pub fn with_user_id(mut self, id: Uuid) -> Self {
		self.user_id = Some(id);
		self
	}

	pub fn with_api_key_id(mut self, id: Uuid) -> Self {
		self.api_key_id = Some(id);
		self
	}

	pub fn with_action(mut self, action: AuditAction) -> Self {
		self.action = action;
		self
	}

	pub fn with_resource_type(mut self, resource_type: ResourceType) -> Self {
		self.resource_type = Some(resource_type);
		self
	}

	pub fn with_resource_id(mut self, id: Uuid) -> Self {
		self.resource_id = Some(id);
		self
	}

	pub fn with_changes(mut self, changes: JsonValue) -> Self {
		self.changes = Some(changes);
		self
	}

	pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
		self.ip_address = IpAddr::from_str(&ip.into()).ok();
		self
	}

	pub fn with_ip_addr(mut self, ip: IpAddr) -> Self {
		self.ip_address = Some(ip);
		self
	}

	pub fn build(self) -> CreateAuditLogRequest {
		CreateAuditLogRequest {
			tenant_id: self.tenant_id,
			user_id: self.user_id,
			api_key_id: self.api_key_id,
			action: self.action,
			resource_type: self.resource_type,
			resource_id: self.resource_id,
			changes: self.changes,
			ip_address: self.ip_address,
			user_agent: self.user_agent,
		}
	}
}
