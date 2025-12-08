-- Data Recipients Tracking (GDPR Article 19)
-- Tracks recipients of personal data for notification purposes

CREATE TABLE IF NOT EXISTS data_recipients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    seal_id VARCHAR(255) NOT NULL REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    recipient_type VARCHAR(50) NOT NULL, -- 'THIRD_PARTY', 'SUBPROCESSOR', 'AUTHORITY', 'OTHER'
    recipient_name VARCHAR(255),
    recipient_contact VARCHAR(255), -- email or identifier
    data_categories JSONB, -- Categories of data shared
    purpose TEXT,
    shared_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_data_recipients_user_id ON data_recipients(user_id);
CREATE INDEX IF NOT EXISTS idx_data_recipients_seal_id ON data_recipients(seal_id);
CREATE INDEX IF NOT EXISTS idx_data_recipients_shared_at ON data_recipients(shared_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_data_recipients_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_data_recipients_updated_at
    BEFORE UPDATE ON data_recipients
    FOR EACH ROW
    EXECUTE FUNCTION update_data_recipients_updated_at();

