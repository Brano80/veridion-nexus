-- Fix timestamp types to TIMESTAMPTZ for compatibility with Rust chrono::DateTime<Utc>
-- This migration converts all TIMESTAMP columns to TIMESTAMPTZ in compliance_records table

-- Drop materialized view that depends on timestamp column
DROP MATERIALIZED VIEW IF EXISTS mv_daily_compliance_summary;

-- Alter compliance_records table
ALTER TABLE compliance_records 
    ALTER COLUMN timestamp TYPE TIMESTAMPTZ USING timestamp AT TIME ZONE 'UTC',
    ALTER COLUMN notification_timestamp TYPE TIMESTAMPTZ USING notification_timestamp AT TIME ZONE 'UTC',
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';

-- Recreate materialized view
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_daily_compliance_summary AS
SELECT
    DATE(timestamp) as date,
    COUNT(*) as total_actions,
    COUNT(*) FILTER (WHERE status LIKE '%BLOCKED%') as blocked_count,
    COUNT(*) FILTER (WHERE risk_level = 'HIGH') as high_risk_count,
    COUNT(*) FILTER (WHERE human_oversight_status = 'PENDING') as pending_oversight_count
FROM compliance_records
GROUP BY DATE(timestamp);

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_daily_compliance_date ON mv_daily_compliance_summary(date);

