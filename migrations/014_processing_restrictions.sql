-- Processing Restrictions Table (GDPR Article 18)
-- Allows data subjects to request restriction of processing of their personal data

CREATE TABLE IF NOT EXISTS processing_restrictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    restriction_id VARCHAR(255) NOT NULL UNIQUE,
    user_id VARCHAR(255) NOT NULL,
    restriction_type VARCHAR(100) NOT NULL, -- 'FULL', 'PARTIAL', 'SPECIFIC_ACTION'
    restricted_actions JSONB, -- Array of action types to restrict (e.g., ["credit_scoring", "automated_decision"])
    reason TEXT, -- Reason for restriction (optional)
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE', -- 'ACTIVE', 'LIFTED', 'EXPIRED'
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    lifted_at TIMESTAMPTZ,
    lifted_by VARCHAR(255), -- User ID or system identifier who lifted the restriction
    lift_reason TEXT, -- Reason for lifting restriction
    expires_at TIMESTAMPTZ, -- Optional expiration date
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure only one active restriction per user at a time
    CONSTRAINT unique_active_restriction UNIQUE (user_id, status) 
        DEFERRABLE INITIALLY DEFERRED
);

-- Partial unique index for active restrictions only
CREATE UNIQUE INDEX IF NOT EXISTS idx_processing_restrictions_active_user 
    ON processing_restrictions(user_id) 
    WHERE status = 'ACTIVE';

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_processing_restrictions_user_id ON processing_restrictions(user_id);
CREATE INDEX IF NOT EXISTS idx_processing_restrictions_status ON processing_restrictions(status);
CREATE INDEX IF NOT EXISTS idx_processing_restrictions_requested_at ON processing_restrictions(requested_at);
CREATE INDEX IF NOT EXISTS idx_processing_restrictions_active ON processing_restrictions(user_id, status) WHERE status = 'ACTIVE';

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_processing_restrictions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_processing_restrictions_updated_at
    BEFORE UPDATE ON processing_restrictions
    FOR EACH ROW
    EXECUTE FUNCTION update_processing_restrictions_updated_at();

-- Restriction History Table (for audit trail)
CREATE TABLE IF NOT EXISTS restriction_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    restriction_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'RESTRICTED', 'LIFTED', 'MODIFIED', 'EXPIRED'
    old_status VARCHAR(50),
    new_status VARCHAR(50),
    changed_by VARCHAR(255), -- User ID or system identifier
    reason TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB -- Additional context
);

CREATE INDEX IF NOT EXISTS idx_restriction_history_restriction_id ON restriction_history(restriction_id);
CREATE INDEX IF NOT EXISTS idx_restriction_history_user_id ON restriction_history(user_id);
CREATE INDEX IF NOT EXISTS idx_restriction_history_changed_at ON restriction_history(changed_at);

-- Function to log restriction changes
CREATE OR REPLACE FUNCTION log_restriction_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO restriction_history (
            restriction_id, user_id, action, new_status, changed_by, reason, metadata
        ) VALUES (
            NEW.restriction_id, NEW.user_id, 'RESTRICTED', NEW.status, 
            COALESCE(NEW.lifted_by, 'SYSTEM'), NEW.reason,
            jsonb_build_object(
                'restriction_type', NEW.restriction_type,
                'restricted_actions', NEW.restricted_actions
            )
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO restriction_history (
            restriction_id, user_id, action, old_status, new_status, changed_by, reason, metadata
        ) VALUES (
            NEW.restriction_id, NEW.user_id,
            CASE 
                WHEN NEW.status = 'LIFTED' AND OLD.status = 'ACTIVE' THEN 'LIFTED'
                WHEN NEW.status != OLD.status THEN 'MODIFIED'
                ELSE 'MODIFIED'
            END,
            OLD.status, NEW.status, COALESCE(NEW.lifted_by, 'SYSTEM'), NEW.lift_reason,
            jsonb_build_object(
                'old_restriction_type', OLD.restriction_type,
                'new_restriction_type', NEW.restriction_type,
                'old_restricted_actions', OLD.restricted_actions,
                'new_restricted_actions', NEW.restricted_actions
            )
        );
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to log restriction changes
CREATE TRIGGER trigger_log_restriction_change
    AFTER INSERT OR UPDATE ON processing_restrictions
    FOR EACH ROW
    EXECUTE FUNCTION log_restriction_change();

