-- Multi-tenant isolation schema for OpenZeppelin Monitor
-- This schema provides complete tenant isolation with resource quotas

-- Tenants table
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    is_active BOOLEAN DEFAULT true,
    -- Resource quotas
    max_monitors INTEGER DEFAULT 10,
    max_networks INTEGER DEFAULT 5,
    max_triggers_per_monitor INTEGER DEFAULT 10,
    max_rpc_requests_per_minute INTEGER DEFAULT 1000,
    max_storage_mb INTEGER DEFAULT 1000,
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Users table (tenant members)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tenant memberships with roles
CREATE TABLE IF NOT EXISTS tenant_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member', -- 'owner', 'admin', 'member', 'viewer'
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, user_id)
);

-- API keys for programmatic access
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    permissions JSONB DEFAULT '[]'::jsonb,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Networks configuration (tenant-isolated)
CREATE TABLE IF NOT EXISTS tenant_networks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    network_id VARCHAR(255) NOT NULL, -- Original OZ Monitor network ID
    name VARCHAR(255) NOT NULL,
    blockchain VARCHAR(50) NOT NULL, -- 'stellar', 'evm'
    configuration JSONB NOT NULL, -- Full network config from OZ Monitor
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, network_id)
);

-- Monitors configuration (tenant-isolated)
CREATE TABLE IF NOT EXISTS tenant_monitors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    monitor_id VARCHAR(255) NOT NULL, -- Original OZ Monitor ID
    name VARCHAR(255) NOT NULL,
    network_id UUID NOT NULL REFERENCES tenant_networks(id) ON DELETE CASCADE,
    configuration JSONB NOT NULL, -- Full monitor config from OZ Monitor
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, monitor_id)
);

-- Triggers configuration (tenant-isolated)
CREATE TABLE IF NOT EXISTS tenant_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    trigger_id VARCHAR(255) NOT NULL, -- Original OZ Monitor trigger ID
    monitor_id UUID NOT NULL REFERENCES tenant_monitors(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL, -- 'webhook', 'email', 'slack', etc.
    configuration JSONB NOT NULL, -- Full trigger config from OZ Monitor
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, trigger_id)
);

-- Resource usage tracking
CREATE TABLE IF NOT EXISTS resource_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL, -- 'rpc_requests', 'storage', 'compute_minutes'
    usage_value BIGINT NOT NULL,
    usage_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, resource_type, usage_date)
);

-- Audit log for tenant actions
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    changes JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_tenant_memberships_tenant_id ON tenant_memberships(tenant_id);
CREATE INDEX idx_tenant_memberships_user_id ON tenant_memberships(user_id);
CREATE INDEX idx_api_keys_tenant_id ON api_keys(tenant_id);
CREATE INDEX idx_tenant_networks_tenant_id ON tenant_networks(tenant_id);
CREATE INDEX idx_tenant_monitors_tenant_id ON tenant_monitors(tenant_id);
CREATE INDEX idx_tenant_monitors_network_id ON tenant_monitors(network_id);
CREATE INDEX idx_tenant_triggers_tenant_id ON tenant_triggers(tenant_id);
CREATE INDEX idx_tenant_triggers_monitor_id ON tenant_triggers(monitor_id);
CREATE INDEX idx_resource_usage_tenant_date ON resource_usage(tenant_id, usage_date);
CREATE INDEX idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Helper functions
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER update_tenant_memberships_updated_at BEFORE UPDATE ON tenant_memberships
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER update_api_keys_updated_at BEFORE UPDATE ON api_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER update_tenant_networks_updated_at BEFORE UPDATE ON tenant_networks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER update_tenant_monitors_updated_at BEFORE UPDATE ON tenant_monitors
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER update_tenant_triggers_updated_at BEFORE UPDATE ON tenant_triggers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();