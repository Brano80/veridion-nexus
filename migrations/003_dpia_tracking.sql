-- DPIA Tracking (GDPR Article 35 - Data Protection Impact Assessment)
-- Migration for tracking and managing DPIAs

-- DPIA Records
CREATE TABLE IF NOT EXISTS dpia_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dpia_id VARCHAR(255) NOT NULL UNIQUE,
    activity_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    legal_basis VARCHAR(50) NOT NULL,
    data_categories TEXT[] NOT NULL,
    data_subject_categories TEXT[] NOT NULL,
    processing_purposes TEXT[] NOT NULL,
    risk_level VARCHAR(20) NOT NULL, -- "LOW", "MEDIUM", "HIGH"
    identified_risks JSONB NOT NULL DEFAULT '[]'::jsonb,
    mitigation_measures JSONB NOT NULL DEFAULT '[]'::jsonb,
    residual_risks JSONB NOT NULL DEFAULT '[]'::jsonb,
    consultation_required BOOLEAN DEFAULT false,
    supervisory_authority_consulted BOOLEAN DEFAULT false,
    consultation_date TIMESTAMP,
    consultation_response TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT', -- "DRAFT", "IN_REVIEW", "APPROVED", "REJECTED", "REQUIRES_CONSULTATION"
    reviewed_by VARCHAR(255),
    reviewed_at TIMESTAMP,
    approval_date TIMESTAMP,
    next_review_date TIMESTAMP,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- DPIA History (for audit trail)
CREATE TABLE IF NOT EXISTS dpia_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dpia_id VARCHAR(255) NOT NULL REFERENCES dpia_records(dpia_id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL, -- "CREATED", "UPDATED", "SUBMITTED", "APPROVED", "REJECTED", "CONSULTATION_REQUESTED"
    changed_by VARCHAR(255),
    changed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    previous_state JSONB,
    new_state JSONB,
    comments TEXT
);

-- DPIA Related Processing Activities
CREATE TABLE IF NOT EXISTS dpia_processing_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dpia_id VARCHAR(255) NOT NULL REFERENCES dpia_records(dpia_id) ON DELETE CASCADE,
    processing_activity_id UUID REFERENCES processing_activities(id) ON DELETE SET NULL,
    activity_name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_dpia_records_dpia_id ON dpia_records(dpia_id);
CREATE INDEX IF NOT EXISTS idx_dpia_records_status ON dpia_records(status);
CREATE INDEX IF NOT EXISTS idx_dpia_records_risk_level ON dpia_records(risk_level);
CREATE INDEX IF NOT EXISTS idx_dpia_records_created_at ON dpia_records(created_at);
CREATE INDEX IF NOT EXISTS idx_dpia_records_next_review_date ON dpia_records(next_review_date) WHERE next_review_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_dpia_history_dpia_id ON dpia_history(dpia_id);
CREATE INDEX IF NOT EXISTS idx_dpia_history_changed_at ON dpia_history(changed_at);
CREATE INDEX IF NOT EXISTS idx_dpia_processing_activities_dpia_id ON dpia_processing_activities(dpia_id);

-- Trigger for dpia_history
CREATE OR REPLACE FUNCTION log_dpia_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO dpia_history (
            dpia_id,
            action,
            changed_by,
            new_state
        ) VALUES (
            NEW.dpia_id,
            'CREATED',
            NEW.created_by,
            jsonb_build_object(
                'status', NEW.status,
                'risk_level', NEW.risk_level,
                'activity_name', NEW.activity_name
            )
        );
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.status != NEW.status OR OLD.risk_level != NEW.risk_level THEN
            INSERT INTO dpia_history (
                dpia_id,
                action,
                changed_by,
                previous_state,
                new_state,
                comments
            ) VALUES (
                NEW.dpia_id,
                CASE 
                    WHEN NEW.status = 'APPROVED' THEN 'APPROVED'
                    WHEN NEW.status = 'REJECTED' THEN 'REJECTED'
                    WHEN NEW.status = 'IN_REVIEW' THEN 'SUBMITTED'
                    WHEN NEW.supervisory_authority_consulted = true AND OLD.supervisory_authority_consulted = false THEN 'CONSULTATION_REQUESTED'
                    ELSE 'UPDATED'
                END,
                COALESCE(NEW.reviewed_by, NEW.created_by),
                jsonb_build_object(
                    'status', OLD.status,
                    'risk_level', OLD.risk_level
                ),
                jsonb_build_object(
                    'status', NEW.status,
                    'risk_level', NEW.risk_level
                ),
                NULL
            );
        END IF;
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER dpia_change_trigger
    AFTER INSERT OR UPDATE ON dpia_records
    FOR EACH ROW
    EXECUTE FUNCTION log_dpia_change();

-- Trigger for updated_at
CREATE TRIGGER update_dpia_records_updated_at BEFORE UPDATE ON dpia_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

