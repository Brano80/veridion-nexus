-- Processing Objections Table (GDPR Article 21)
-- Allows data subjects to object to processing of their personal data

CREATE TABLE IF NOT EXISTS processing_objections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    objection_id VARCHAR(255) NOT NULL UNIQUE,
    user_id VARCHAR(255) NOT NULL,
    objection_type VARCHAR(100) NOT NULL, -- 'FULL', 'PARTIAL', 'SPECIFIC_ACTION', 'DIRECT_MARKETING', 'PROFILING'
    objected_actions JSONB, -- Array of action types to object (e.g., ["credit_scoring", "automated_decision"])
    legal_basis VARCHAR(100), -- Legal basis being objected to (e.g., "LEGITIMATE_INTERESTS", "PUBLIC_TASK")
    reason TEXT, -- Reason for objection (optional)
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE', -- 'ACTIVE', 'WITHDRAWN', 'REJECTED', 'RESOLVED'
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    withdrawn_at TIMESTAMPTZ,
    withdrawn_by VARCHAR(255), -- User ID or system identifier who withdrew the objection
    withdraw_reason TEXT, -- Reason for withdrawing objection
    rejected_at TIMESTAMPTZ,
    rejected_by VARCHAR(255), -- User ID or system identifier who rejected the objection
    rejection_reason TEXT, -- Reason for rejection (must be provided per GDPR Article 21(1))
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one active objection per user at a time (unless different types)
    CONSTRAINT unique_active_objection UNIQUE (user_id, objection_type, status) 
        DEFERRABLE INITIALLY DEFERRED
);

-- Partial unique index for active objections only
CREATE UNIQUE INDEX IF NOT EXISTS idx_processing_objections_active_user_type 
    ON processing_objections(user_id, objection_type) 
    WHERE status = 'ACTIVE';

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_processing_objections_user_id ON processing_objections(user_id);
CREATE INDEX IF NOT EXISTS idx_processing_objections_status ON processing_objections(status);
CREATE INDEX IF NOT EXISTS idx_processing_objections_type ON processing_objections(objection_type);
CREATE INDEX IF NOT EXISTS idx_processing_objections_requested_at ON processing_objections(requested_at);
CREATE INDEX IF NOT EXISTS idx_processing_objections_active ON processing_objections(user_id, status) WHERE status = 'ACTIVE';

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_processing_objections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_processing_objections_updated_at
    BEFORE UPDATE ON processing_objections
    FOR EACH ROW
    EXECUTE FUNCTION update_processing_objections_updated_at();

-- Objection History Table (for audit trail)
CREATE TABLE IF NOT EXISTS objection_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    objection_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'OBJECTED', 'WITHDRAWN', 'REJECTED', 'RESOLVED', 'MODIFIED'
    old_status VARCHAR(50),
    new_status VARCHAR(50),
    changed_by VARCHAR(255), -- User ID or system identifier
    reason TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB -- Additional context
);

CREATE INDEX IF NOT EXISTS idx_objection_history_objection_id ON objection_history(objection_id);
CREATE INDEX IF NOT EXISTS idx_objection_history_user_id ON objection_history(user_id);
CREATE INDEX IF NOT EXISTS idx_objection_history_changed_at ON objection_history(changed_at);

-- Function to log objection changes
CREATE OR REPLACE FUNCTION log_objection_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO objection_history (
            objection_id, user_id, action, new_status, changed_by, reason, metadata
        ) VALUES (
            NEW.objection_id, NEW.user_id, 'OBJECTED', NEW.status, 
            COALESCE(NEW.withdrawn_by, NEW.rejected_by, 'SYSTEM'), NEW.reason,
            jsonb_build_object(
                'objection_type', NEW.objection_type,
                'objected_actions', NEW.objected_actions,
                'legal_basis', NEW.legal_basis
            )
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO objection_history (
            objection_id, user_id, action, old_status, new_status, changed_by, reason, metadata
        ) VALUES (
            NEW.objection_id, NEW.user_id,
            CASE 
                WHEN NEW.status = 'WITHDRAWN' AND OLD.status = 'ACTIVE' THEN 'WITHDRAWN'
                WHEN NEW.status = 'REJECTED' AND OLD.status = 'ACTIVE' THEN 'REJECTED'
                WHEN NEW.status = 'RESOLVED' AND OLD.status = 'ACTIVE' THEN 'RESOLVED'
                WHEN NEW.status != OLD.status THEN 'MODIFIED'
                ELSE 'MODIFIED'
            END,
            OLD.status, NEW.status, COALESCE(NEW.withdrawn_by, NEW.rejected_by, 'SYSTEM'), 
            COALESCE(NEW.withdraw_reason, NEW.rejection_reason, NEW.resolution_notes),
            jsonb_build_object(
                'old_objection_type', OLD.objection_type,
                'new_objection_type', NEW.objection_type,
                'old_objected_actions', OLD.objected_actions,
                'new_objected_actions', NEW.objected_actions
            )
        );
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to log objection changes
CREATE TRIGGER trigger_log_objection_change
    AFTER INSERT OR UPDATE ON processing_objections
    FOR EACH ROW
    EXECUTE FUNCTION log_objection_change();

