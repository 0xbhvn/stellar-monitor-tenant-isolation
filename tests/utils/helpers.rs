use mockall::{predicate::*, Predicate};
use stellar_monitor_tenant_isolation::{
	models::*, repositories::error::TenantRepositoryError, services::monitor_service::ServiceError,
};
use uuid::Uuid;

// Helper functions for creating common test expectations

pub fn expect_successful_create<T: Clone>(
	result: T,
) -> impl Fn() -> Result<T, TenantRepositoryError> {
	move || Ok(result.clone())
}

pub fn expect_not_found_error() -> impl Fn() -> Result<Tenant, TenantRepositoryError> {
	|| Err(TenantRepositoryError::TenantNotFound(Uuid::new_v4()))
}

pub fn expect_quota_exceeded_error() -> impl Fn() -> Result<bool, TenantRepositoryError> {
	|| Ok(false) // false means quota exceeded
}

// Predicate helpers for mockall
pub fn uuid_eq(id: Uuid) -> impl Predicate<Uuid> {
	eq(id)
}

pub fn str_eq(s: &str) -> impl Predicate<str> {
	eq(s.to_string())
}

pub fn any_uuid() -> impl Predicate<Uuid> {
	always()
}

pub fn any_str() -> impl Predicate<str> {
	always()
}

// Service error helpers
pub fn service_access_denied(msg: &str) -> ServiceError {
	ServiceError::AccessDenied(msg.to_string())
}

pub fn service_quota_exceeded(msg: &str) -> ServiceError {
	ServiceError::QuotaExceeded(msg.to_string())
}

pub fn service_validation_error(msg: &str) -> ServiceError {
	ServiceError::ValidationError(msg.to_string())
}

pub fn service_internal_error(msg: &str) -> ServiceError {
	ServiceError::Internal(msg.to_string())
}

// Test data consistency helpers
pub struct TestDataValidator;

impl TestDataValidator {
	pub fn validate_tenant(tenant: &Tenant) -> Result<(), String> {
		if tenant.name.is_empty() {
			return Err("Tenant name cannot be empty".to_string());
		}
		if tenant.slug.is_empty() {
			return Err("Tenant slug cannot be empty".to_string());
		}
		if tenant.max_monitors < 0 {
			return Err("Max monitors cannot be negative".to_string());
		}
		if tenant.max_networks < 0 {
			return Err("Max networks cannot be negative".to_string());
		}
		if tenant.max_triggers_per_monitor < 0 {
			return Err("Max triggers per monitor cannot be negative".to_string());
		}
		Ok(())
	}

	pub fn validate_monitor(monitor: &TenantMonitor) -> Result<(), String> {
		if monitor.name.is_empty() {
			return Err("Monitor name cannot be empty".to_string());
		}
		if monitor.monitor_id.is_empty() {
			return Err("Monitor ID cannot be empty".to_string());
		}
		Ok(())
	}

	pub fn validate_network(network: &TenantNetwork) -> Result<(), String> {
		if network.name.is_empty() {
			return Err("Network name cannot be empty".to_string());
		}
		if network.network_id.is_empty() {
			return Err("Network ID cannot be empty".to_string());
		}
		if network.blockchain.is_empty() {
			return Err("Blockchain type cannot be empty".to_string());
		}
		Ok(())
	}

	pub fn validate_trigger(trigger: &TenantTrigger) -> Result<(), String> {
		if trigger.name.is_empty() {
			return Err("Trigger name cannot be empty".to_string());
		}
		if trigger.trigger_type.is_empty() {
			return Err("Trigger type cannot be empty".to_string());
		}
		Ok(())
	}
}

// Assertion helpers
#[macro_export]
macro_rules! assert_tenant_eq {
	($left:expr, $right:expr) => {
		assert_eq!($left.id, $right.id, "Tenant IDs don't match");
		assert_eq!($left.name, $right.name, "Tenant names don't match");
		assert_eq!($left.slug, $right.slug, "Tenant slugs don't match");
		assert_eq!(
			$left.is_active, $right.is_active,
			"Tenant active status doesn't match"
		);
		assert_eq!(
			$left.max_monitors, $right.max_monitors,
			"Tenant max_monitors don't match"
		);
		assert_eq!(
			$left.max_networks, $right.max_networks,
			"Tenant max_networks don't match"
		);
	};
}

#[macro_export]
macro_rules! assert_monitor_eq {
	($left:expr, $right:expr) => {
		assert_eq!($left.id, $right.id, "Monitor IDs don't match");
		assert_eq!(
			$left.tenant_id, $right.tenant_id,
			"Monitor tenant_ids don't match"
		);
		assert_eq!($left.name, $right.name, "Monitor names don't match");
		assert_eq!(
			$left.monitor_id, $right.monitor_id,
			"Monitor original IDs don't match"
		);
		assert_eq!(
			$left.is_active, $right.is_active,
			"Monitor active status doesn't match"
		);
	};
}

// Cleanup helpers
pub struct TestCleanup {
	cleanup_fns: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestCleanup {
	pub fn new() -> Self {
		Self {
			cleanup_fns: Vec::new(),
		}
	}

	pub fn add(&mut self, f: impl FnOnce() + Send + 'static) {
		self.cleanup_fns.push(Box::new(f));
	}

	pub fn run(mut self) {
		// Take ownership of the cleanup functions to avoid move errors
		let cleanup_fns = std::mem::take(&mut self.cleanup_fns);
		for f in cleanup_fns {
			f();
		}
	}
}

impl Drop for TestCleanup {
	fn drop(&mut self) {
		// Cleanup is already run in the run() method
	}
}

// Time helpers for testing
pub mod time {
	use chrono::{DateTime, Duration, Utc};

	pub fn now() -> DateTime<Utc> {
		Utc::now()
	}

	pub fn yesterday() -> DateTime<Utc> {
		Utc::now() - Duration::days(1)
	}

	pub fn tomorrow() -> DateTime<Utc> {
		Utc::now() + Duration::days(1)
	}

	pub fn days_ago(days: i64) -> DateTime<Utc> {
		Utc::now() - Duration::days(days)
	}

	pub fn days_from_now(days: i64) -> DateTime<Utc> {
		Utc::now() + Duration::days(days)
	}
}
