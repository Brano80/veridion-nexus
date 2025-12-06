-- Performance Optimization Migration
-- Adds indexes, materialized views, and query optimization helpers

-- ========== ADDITIONAL INDEXES FOR PERFORMANCE ==========

-- Compliance records indexes
CREATE INDEX IF NOT EXISTS idx_compliance_records_timestamp_desc ON compliance_records(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_records_status ON compliance_records(status);
CREATE INDEX IF NOT EXISTS idx_compliance_records_risk_level ON compliance_records(risk_level);
CREATE INDEX IF NOT EXISTS idx_compliance_records_user_id ON compliance_records(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_compliance_records_human_oversight ON compliance_records(human_oversight_status) WHERE human_oversight_status IS NOT NULL;

-- Risk assessments indexes
CREATE INDEX IF NOT EXISTS idx_risk_assessments_assessed_at_desc ON risk_assessments(assessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_risk_assessments_risk_level ON risk_assessments(risk_level);

-- Human oversight indexes
CREATE INDEX IF NOT EXISTS idx_human_oversight_status ON human_oversight(status);
CREATE INDEX IF NOT EXISTS idx_human_oversight_updated_at ON human_oversight(updated_at DESC);

-- Data breaches indexes
CREATE INDEX IF NOT EXISTS idx_data_breaches_detected_at_desc ON data_breaches(detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_data_breaches_status ON data_breaches(status);
CREATE INDEX IF NOT EXISTS idx_data_breaches_breach_type ON data_breaches(breach_type);

-- Consent records indexes
CREATE INDEX IF NOT EXISTS idx_consent_records_user_id_consent_type ON consent_records(user_id, consent_type);
CREATE INDEX IF NOT EXISTS idx_consent_records_granted ON consent_records(granted) WHERE granted = true;
CREATE INDEX IF NOT EXISTS idx_consent_records_expires_at ON consent_records(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_consent_records_created_at_desc ON consent_records(created_at DESC);

-- DPIA records indexes
CREATE INDEX IF NOT EXISTS idx_dpia_records_status ON dpia_records(status);
CREATE INDEX IF NOT EXISTS idx_dpia_records_risk_level ON dpia_records(risk_level);
CREATE INDEX IF NOT EXISTS idx_dpia_records_created_at_desc ON dpia_records(created_at DESC);

-- Retention indexes
CREATE INDEX IF NOT EXISTS idx_retention_assignments_expires_at ON retention_assignments(expires_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_retention_assignments_deletion_status ON retention_assignments(deletion_status);
CREATE INDEX IF NOT EXISTS idx_retention_assignments_record_type_id ON retention_assignments(record_type, record_id);

-- Monitoring events indexes
CREATE INDEX IF NOT EXISTS idx_monitoring_events_detected_at_desc ON monitoring_events(detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_resolution_status ON monitoring_events(resolution_status);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_severity ON monitoring_events(severity);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_system_id ON monitoring_events(system_id);

-- User data index
CREATE INDEX IF NOT EXISTS idx_user_data_index_user_id ON user_data_index(user_id);
CREATE INDEX IF NOT EXISTS idx_user_data_index_seal_id ON user_data_index(seal_id);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_compliance_records_user_status ON compliance_records(user_id, status) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_monitoring_events_system_status ON monitoring_events(system_id, resolution_status);

-- ========== MATERIALIZED VIEWS FOR REPORTING ==========

-- Daily compliance summary
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

-- System health summary
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_system_health_summary AS
SELECT
    system_id,
    COUNT(*) as total_events,
    COUNT(*) FILTER (WHERE resolution_status = 'OPEN') as open_events,
    COUNT(*) FILTER (WHERE severity = 'CRITICAL') as critical_events,
    MAX(detected_at) as last_event_at
FROM monitoring_events
GROUP BY system_id;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_system_health_system_id ON mv_system_health_summary(system_id);

-- ========== FUNCTIONS FOR QUERY OPTIMIZATION ==========

-- Function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_materialized_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_compliance_summary;
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_system_health_summary;
END;
$$ LANGUAGE plpgsql;

-- Function to analyze tables
CREATE OR REPLACE FUNCTION analyze_tables()
RETURNS void AS $$
BEGIN
    ANALYZE compliance_records;
    ANALYZE risk_assessments;
    ANALYZE human_oversight;
    ANALYZE data_breaches;
    ANALYZE consent_records;
    ANALYZE dpia_records;
    ANALYZE retention_assignments;
    ANALYZE monitoring_events;
END;
$$ LANGUAGE plpgsql;

-- ========== PARTITIONING PREPARATION (for future scaling) ==========

-- Note: Partitioning would require table recreation, so we prepare the structure
-- but don't implement it yet to avoid breaking existing data

-- ========== QUERY PERFORMANCE HELPERS ==========

-- View for slow queries (requires pg_stat_statements extension)
-- This is informational - actual slow query logging should be configured in postgresql.conf

COMMENT ON MATERIALIZED VIEW mv_daily_compliance_summary IS 'Daily compliance metrics summary - refresh with REFRESH MATERIALIZED VIEW mv_daily_compliance_summary';
COMMENT ON MATERIALIZED VIEW mv_system_health_summary IS 'System health metrics summary - refresh with REFRESH MATERIALIZED VIEW mv_system_health_summary';
