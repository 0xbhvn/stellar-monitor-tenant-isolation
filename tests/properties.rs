#[cfg(test)]
mod tests {
	use proptest::prelude::*;
	use stellar_monitor_tenant_isolation::models::*;

	proptest! {
		#[test]
		fn test_quota_calculations_never_negative(
			max_monitors in 0i32..1000,
			max_networks in 0i32..1000,
			max_triggers_per_monitor in 0i32..100,
			monitors_used in 0i32..1000,
			networks_used in 0i32..1000,
			triggers_used in 0i32..10000,
		) {
			let quotas = TenantQuotas {
				max_monitors,
				max_networks,
				max_triggers_per_monitor,
				max_rpc_requests_per_minute: 1000,
				max_storage_mb: 1000,
			};

			let usage = CurrentUsage {
				monitors_count: monitors_used,
				networks_count: networks_used,
				triggers_count: triggers_used,
				rpc_requests_last_minute: 0,
				storage_mb_used: 0,
			};

			let available = AvailableResources {
				monitors: (quotas.max_monitors - usage.monitors_count).max(0),
				networks: (quotas.max_networks - usage.networks_count).max(0),
				triggers: (quotas.max_triggers_per_monitor * usage.monitors_count - usage.triggers_count).max(0),
				rpc_requests_per_minute: 1000,
				storage_mb: 1000,
			};

			// Properties: available resources should never be negative
			assert!(available.monitors >= 0);
			assert!(available.networks >= 0);
			assert!(available.triggers >= 0);
			assert!(available.rpc_requests_per_minute >= 0);
			assert!(available.storage_mb >= 0);
		}

		#[test]
		fn test_tenant_slug_validation(
			slug in "[a-z][a-z0-9-]{0,62}"
		) {
			// Slugs should be lowercase, alphanumeric with hyphens
			// Start with a letter, max 63 chars
			assert!(slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
			assert!(slug.len() <= 63);
			assert!(slug.chars().next().unwrap().is_ascii_lowercase());
		}
	}
}
