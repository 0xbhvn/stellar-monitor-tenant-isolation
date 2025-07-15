# Stellar Monitor Tenant Isolation Engine

A multi-tenant isolation layer for OpenZeppelin Monitor that enables sandboxed monitoring environments with resource quotas and performance isolation.

## Overview

This project extends OpenZeppelin Monitor with enterprise-grade multi-tenancy capabilities, allowing multiple organizations to share the same monitoring infrastructure while maintaining complete data isolation and resource control.

## Architecture

### Core Design Principles

1. **Zero Core Modifications**: Built as an extension layer without modifying OpenZeppelin Monitor
2. **Complete Tenant Isolation**: Each tenant's data is completely isolated
3. **Resource Quotas**: Configurable limits per tenant for monitors, networks, triggers, RPC requests, and storage
4. **Database-Backed Configuration**: Replaces file-based configuration with PostgreSQL/SQLite storage
5. **Tenant Context Propagation**: Automatic tenant context throughout the request lifecycle

### System Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                    API Gateway                          │
│  (Authentication, Rate Limiting, Tenant Resolution)     │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│              Tenant Context Middleware                  │
│         (Propagates tenant_id through requests)         │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│           Multi-Tenant Repository Layer                 │
│   ┌─────────────┐ ┌──────────────┐ ┌──────────────┐     │
│   │   Monitor   │ │   Network    │ │   Trigger    │     │
│   │ Repository  │ │ Repository   │ │ Repository   │     │
│   └─────────────┘ └──────────────┘ └──────────────┘     │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│           PostgreSQL/SQLite Database                    │
│   (Tenant-isolated configuration storage)               │
└─────────────────────────────────────────────────────────┘
```

### Database Schema

The multi-tenant architecture uses the following key tables:

- **tenants**: Core tenant information and resource quotas
- **users**: User accounts with secure password storage
- **tenant_memberships**: User-tenant relationships with roles
- **api_keys**: Programmatic access tokens
- **tenant_monitors**: Monitor configurations per tenant
- **tenant_networks**: Network configurations per tenant
- **tenant_triggers**: Trigger configurations per tenant
- **resource_usage**: Usage tracking for quotas
- **audit_logs**: Comprehensive audit trail

### Multi-Tenant Isolation

#### 1. Repository Layer Isolation

Each repository wrapper ensures tenant isolation by:

- Automatically filtering queries by `tenant_id`
- Validating resource ownership before operations
- Enforcing resource quotas before creation

```rust
// Example: Getting a monitor automatically filters by current tenant
let monitor = monitor_repo.get("monitor-123").await?;
// SQL: SELECT * FROM tenant_monitors WHERE tenant_id = $1 AND monitor_id = $2
```

#### 2. Tenant Context Propagation

The system uses Tokio's task-local storage for tenant context:

```rust
// Set tenant context for a request
with_tenant_context(context, async {
    // All operations within this scope use the tenant context
    let monitors = monitor_repo.get_all().await?;
}).await
```

#### 3. Resource Quotas

Configurable per-tenant limits:

- Maximum monitors
- Maximum networks
- Maximum triggers per monitor
- RPC requests per minute
- Storage in MB

### Security Features

1. **Authentication**: JWT-based authentication with refresh tokens
2. **Authorization**: Role-based access control (Owner, Admin, Member, Viewer)
3. **API Keys**: Scoped API keys for programmatic access
4. **Audit Logging**: All actions are logged with user/IP information
5. **Password Security**: Argon2 password hashing

## Installation

### Prerequisites

- Rust 1.86+
- PostgreSQL 14+ or SQLite 3.35+
- OpenZeppelin Monitor (as a dependency)

### Setup

1. Clone the repository:

```bash
git clone https://github.com/blip0/stellar-monitor-tenant-isolation
cd stellar-monitor-tenant-isolation
```

1. Set up the database:

```bash
# PostgreSQL
createdb stellar_monitor_tenant
psql stellar_monitor_tenant < migrations/001_initial_schema.sql

