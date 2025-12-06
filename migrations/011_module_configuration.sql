-- Module Configuration and Feature Flags
-- This migration enables modular architecture where features can be enabled/disabled

-- Module definitions table
CREATE TABLE IF NOT EXISTS modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL, -- 'core', 'operational', 'integration'
    enabled_by_default BOOLEAN DEFAULT false,
    requires_license BOOLEAN DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Module activation per installation
CREATE TABLE IF NOT EXISTS module_activations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    enabled BOOLEAN DEFAULT true,
    activated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deactivated_at TIMESTAMP,
    activated_by UUID REFERENCES users(id),
    notes TEXT,
    UNIQUE(module_id)
);

-- Feature flags for fine-grained control
CREATE TABLE IF NOT EXISTS feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    module_id UUID REFERENCES modules(id) ON DELETE CASCADE,
    description TEXT,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert Core Modules (always enabled, cannot be disabled)
INSERT INTO modules (name, display_name, description, category, enabled_by_default, requires_license) VALUES
('core_sovereign_lock', 'Sovereign Lock', 'Runtime geofencing for data sovereignty', 'core', true, false),
('core_crypto_shredder', 'Crypto-Shredder', 'GDPR envelope encryption for data erasure', 'core', true, false),
('core_privacy_bridge', 'Privacy Bridge', 'QES sealing for eIDAS compliance', 'core', true, false),
('core_audit_log', 'Audit Log Chain', 'Immutable audit trail for compliance', 'core', true, false),
('core_annex_iv', 'Annex IV Compiler', 'Automated technical documentation generation', 'core', true, false)
ON CONFLICT (name) DO NOTHING;

-- Insert Operational Modules (optional, can be enabled/disabled)
INSERT INTO modules (name, display_name, description, category, enabled_by_default, requires_license) VALUES
('module_data_subject_rights', 'Data Subject Rights', 'GDPR Articles 15-22: Access, export, rectification, erasure', 'operational', true, true),
('module_human_oversight', 'Human Oversight', 'EU AI Act Article 14: Human review queue', 'operational', true, true),
('module_risk_assessment', 'Risk Assessment', 'EU AI Act Article 9: Risk analysis and mitigation', 'operational', true, true),
('module_breach_management', 'Data Breach Management', 'GDPR Articles 33-34: Breach reporting and notification', 'operational', true, true),
('module_consent', 'Consent Management', 'GDPR Articles 6-7: Consent tracking and withdrawal', 'operational', false, true),
('module_dpia', 'DPIA Tracking', 'GDPR Article 35: Data Protection Impact Assessments', 'operational', false, true),
('module_retention', 'Retention Policies', 'GDPR Article 5(1)(e): Automated data retention', 'operational', false, true),
('module_monitoring', 'Post-Market Monitoring', 'EU AI Act Article 72: System monitoring', 'operational', false, true),
('module_green_ai', 'Green AI Telemetry', 'EU AI Act Article 40: Energy and carbon tracking', 'operational', false, true),
('module_ai_bom', 'AI-BOM', 'CycloneDX AI Bill of Materials', 'operational', false, true)
ON CONFLICT (name) DO NOTHING;

-- Insert Integration Modules (always available)
INSERT INTO modules (name, display_name, description, category, enabled_by_default, requires_license) VALUES
('integration_sdks', 'AI Platform SDKs', 'SDKs for Azure, AWS, GCP, LangChain, OpenAI, HuggingFace', 'integration', true, false),
('integration_webhooks', 'Webhooks', 'Real-time event notifications', 'integration', true, false),
('integration_api', 'REST API', 'RESTful API for all features', 'integration', true, false)
ON CONFLICT (name) DO NOTHING;

-- Enable all core modules by default
INSERT INTO module_activations (module_id, enabled, activated_at)
SELECT id, true, CURRENT_TIMESTAMP
FROM modules
WHERE category = 'core'
ON CONFLICT (module_id) DO NOTHING;

-- Enable default operational modules
INSERT INTO module_activations (module_id, enabled, activated_at)
SELECT id, enabled_by_default, CURRENT_TIMESTAMP
FROM modules
WHERE category = 'operational' AND enabled_by_default = true
ON CONFLICT (module_id) DO NOTHING;

-- Enable all integration modules
INSERT INTO module_activations (module_id, enabled, activated_at)
SELECT id, true, CURRENT_TIMESTAMP
FROM modules
WHERE category = 'integration'
ON CONFLICT (module_id) DO NOTHING;

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_module_activations_module_id ON module_activations(module_id);
CREATE INDEX IF NOT EXISTS idx_module_activations_enabled ON module_activations(enabled);
CREATE INDEX IF NOT EXISTS idx_feature_flags_module_id ON feature_flags(module_id);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON feature_flags(enabled);

-- Function to check if module is enabled
CREATE OR REPLACE FUNCTION is_module_enabled(module_name VARCHAR)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1
        FROM modules m
        JOIN module_activations ma ON m.id = ma.module_id
        WHERE m.name = module_name
        AND ma.enabled = true
        AND (ma.deactivated_at IS NULL OR ma.deactivated_at > CURRENT_TIMESTAMP)
    );
END;
$$ LANGUAGE plpgsql;

-- Function to check if feature flag is enabled
CREATE OR REPLACE FUNCTION is_feature_enabled(feature_name VARCHAR)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1
        FROM feature_flags
        WHERE name = feature_name
        AND enabled = true
    );
END;
$$ LANGUAGE plpgsql;

