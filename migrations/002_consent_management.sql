-- Consent Management (GDPR Articles 6, 7)
-- Migration for consent tracking and management

-- Consent Records
CREATE TABLE IF NOT EXISTS consent_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    consent_type VARCHAR(100) NOT NULL, -- "PROCESSING", "STORAGE", "TRANSFER", "MARKETING"
    purpose TEXT NOT NULL,
    legal_basis VARCHAR(50) NOT NULL, -- "CONSENT", "CONTRACT", "LEGAL_OBLIGATION", "VITAL_INTERESTS", "PUBLIC_TASK", "LEGITIMATE_INTERESTS"
    granted BOOLEAN NOT NULL DEFAULT false,
    granted_at TIMESTAMP,
    withdrawn_at TIMESTAMP,
    expires_at TIMESTAMP,
    consent_method VARCHAR(50), -- "EXPLICIT", "IMPLICIT", "OPT_IN", "OPT_OUT"
    ip_address VARCHAR(45), -- IPv4 or IPv6
    user_agent TEXT,
    consent_text TEXT, -- The actual consent text shown to user
    version INTEGER NOT NULL DEFAULT 1, -- For tracking consent text changes
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Consent History (for audit trail)
CREATE TABLE IF NOT EXISTS consent_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    consent_record_id UUID NOT NULL REFERENCES consent_records(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL, -- "GRANTED", "WITHDRAWN", "UPDATED", "EXPIRED"
    changed_by VARCHAR(255), -- User ID or system
    changed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    previous_state JSONB, -- Previous consent state
    new_state JSONB, -- New consent state
    reason TEXT
);

-- Processing Activities (GDPR Article 30 - Record of Processing Activities)
CREATE TABLE IF NOT EXISTS processing_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_name VARCHAR(255) NOT NULL,
    purpose TEXT NOT NULL,
    legal_basis VARCHAR(50) NOT NULL,
    data_categories TEXT[], -- Array of data categories processed
    data_subject_categories TEXT[], -- Array of data subject categories
    recipients TEXT[], -- Who receives the data
    third_country_transfers BOOLEAN DEFAULT false,
    third_countries TEXT[], -- List of third countries if transfers occur
    retention_period_days INTEGER,
    security_measures TEXT[], -- Security measures in place
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_consent_records_user_id ON consent_records(user_id);
CREATE INDEX IF NOT EXISTS idx_consent_records_consent_type ON consent_records(consent_type);
CREATE INDEX IF NOT EXISTS idx_consent_records_granted ON consent_records(granted) WHERE granted = true;
CREATE INDEX IF NOT EXISTS idx_consent_records_expires_at ON consent_records(expires_at) WHERE expires_at IS NOT NULL;

-- Partial unique index: Ensure one active consent per user per type
-- This enforces uniqueness only for granted consents that haven't been withdrawn
CREATE UNIQUE INDEX IF NOT EXISTS idx_consent_records_unique_active 
    ON consent_records(user_id, consent_type, version) 
    WHERE granted = true AND withdrawn_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_consent_history_consent_record_id ON consent_history(consent_record_id);
CREATE INDEX IF NOT EXISTS idx_consent_history_changed_at ON consent_history(changed_at);
CREATE INDEX IF NOT EXISTS idx_processing_activities_legal_basis ON processing_activities(legal_basis);

-- Trigger for consent_history
CREATE OR REPLACE FUNCTION log_consent_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF OLD.granted != NEW.granted OR OLD.withdrawn_at IS DISTINCT FROM NEW.withdrawn_at THEN
            INSERT INTO consent_history (
                consent_record_id,
                action,
                changed_by,
                previous_state,
                new_state
            ) VALUES (
                NEW.id,
                CASE 
                    WHEN NEW.granted = true AND OLD.granted = false THEN 'GRANTED'
                    WHEN NEW.granted = false AND OLD.granted = true THEN 'WITHDRAWN'
                    ELSE 'UPDATED'
                END,
                NEW.user_id,
                jsonb_build_object(
                    'granted', OLD.granted,
                    'withdrawn_at', OLD.withdrawn_at,
                    'expires_at', OLD.expires_at
                ),
                jsonb_build_object(
                    'granted', NEW.granted,
                    'withdrawn_at', NEW.withdrawn_at,
                    'expires_at', NEW.expires_at
                )
            );
        END IF;
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER consent_change_trigger
    AFTER UPDATE ON consent_records
    FOR EACH ROW
    EXECUTE FUNCTION log_consent_change();

-- Trigger for updated_at
CREATE TRIGGER update_consent_records_updated_at BEFORE UPDATE ON consent_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_processing_activities_updated_at BEFORE UPDATE ON processing_activities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

