use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing::info;

use stellar_monitor_tenant_isolation::{
	api::{create_router, AppState},
	repositories::*,
	services::*,
	utils::AuthService,
	Config,
};

#[tokio::main]
async fn main() -> Result<()> {
	// Load configuration
	let config = Config::from_env().unwrap_or_else(|_| {
		eprintln!("Failed to load configuration from environment, using defaults");
		Config::default()
	});

	// Validate configuration
	if let Err(e) = config.validate() {
		eprintln!("Configuration validation failed: {}", e);
		std::process::exit(1);
	}

	// Initialize tracing
	init_tracing(&config)?;

	info!("Starting Stellar Monitor Tenant Isolation Engine");

	// Initialize database connection pool
	let pool = PgPoolOptions::new()
		.max_connections(config.database.max_connections)
		.min_connections(config.database.min_connections)
		.acquire_timeout(std::time::Duration::from_secs(
			config.database.connect_timeout_seconds,
		))
		.idle_timeout(std::time::Duration::from_secs(
			config.database.idle_timeout_seconds,
		))
		.connect(&config.database.url)
		.await?;

	info!("Connected to database");

	// Run migrations
	info!("Running database migrations...");
	sqlx::migrate!("./migrations").run(&pool).await?;
	info!("Database migrations completed");

	// Initialize repositories
	let tenant_repo = TenantRepository::new(pool.clone());
	let monitor_repo = TenantMonitorRepository::new(pool.clone());
	let network_repo = TenantNetworkRepository::new(pool.clone());
	let trigger_repo = TenantTriggerRepository::new(pool.clone());

	// Initialize services
	let auth_service = AuthService::new(config.auth.jwt_secret.clone());
	let audit_service = AuditService::new(pool.clone());

	let monitor_service = MonitorService::new(
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let network_service = NetworkService::new(
		network_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	let trigger_service = TriggerService::new(
		trigger_repo.clone(),
		monitor_repo.clone(),
		tenant_repo.clone(),
		audit_service.clone(),
	);

	// Create app state
	let app_state = AppState::new(
		monitor_service,
		network_service,
		trigger_service,
		tenant_repo,
		audit_service,
		pool.clone(),
		auth_service,
	);

	// Create router
	let app = create_router(app_state);

	// Start metrics server if enabled
	if config.monitoring.metrics_enabled {
		tokio::spawn(async move {
			start_metrics_server(config.monitoring.metrics_port).await;
		});
	}

	// Start API server
	let addr = config.server.socket_addr();
	info!("Starting API server on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await?;
	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await?;

	info!("Server shut down gracefully");
	Ok(())
}

fn init_tracing(config: &Config) -> Result<()> {
	use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

	let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
		.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.monitoring.tracing_level));

	let fmt_layer = tracing_subscriber::fmt::layer();

	tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt_layer)
		.init();

	Ok(())
}

async fn start_metrics_server(port: u16) {
	use axum::{routing::get, Router};
	use prometheus::{Encoder, TextEncoder};

	let app = Router::new().route(
		"/metrics",
		get(|| async {
			let encoder = TextEncoder::new();
			let metric_families = prometheus::gather();
			let mut buffer = Vec::new();
			encoder.encode(&metric_families, &mut buffer).unwrap();
			String::from_utf8(buffer).unwrap()
		}),
	);

	let addr = SocketAddr::from(([0, 0, 0, 0], port));
	info!("Starting metrics server on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}

async fn shutdown_signal() {
	use tokio::signal;

	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {
			info!("Received Ctrl+C, shutting down");
		},
		_ = terminate => {
			info!("Received terminate signal, shutting down");
		},
	}
}