# Or SQLite
sqlite3 stellar_monitor_tenant.db < migrations/001_initial_schema.sql
```

1. Configure environment variables:

```bash
cp .env.example .env
# Edit .env with your configuration
```

1. Build and run:

```bash
cargo build --release
cargo run --release
```

## Usage

### API Endpoints

All endpoints are tenant-scoped: `/api/v1/tenants/{tenant_slug}/...`

#### Tenant Management

- `POST /api/v1/tenants` - Create new tenant
- `GET /api/v1/tenants/{slug}` - Get tenant details
- `PUT /api/v1/tenants/{slug}` - Update tenant
- `DELETE /api/v1/tenants/{slug}` - Delete tenant

#### Monitor Management

- `POST /api/v1/tenants/{slug}/monitors` - Create monitor
- `GET /api/v1/tenants/{slug}/monitors` - List monitors
- `GET /api/v1/tenants/{slug}/monitors/{id}` - Get monitor
- `PUT /api/v1/tenants/{slug}/monitors/{id}` - Update monitor
- `DELETE /api/v1/tenants/{slug}/monitors/{id}` - Delete monitor

#### Network Management

- `POST /api/v1/tenants/{slug}/networks` - Create network
- `GET /api/v1/tenants/{slug}/networks` - List networks
- `GET /api/v1/tenants/{slug}/networks/{id}` - Get network
- `PUT /api/v1/tenants/{slug}/networks/{id}` - Update network
- `DELETE /api/v1/tenants/{slug}/networks/{id}` - Delete network

#### Trigger Management

- `POST /api/v1/tenants/{slug}/triggers` - Create trigger
- `GET /api/v1/tenants/{slug}/triggers` - List triggers
- `GET /api/v1/tenants/{slug}/triggers/{id}` - Get trigger
- `PUT /api/v1/tenants/{slug}/triggers/{id}` - Update trigger
- `DELETE /api/v1/tenants/{slug}/triggers/{id}` - Delete trigger

### Example Usage

```rust
// Create a tenant
let tenant = CreateTenantRequest {
    name: "Acme Corp".to_string(),
    slug: "acme-corp".to_string(),
    max_monitors: Some(50),
    max_networks: Some(10),
    max_triggers_per_monitor: Some(20),
    max_rpc_requests_per_minute: Some(5000),
    max_storage_mb: Some(10000),
};

// Create a monitor
let monitor = CreateMonitorRequest {
    monitor_id: "stellar-usdc-monitor".to_string(),
    name: "Stellar USDC Monitor".to_string(),
    network_id: network.id,
    configuration: json!({
        "addresses": ["GCKFBEIYV2U22IO2BJ4KVJOIP7XPWQGZUEQ4NBFKGO6EWUA5HSHBCPZ"],
        "match_conditions": {
            "transactions": true,
            "operations": ["payment", "path_payment_strict_send"]
        }
    }),
};
```

## Development

### Project Structure

```bash
stellar-monitor-tenant-isolation/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── models/              # Data models
│   │   ├── tenant.rs        # Tenant models
│   │   ├── user.rs          # User models
│   │   ├── monitor.rs       # Monitor/Network/Trigger models
│   │   └── ...
│   ├── repositories/        # Data access layer
│   │   ├── tenant.rs        # Tenant repository
│   │   ├── monitor.rs       # Monitor repository wrapper
│   │   ├── network.rs       # Network repository wrapper
│   │   └── trigger.rs       # Trigger repository wrapper
│   ├── services/            # Business logic layer
│   ├── api/                 # REST API handlers
│   └── utils/               # Utilities
│       └── tenant_context.rs # Tenant context management
├── migrations/              # Database migrations
├── tests/                   # Test suite
└── docs/                    # Documentation
```

### Testing

Run the test suite:

```bash
cargo test
```

Run with coverage:

```bash
cargo llvm-cov
```

### Code Quality

Format code:

```bash
cargo fmt
```

Run linter:

```bash
cargo clippy
```

## Deployment

### Docker

```dockerfile
FROM rust:1.86 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/stellar-monitor-tenant /usr/local/bin/
CMD ["stellar-monitor-tenant"]
```

### Kubernetes

See `k8s/` directory for Kubernetes manifests supporting:

- Horizontal pod autoscaling
- Resource limits per pod
- Network policies for tenant isolation
- Persistent volume claims for data

## Performance Considerations

1. **Connection Pooling**: Database connections are pooled per tenant
2. **Caching**: Redis cache for frequently accessed configurations
3. **Rate Limiting**: Per-tenant rate limits at API gateway
4. **Resource Limits**: Container-level resource constraints

## Roadmap

- [ ] Service isolation wrappers
- [ ] API gateway implementation
- [ ] Comprehensive test suite
- [ ] Prometheus metrics per tenant
- [ ] GraphQL API support
- [ ] Webhook management UI
- [ ] Tenant billing integration

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built on top of the excellent [OpenZeppelin Monitor](https://github.com/OpenZeppelin/openzeppelin-monitor) project.
