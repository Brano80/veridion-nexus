-- Veridion Nexus Database Schema
-- Initial migration for High-Risk AI compliance system

-- Compliance Records (Annex IV documentation)
CREATE TABLE IF NOT EXISTS compliance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    agent_id VARCHAR(255) NOT NULL,
    action_summary TEXT NOT NULL,
    seal_id VARCHAR(255) NOT NULL UNIQUE,
    status VARCHAR(100) NOT NULL,
    user_notified BOOLEAN,
    notification_timestamp TIMESTAMP,
    human_oversight_status VARCHAR(50),
    risk_level VARCHAR(20),
    user_id VARCHAR(255),
    tx_id VARCHAR(255) NOT NULL,
    payload_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Risk Assessments (EU AI Act Article 9)
CREATE TABLE IF NOT EXISTS risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seal_id VARCHAR(255) NOT NULL UNIQUE REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    risk_level VARCHAR(20) NOT NULL,
    risk_factors JSONB NOT NULL DEFAULT '[]'::jsonb,
    mitigation_actions JSONB NOT NULL DEFAULT '[]'::jsonb,
    assessed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Human Oversight (EU AI Act Article 14)
CREATE TABLE IF NOT EXISTS human_oversight (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seal_id VARCHAR(255) NOT NULL UNIQUE REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    reviewer_id VARCHAR(255),
    decided_at TIMESTAMP,
    comments TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Data Breaches (GDPR Articles 33-34)
CREATE TABLE IF NOT EXISTS data_breaches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    breach_id VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    breach_type VARCHAR(100) NOT NULL,
    affected_users JSONB NOT NULL DEFAULT '[]'::jsonb,
    detected_at TIMESTAMP NOT NULL,
    affected_records_count INTEGER,
    status VARCHAR(50) NOT NULL DEFAULT 'REPORTED',
    authority_notified_at TIMESTAMP,
    users_notified_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- User Data Index (for GDPR data subject rights)
CREATE TABLE IF NOT EXISTS user_data_index (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    seal_id VARCHAR(255) NOT NULL REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, seal_id)
);

-- Encrypted Log Keys (for Crypto-Shredder)
CREATE TABLE IF NOT EXISTS encrypted_log_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    log_id VARCHAR(255) NOT NULL UNIQUE,
    wrapped_dek BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    shredded_at TIMESTAMP
);

-- System Configuration
CREATE TABLE IF NOT EXISTS system_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(255) NOT NULL UNIQUE,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_compliance_records_seal_id ON compliance_records(seal_id);
CREATE INDEX IF NOT EXISTS idx_compliance_records_user_id ON compliance_records(user_id);
CREATE INDEX IF NOT EXISTS idx_compliance_records_timestamp ON compliance_records(timestamp);
CREATE INDEX IF NOT EXISTS idx_compliance_records_status ON compliance_records(status);
CREATE INDEX IF NOT EXISTS idx_user_data_index_user_id ON user_data_index(user_id);
CREATE INDEX IF NOT EXISTS idx_user_data_index_seal_id ON user_data_index(seal_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_log_keys_log_id ON encrypted_log_keys(log_id);
CREATE INDEX IF NOT EXISTS idx_encrypted_log_keys_shredded_at ON encrypted_log_keys(shredded_at) WHERE shredded_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_data_breaches_breach_id ON data_breaches(breach_id);
CREATE INDEX IF NOT EXISTS idx_data_breaches_detected_at ON data_breaches(detected_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_compliance_records_updated_at BEFORE UPDATE ON compliance_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_human_oversight_updated_at BEFORE UPDATE ON human_oversight
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

