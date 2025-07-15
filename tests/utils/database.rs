use sqlx::{postgres::PgPoolOptions, PgPool};
use stellar_monitor_tenant_isolation::config::Config;
use uuid::Uuid;

/// Create a test database pool with a unique database for isolation
pub async fn create_test_pool() -> anyhow::Result<PgPool> {
	let config = Config::from_env()?;
	let database_url = config.database.url.clone();

	// Parse the database URL to extract components
	let base_url = if database_url.contains('?') {
		database_url.split('?').next().unwrap()
	} else {
		&database_url
	};

	// Create a unique test database name
	let test_db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace('-', "_"));

	// Connect to the default database to create the test database
	let admin_url = base_url
		.rsplit('/')
		.next()
		.map(|_| base_url.rsplitn(2, '/').nth(1).unwrap())
		.unwrap_or(&base_url);
	let admin_url = format!("{}/postgres", admin_url);

	let admin_pool = PgPoolOptions::new()
		.max_connections(1)
		.connect(&admin_url)
		.await?;

	// Create the test database
	sqlx::query(&format!("CREATE DATABASE {}", test_db_name))
		.execute(&admin_pool)
		.await?;

	admin_pool.close().await;

	// Connect to the test database
	let test_database_url = format!(
		"{}/{}",
		base_url.rsplitn(2, '/').nth(1).unwrap_or(&base_url),
		test_db_name
	);

	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&test_database_url)
		.await?;

	Ok(pool)
}

/// Run database migrations on the test pool
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
	sqlx::migrate!("./migrations").run(pool).await?;
	Ok(())
}

/// Clean up test database after tests
pub async fn cleanup_database(pool: PgPool) -> anyhow::Result<()> {
	let db_name = pool
		.connect_options()
		.get_database()
		.unwrap_or("unknown")
		.to_string();

	pool.close().await;

	// Connect to admin database to drop the test database
	let config = Config::from_env()?;
	let database_url = config.database.url.clone();
	let base_url = if database_url.contains('?') {
		database_url.split('?').next().unwrap()
	} else {
		&database_url
	};

	let admin_url = format!(
		"{}/postgres",
		base_url.rsplitn(2, '/').nth(1).unwrap_or(&base_url)
	);

	let admin_pool = PgPoolOptions::new()
		.max_connections(1)
		.connect(&admin_url)
		.await?;

	// Force disconnect all connections to the test database
	sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
        db_name
    ))
    .execute(&admin_pool)
    .await?;

	// Drop the test database
	sqlx::query(&format!("DROP DATABASE IF EXISTS {}", db_name))
		.execute(&admin_pool)
		.await?;

	admin_pool.close().await;

	Ok(())
}

/// Test database wrapper that automatically cleans up on drop
pub struct TestDatabase {
	pub pool: PgPool,
}

impl TestDatabase {
	pub async fn new() -> anyhow::Result<Self> {
		let pool = create_test_pool().await?;
		run_migrations(&pool).await?;
		Ok(Self { pool })
	}
}

impl Drop for TestDatabase {
	fn drop(&mut self) {
		// We can't do async cleanup in Drop, so we'll rely on explicit cleanup
		// or let the test database be cleaned up by the test runner
	}
}

/// Macro to set up a test database for a test function
#[macro_export]
macro_rules! test_db {
	() => {{
		let db = $crate::utils::database::TestDatabase::new().await?;
		db.pool
	}};
}
