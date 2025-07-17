-- Add trigger scripts table for storing script content in database
-- This removes the need for filesystem access for script execution

-- Trigger scripts table
CREATE TABLE IF NOT EXISTS trigger_scripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    language VARCHAR(50) NOT NULL, -- 'Python', 'JavaScript', 'Bash'
    content TEXT NOT NULL, -- The actual script content
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

-- Add reference to scripts in monitor configurations
ALTER TABLE tenant_monitors
ADD COLUMN trigger_script_ids UUID[] DEFAULT '{}';

-- Add reference to scripts in trigger configurations
ALTER TABLE tenant_triggers
ADD COLUMN script_id UUID REFERENCES trigger_scripts(id) ON DELETE SET NULL;

-- Indexes for performance
CREATE INDEX idx_trigger_scripts_tenant_id ON trigger_scripts(tenant_id);
CREATE INDEX idx_trigger_scripts_language ON trigger_scripts(language);
CREATE INDEX idx_trigger_scripts_is_active ON trigger_scripts(is_active);

-- Add trigger for updated_at
CREATE TRIGGER update_trigger_scripts_updated_at BEFORE UPDATE ON trigger_scripts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Sample script data for development/testing
-- INSERT INTO trigger_scripts (tenant_id, name, description, language, content) VALUES
-- ('550e8400-e29b-41d4-a716-446655440001', 'filter_even_blocks', 'Filters transactions on even block numbers', 'Python', E'import sys\nimport json\n\ndef main():\n    try:\n        input_data = sys.stdin.read()\n        data = json.loads(input_data)\n        monitor_match = data[\'monitor_match\']\n        \n        # Extract block number based on blockchain type\n        block_number = None\n        if "EVM" in monitor_match:\n            hex_block = monitor_match[\'EVM\'][\'transaction\'].get(\'blockNumber\')\n            if hex_block:\n                block_number = int(hex_block, 16)\n        elif "Stellar" in monitor_match:\n            block_number = monitor_match[\'Stellar\'][\'ledger\'].get(\'sequence\')\n        \n        if block_number is None:\n            print("false")\n            return\n        \n        # Return true for even blocks (filter them out)\n        result = block_number % 2 == 0\n        print(str(result).lower())\n        \n    except Exception as e:\n        print(f"Error: {e}", file=sys.stderr)\n        print("false")\n\nif __name__ == "__main__":\n    main()');