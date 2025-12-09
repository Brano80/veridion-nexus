-- Security Hardening: Users, Roles, API Keys, Audit Logs

-- Users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL, -- bcrypt hashed
    full_name VARCHAR(255),
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- User-Role mapping (many-to-many)
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id)
);

-- Permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    resource VARCHAR(100) NOT NULL, -- e.g., 'compliance_records', 'webhooks'
    action VARCHAR(50) NOT NULL, -- e.g., 'read', 'write', 'delete'
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Role-Permission mapping (many-to-many)
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- API Keys for service-to-service authentication
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash VARCHAR(255) UNIQUE NOT NULL, -- SHA-256 hash of the key
    name VARCHAR(255) NOT NULL,
    description TEXT,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    permissions TEXT[], -- Array of permission names
    expires_at TIMESTAMP,
    last_used_at TIMESTAMP,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Security audit logs
CREATE TABLE IF NOT EXISTS security_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    event_type VARCHAR(100) NOT NULL, -- 'login', 'logout', 'permission_denied', 'rate_limit_exceeded', etc.
    resource VARCHAR(100),
    action VARCHAR(50),
    ip_address INET,
    user_agent TEXT,
    success BOOLEAN DEFAULT true,
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Rate limiting tracking
CREATE TABLE IF NOT EXISTS rate_limit_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identifier VARCHAR(255) NOT NULL, -- IP address or user_id or api_key_id
    endpoint VARCHAR(255) NOT NULL,
    request_count INTEGER DEFAULT 1,
    window_start TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(identifier, endpoint, window_start)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(active) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON role_permissions(permission_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(active) WHERE active = true;
CREATE INDEX IF NOT EXISTS idx_security_audit_logs_user_id ON security_audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_security_audit_logs_event_type ON security_audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_security_audit_logs_created_at_desc ON security_audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_rate_limit_tracking_identifier_endpoint ON rate_limit_tracking(identifier, endpoint);
CREATE INDEX IF NOT EXISTS idx_rate_limit_tracking_window_start ON rate_limit_tracking(window_start);

-- Triggers
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_api_keys_updated_at BEFORE UPDATE ON api_keys
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default roles
INSERT INTO roles (name, description) VALUES
    ('admin', 'Full system access'),
    ('compliance_officer', 'Compliance management and oversight'),
    ('auditor', 'Read-only access for auditing'),
    ('viewer', 'Limited read-only access')
ON CONFLICT (name) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, resource, action, description) VALUES
    -- Compliance Records
    ('compliance.read', 'compliance_records', 'read', 'View compliance records'),
    ('compliance.write', 'compliance_records', 'write', 'Create/update compliance records'),
    ('compliance.delete', 'compliance_records', 'delete', 'Delete compliance records'),
    -- Data Subject Rights
    ('data_subject.read', 'data_subject', 'read', 'View data subject information'),
    ('data_subject.export', 'data_subject', 'export', 'Export data subject data'),
    ('data_subject.rectify', 'data_subject', 'rectify', 'Rectify data subject data'),
    -- Human Oversight
    ('oversight.read', 'human_oversight', 'read', 'View oversight requests'),
    ('oversight.approve', 'human_oversight', 'approve', 'Approve/reject oversight requests'),
    -- Risk Assessment
    ('risk.read', 'risk_assessment', 'read', 'View risk assessments'),
    ('risk.write', 'risk_assessment', 'write', 'Create/update risk assessments'),
    -- Data Breaches
    ('breach.read', 'data_breaches', 'read', 'View breach reports'),
    ('breach.write', 'data_breaches', 'write', 'Create/update breach reports'),
    -- Consent
    ('consent.read', 'consent', 'read', 'View consent records'),
    ('consent.write', 'consent', 'write', 'Grant/withdraw consent'),
    -- DPIA
    ('dpia.read', 'dpia', 'read', 'View DPIAs'),
    ('dpia.write', 'dpia', 'write', 'Create/update DPIAs'),
    -- Retention
    ('retention.read', 'retention', 'read', 'View retention policies'),
    ('retention.write', 'retention', 'write', 'Create/update retention policies'),
    -- Monitoring
    ('monitoring.read', 'monitoring', 'read', 'View monitoring events'),
    ('monitoring.write', 'monitoring', 'write', 'Create/update monitoring events'),
    -- Webhooks
    ('webhook.read', 'webhooks', 'read', 'View webhook configurations'),
    ('webhook.write', 'webhooks', 'write', 'Create/update webhooks'),
    ('webhook.delete', 'webhooks', 'delete', 'Delete webhooks'),
    -- System
    ('system.admin', 'system', 'admin', 'Full system administration'),
    ('system.audit', 'system', 'audit', 'View audit logs')
ON CONFLICT (name) DO NOTHING;

-- Assign permissions to roles
-- Admin: all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin'
ON CONFLICT DO NOTHING;

-- Compliance Officer: compliance, oversight, breach, consent, dpia, retention, monitoring
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'compliance_officer'
  AND p.name IN (
    'compliance.read', 'compliance.write',
    'oversight.read', 'oversight.approve',
    'breach.read', 'breach.write',
    'consent.read', 'consent.write',
    'dpia.read', 'dpia.write',
    'retention.read', 'retention.write',
    'monitoring.read', 'monitoring.write',
    'webhook.read', 'webhook.write'
  )
ON CONFLICT DO NOTHING;

-- Auditor: read-only access
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'auditor'
  AND p.action = 'read'
ON CONFLICT DO NOTHING;

-- Viewer: limited read-only
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'viewer'
  AND p.name IN (
    'compliance.read',
    'oversight.read',
    'risk.read',
    'breach.read',
    'monitoring.read'
  )
ON CONFLICT DO NOTHING;

-- ============================================================================
-- ADMIN USER CREATION
-- ============================================================================
-- SECURITY NOTE: Default admin user creation is DISABLED for security.
-- 
-- To create an admin user in production:
-- 1. Use the /api/v1/auth/register endpoint to create a user
-- 2. Manually assign admin role via SQL:
--    INSERT INTO user_roles (user_id, role_id)
--    SELECT u.id, r.id FROM users u, roles r
--    WHERE u.username = 'your_admin_username' AND r.name = 'admin';
--
-- DO NOT use default credentials in production!
--
-- The following INSERT is commented out for security:
-- INSERT INTO users (username, email, password_hash, full_name) VALUES
--     ('admin', 'admin@veridion-nexus.local', '$2b$12$...', 'System Administrator')
-- ON CONFLICT (username) DO NOTHING;

-- Assign admin role to default admin user
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.username = 'admin' AND r.name = 'admin'
ON CONFLICT DO NOTHING;

