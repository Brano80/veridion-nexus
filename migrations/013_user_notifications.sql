-- User Notifications Table (GDPR Article 33, EU AI Act Article 13)
-- Tracks all notifications sent to users for compliance purposes

CREATE TABLE IF NOT EXISTS user_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id VARCHAR(255) NOT NULL UNIQUE,
    user_id VARCHAR(255) NOT NULL,
    notification_type VARCHAR(100) NOT NULL, -- 'DATA_BREACH', 'HIGH_RISK_AI_ACTION', 'AUTOMATED_DECISION', etc.
    channel VARCHAR(50) NOT NULL, -- 'EMAIL', 'SMS', 'IN_APP'
    subject TEXT,
    body TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING', -- 'PENDING', 'SENT', 'FAILED', 'DELIVERED'
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    failure_reason TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    related_entity_type VARCHAR(100), -- 'BREACH', 'COMPLIANCE_RECORD', 'AUTOMATED_DECISION'
    related_entity_id VARCHAR(255), -- breach_id, seal_id, decision_id, etc.
    language VARCHAR(10) NOT NULL DEFAULT 'en', -- ISO 639-1 language code
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_notifications_user_id ON user_notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_user_notifications_type ON user_notifications(notification_type);
CREATE INDEX IF NOT EXISTS idx_user_notifications_status ON user_notifications(status);
CREATE INDEX IF NOT EXISTS idx_user_notifications_created_at ON user_notifications(created_at);
CREATE INDEX IF NOT EXISTS idx_user_notifications_related_entity ON user_notifications(related_entity_type, related_entity_id);
CREATE INDEX IF NOT EXISTS idx_user_notifications_pending ON user_notifications(status, created_at) WHERE status = 'PENDING';

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_user_notifications_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_user_notifications_updated_at
    BEFORE UPDATE ON user_notifications
    FOR EACH ROW
    EXECUTE FUNCTION update_user_notifications_updated_at();

-- Notification Templates Table (for storing reusable templates)
CREATE TABLE IF NOT EXISTS notification_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id VARCHAR(255) NOT NULL UNIQUE,
    template_type VARCHAR(100) NOT NULL, -- 'DATA_BREACH', 'HIGH_RISK_AI_ACTION', etc.
    channel VARCHAR(50) NOT NULL, -- 'EMAIL', 'SMS'
    language VARCHAR(10) NOT NULL DEFAULT 'en',
    subject_template TEXT,
    body_template TEXT NOT NULL,
    variables JSONB, -- JSON schema for required variables
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_notification_templates_type ON notification_templates(template_type, channel, language, is_active);

-- Trigger for notification_templates updated_at
CREATE TRIGGER trigger_update_notification_templates_updated_at
    BEFORE UPDATE ON notification_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_user_notifications_updated_at();

-- Insert default templates for GDPR Article 33 (Data Breach Notification) and EU AI Act Article 13
-- English templates
INSERT INTO notification_templates (template_id, template_type, channel, language, subject_template, body_template, variables) VALUES
('data_breach_email_en', 'DATA_BREACH', 'EMAIL', 'en', 
 'Data Breach Notification - {{breach_type}}',
 'Dear {{user_name}},\n\nWe are writing to inform you of a data breach that may affect your personal data.\n\nBreach Details:\n- Type: {{breach_type}}\n- Detected: {{detected_at}}\n- Description: {{description}}\n\nWe have reported this breach to the relevant supervisory authority within 72 hours as required by GDPR Article 33.\n\nIf you have any questions or concerns, please contact our Data Protection Officer.\n\nBest regards,\nVeridion Nexus Compliance Team',
 '{"breach_type": "string", "user_name": "string", "detected_at": "string", "description": "string"}'::jsonb),
('data_breach_sms_en', 'DATA_BREACH', 'SMS', 'en',
 NULL,
 'Data breach detected: {{breach_type}}. Detected: {{detected_at}}. We have notified the supervisory authority. Contact DPO for details.',
 '{"breach_type": "string", "detected_at": "string"}'::jsonb),
('high_risk_ai_action_email_en', 'HIGH_RISK_AI_ACTION', 'EMAIL', 'en',
 'High-Risk AI Action Notification - {{action_type}}',
 'Dear {{user_name}},\n\nThis is to inform you that a high-risk AI system has processed your data, as required by EU AI Act Article 13.\n\n=== ACTION DETAILS ===\n- Type: {{action_type}}\n- Timestamp: {{timestamp}}\n- Risk Level: {{risk_level}}\n- Purpose: {{purpose}}\n\n=== AI SYSTEM INFORMATION ===\n- How it works: {{ai_functioning}}\n- Risks: {{risks}}\n\n=== YOUR RIGHTS ===\nYou have the right to:\n- Request human review of this decision\n- Object to this processing\n- Request an explanation of the logic involved\n- Lodge a complaint with a supervisory authority\n\nIf you have questions or wish to exercise your rights, please contact us.\n\nBest regards,\nVeridion Nexus Compliance Team',
 '{"action_type": "string", "user_name": "string", "timestamp": "string", "risk_level": "string", "purpose": "string", "ai_functioning": "string", "risks": "string"}'::jsonb),
('high_risk_ai_action_sms_en', 'HIGH_RISK_AI_ACTION', 'SMS', 'en',
 NULL,
 'High-risk AI action: {{action_type}} at {{timestamp}}. Risk: {{risk_level}}. Purpose: {{purpose}}. You have rights to review, object, and request explanation. Contact us for details.',
 '{"action_type": "string", "timestamp": "string", "risk_level": "string", "purpose": "string"}'::jsonb),
('high_risk_ai_action_inapp_en', 'HIGH_RISK_AI_ACTION', 'IN_APP', 'en',
 NULL,
 'High-Risk AI Action: {{action_type}}\nTimestamp: {{timestamp}}\nRisk Level: {{risk_level}}\nPurpose: {{purpose}}\n\nAI Functioning: {{ai_functioning}}\nRisks: {{risks}}\n\nYou can request human review or object to this processing.',
 '{"action_type": "string", "timestamp": "string", "risk_level": "string", "purpose": "string", "ai_functioning": "string", "risks": "string"}'::jsonb),
-- Slovak templates
('high_risk_ai_action_email_sk', 'HIGH_RISK_AI_ACTION', 'EMAIL', 'sk',
 'Notifikácia o vysoko rizikovej akcii AI - {{action_type}}',
 'Vážený používateľ,\n\nTýmto Vás informujeme, že systém AI s vysokým rizikom spracoval Vaše údaje, podľa požiadaviek článku 13 zákona o AI EÚ.\n\n=== DETALY AKCIE ===\n- Typ: {{action_type}}\n- Čas: {{timestamp}}\n- Úroveň rizika: {{risk_level}}\n- Účel: {{purpose}}\n\n=== INFORMÁCIE O SYSTÉME AI ===\n- Ako funguje: {{ai_functioning}}\n- Riziká: {{risks}}\n\n=== VAŠE PRÁVA ===\nMáte právo:\n- Požiadať o ľudský prehľad tohto rozhodnutia\n- Namietať proti tomuto spracovaniu\n- Požiadať o vysvetlenie použitej logiky\n- Podávať sťažnosť dozornému orgánu\n\nAk máte otázky alebo chcete uplatniť svoje práva, kontaktujte nás.\n\nS pozdravom,\nTím Veridion Nexus Compliance',
 '{"action_type": "string", "user_name": "string", "timestamp": "string", "risk_level": "string", "purpose": "string", "ai_functioning": "string", "risks": "string"}'::jsonb)
ON CONFLICT (template_id) DO NOTHING;

