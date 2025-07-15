use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
	pub ip_address: Option<IpAddr>,
	pub user_agent: Option<String>,
}

impl RequestMetadata {
	pub fn new() -> Self {
		Self {
			ip_address: None,
			user_agent: None,
		}
	}

	pub fn with_ip(mut self, ip: Option<IpAddr>) -> Self {
		self.ip_address = ip;
		self
	}

	pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
		self.user_agent = user_agent;
		self
	}
}

impl Default for RequestMetadata {
	fn default() -> Self {
		Self::new()
	}
}
