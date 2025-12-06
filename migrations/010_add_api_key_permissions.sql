-- Add API Key permissions
INSERT INTO permissions (name, resource, action, description) VALUES
    ('api_key.read', 'api_keys', 'read', 'View API keys'),
    ('api_key.write', 'api_keys', 'write', 'Create/update API keys'),
    ('api_key.delete', 'api_keys', 'delete', 'Delete/revoke API keys')
ON CONFLICT (name) DO NOTHING;

-- Assign API key permissions to admin role (admin already has all permissions, but explicit for clarity)
-- Admin role already gets all permissions via the existing migration

-- Assign API key permissions to compliance_officer role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'compliance_officer'
  AND p.name IN ('api_key.read', 'api_key.write', 'api_key.delete')
ON CONFLICT DO NOTHING;

