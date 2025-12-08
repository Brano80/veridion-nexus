-- Automated Decisions Table (GDPR Article 22)
-- Tracks automated decision-making that produces legal effects or significantly affects individuals

CREATE TABLE IF NOT EXISTS automated_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    decision_id VARCHAR(255) NOT NULL UNIQUE,
    user_id VARCHAR(255) NOT NULL,
    seal_id VARCHAR(255) NOT NULL, -- Reference to compliance record
    action_type VARCHAR(255) NOT NULL, -- e.g., "credit_scoring", "loan_approval", "job_screening"
    decision_outcome VARCHAR(100) NOT NULL, -- 'APPROVED', 'REJECTED', 'PENDING', 'CONDITIONAL'
    decision_reasoning TEXT, -- Explanation of how the decision was reached
    legal_effect VARCHAR(255), -- Description of legal effect (e.g., "Loan denied", "Credit limit reduced")
    significant_impact BOOLEAN NOT NULL DEFAULT true, -- Whether decision significantly affects the individual
    decision_timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING_REVIEW', -- 'PENDING_REVIEW', 'UNDER_REVIEW', 'REVIEWED', 'APPEALED', 'OVERRIDDEN'
    human_review_required BOOLEAN NOT NULL DEFAULT true,
    human_reviewer_id VARCHAR(255),
    human_reviewed_at TIMESTAMPTZ,
    human_review_notes TEXT,
    review_decision VARCHAR(50), -- 'UPHELD', 'OVERRIDDEN', 'REQUIRES_APPEAL'
    appeal_requested BOOLEAN NOT NULL DEFAULT false,
    appeal_requested_at TIMESTAMPTZ,
    appeal_reason TEXT,
    appeal_resolved_at TIMESTAMPTZ,
    appeal_resolution TEXT,
    notification_sent BOOLEAN NOT NULL DEFAULT false,
    notification_sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_automated_decisions_user_id ON automated_decisions(user_id);
CREATE INDEX IF NOT EXISTS idx_automated_decisions_seal_id ON automated_decisions(seal_id);
CREATE INDEX IF NOT EXISTS idx_automated_decisions_status ON automated_decisions(status);
CREATE INDEX IF NOT EXISTS idx_automated_decisions_decision_timestamp ON automated_decisions(decision_timestamp);
CREATE INDEX IF NOT EXISTS idx_automated_decisions_pending_review ON automated_decisions(status, decision_timestamp) WHERE status = 'PENDING_REVIEW';
CREATE INDEX IF NOT EXISTS idx_automated_decisions_appeal_requested ON automated_decisions(appeal_requested, appeal_requested_at) WHERE appeal_requested = true;

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_automated_decisions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_automated_decisions_updated_at
    BEFORE UPDATE ON automated_decisions
    FOR EACH ROW
    EXECUTE FUNCTION update_automated_decisions_updated_at();

-- Decision History Table (for audit trail)
CREATE TABLE IF NOT EXISTS decision_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    decision_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'CREATED', 'REVIEWED', 'APPEALED', 'OVERRIDDEN', 'NOTIFIED'
    old_status VARCHAR(50),
    new_status VARCHAR(50),
    changed_by VARCHAR(255), -- User ID or system identifier
    reason TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB -- Additional context
);

CREATE INDEX IF NOT EXISTS idx_decision_history_decision_id ON decision_history(decision_id);
CREATE INDEX IF NOT EXISTS idx_decision_history_user_id ON decision_history(user_id);
CREATE INDEX IF NOT EXISTS idx_decision_history_changed_at ON decision_history(changed_at);

-- Function to log decision changes
CREATE OR REPLACE FUNCTION log_decision_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO decision_history (
            decision_id, user_id, action, new_status, changed_by, reason, metadata
        ) VALUES (
            NEW.decision_id, NEW.user_id, 'CREATED', NEW.status, 
            COALESCE(NEW.human_reviewer_id, 'SYSTEM'), NEW.decision_reasoning,
            jsonb_build_object(
                'action_type', NEW.action_type,
                'decision_outcome', NEW.decision_outcome,
                'legal_effect', NEW.legal_effect
            )
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO decision_history (
            decision_id, user_id, action, old_status, new_status, changed_by, reason, metadata
        ) VALUES (
            NEW.decision_id, NEW.user_id,
            CASE 
                WHEN NEW.status = 'UNDER_REVIEW' AND OLD.status = 'PENDING_REVIEW' THEN 'REVIEWED'
                WHEN NEW.status = 'REVIEWED' AND OLD.status = 'UNDER_REVIEW' THEN 'REVIEWED'
                WHEN NEW.appeal_requested = true AND OLD.appeal_requested = false THEN 'APPEALED'
                WHEN NEW.review_decision = 'OVERRIDDEN' THEN 'OVERRIDDEN'
                WHEN NEW.notification_sent = true AND OLD.notification_sent = false THEN 'NOTIFIED'
                ELSE 'MODIFIED'
            END,
            OLD.status, NEW.status, COALESCE(NEW.human_reviewer_id, 'SYSTEM'), 
            COALESCE(NEW.human_review_notes, NEW.appeal_reason, NEW.appeal_resolution),
            jsonb_build_object(
                'old_decision_outcome', OLD.decision_outcome,
                'new_decision_outcome', NEW.decision_outcome,
                'review_decision', NEW.review_decision
            )
        );
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to log decision changes
CREATE TRIGGER trigger_log_decision_change
    AFTER INSERT OR UPDATE ON automated_decisions
    FOR EACH ROW
    EXECUTE FUNCTION log_decision_change();

