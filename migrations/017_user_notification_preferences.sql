-- User Notification Preferences (EU AI Act Article 13)
-- Allows users to configure their notification preferences for transparency notifications

CREATE TABLE IF NOT EXISTS user_notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    notification_type VARCHAR(100) NOT NULL, -- 'HIGH_RISK_AI_ACTION', 'AUTOMATED_DECISION', 'DATA_BREACH', etc.
    preferred_channels JSONB NOT NULL DEFAULT '["EMAIL"]'::jsonb, -- Array of channels: ["EMAIL", "SMS", "IN_APP"]
    language VARCHAR(10) NOT NULL DEFAULT 'en', -- ISO 639-1 language code
    enabled BOOLEAN NOT NULL DEFAULT true, -- Whether this notification type is enabled
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, notification_type)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_notification_preferences_user_id ON user_notification_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_user_notification_preferences_type ON user_notification_preferences(notification_type);
CREATE INDEX IF NOT EXISTS idx_user_notification_preferences_enabled ON user_notification_preferences(user_id, enabled) WHERE enabled = true;

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_user_notification_preferences_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER trigger_update_user_notification_preferences_updated_at
    BEFORE UPDATE ON user_notification_preferences
    FOR EACH ROW
    EXECUTE FUNCTION update_user_notification_preferences_updated_at();

-- Insert default preferences for common notification types
-- These will be used if user hasn't set their own preferences
INSERT INTO user_notification_preferences (user_id, notification_type, preferred_channels, language, enabled) VALUES
('DEFAULT', 'HIGH_RISK_AI_ACTION', '["EMAIL", "IN_APP"]'::jsonb, 'en', true),
('DEFAULT', 'AUTOMATED_DECISION', '["EMAIL", "SMS", "IN_APP"]'::jsonb, 'en', true),
('DEFAULT', 'DATA_BREACH', '["EMAIL", "SMS"]'::jsonb, 'en', true)
ON CONFLICT (user_id, notification_type) DO NOTHING;

