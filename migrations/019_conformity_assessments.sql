-- Conformity Assessments (EU AI Act Article 8)
-- Tracks conformity assessments for AI systems

CREATE TABLE IF NOT EXISTS conformity_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id VARCHAR(255) NOT NULL UNIQUE,
    system_id VARCHAR(255) NOT NULL,
    system_name VARCHAR(255) NOT NULL,
    assessment_type VARCHAR(100) NOT NULL, -- 'SELF_ASSESSMENT', 'THIRD_PARTY', 'NOTIFIED_BODY'
    assessment_date TIMESTAMPTZ NOT NULL,
    expiration_date TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'PASSED', 'FAILED', 'EXPIRED'
    assessment_result JSONB, -- Detailed assessment results
    assessor_name VARCHAR(255),
    assessor_contact VARCHAR(255),
    certificate_number VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_conformity_assessments_system_id ON conformity_assessments(system_id);
CREATE INDEX IF NOT EXISTS idx_conformity_assessments_status ON conformity_assessments(status);
CREATE INDEX IF NOT EXISTS idx_conformity_assessments_expiration ON conformity_assessments(expiration_date) WHERE expiration_date IS NOT NULL;
-- Note: Cannot use CURRENT_TIMESTAMP in index predicate (must be IMMUTABLE)
-- Use idx_conformity_assessments_expiration and filter in queries: WHERE expiration_date > CURRENT_TIMESTAMP AND expiration_date < CURRENT_TIMESTAMP + INTERVAL '30 days'

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_conformity_assessments_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_conformity_assessments_updated_at
    BEFORE UPDATE ON conformity_assessments
    FOR EACH ROW
    EXECUTE FUNCTION update_conformity_assessments_updated_at();

