-- Retention Period Automation (GDPR Article 5(1)(e) - Storage Limitation)
-- Migration for automatic data deletion after retention periods

-- Retention Policies
CREATE TABLE IF NOT EXISTS retention_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_name VARCHAR(255) NOT NULL UNIQUE,
    data_category VARCHAR(100) NOT NULL, -- "COMPLIANCE_RECORDS", "USER_DATA", "CONSENT", "DPIA", "BREACH_REPORTS"
    retention_period_days INTEGER NOT NULL,
    legal_basis TEXT NOT NULL,
    description TEXT,
    auto_delete BOOLEAN DEFAULT true,
    notification_days_before INTEGER, -- Notify X days before deletion
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Retention Assignments (tracks when records should be deleted)
CREATE TABLE IF NOT EXISTS retention_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type VARCHAR(100) NOT NULL, -- "COMPLIANCE_RECORD", "CONSENT", "DPIA", "BREACH", "USER_DATA"
    record_id VARCHAR(255) NOT NULL, -- ID of the record (seal_id, consent_id, dpia_id, etc.)
    policy_id UUID REFERENCES retention_policies(id) ON DELETE SET NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP,
    deletion_status VARCHAR(50) DEFAULT 'PENDING', -- "PENDING", "SCHEDULED", "DELETED", "EXEMPT"
    exemption_reason TEXT,
    last_notification_sent TIMESTAMP,
    created_by VARCHAR(255),
    UNIQUE(record_type, record_id)
);

-- Retention Exemptions (records that should not be auto-deleted)
CREATE TABLE IF NOT EXISTS retention_exemptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type VARCHAR(100) NOT NULL,
    record_id VARCHAR(255) NOT NULL,
    reason TEXT NOT NULL,
    exempted_by VARCHAR(255) NOT NULL,
    exempted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP, -- Optional: exemption can expire
    UNIQUE(record_type, record_id)
);

-- Retention Deletion Log (audit trail)
CREATE TABLE IF NOT EXISTS retention_deletion_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID REFERENCES retention_assignments(id) ON DELETE SET NULL,
    record_type VARCHAR(100) NOT NULL,
    record_id VARCHAR(255) NOT NULL,
    policy_name VARCHAR(255),
    deleted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deletion_method VARCHAR(50) NOT NULL, -- "AUTO", "MANUAL", "CRYPTO_SHRED"
    deleted_by VARCHAR(255),
    records_affected INTEGER,
    metadata JSONB -- Additional info about what was deleted
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_retention_policies_data_category ON retention_policies(data_category);
CREATE INDEX IF NOT EXISTS idx_retention_assignments_expires_at ON retention_assignments(expires_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_retention_assignments_deletion_status ON retention_assignments(deletion_status);
CREATE INDEX IF NOT EXISTS idx_retention_assignments_record ON retention_assignments(record_type, record_id);
CREATE INDEX IF NOT EXISTS idx_retention_exemptions_record ON retention_exemptions(record_type, record_id);
CREATE INDEX IF NOT EXISTS idx_retention_exemptions_expires_at ON retention_exemptions(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_retention_deletion_log_deleted_at ON retention_deletion_log(deleted_at);

-- Note: Automatic retention assignment can be implemented via triggers
-- or via API calls when records are created

-- Trigger for updated_at
CREATE TRIGGER update_retention_policies_updated_at BEFORE UPDATE ON retention_policies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
