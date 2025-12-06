-- Post-Market Monitoring (EU AI Act Article 72)
-- Migration for monitoring AI systems after market release

-- Monitoring Events (incidents, anomalies, performance issues)
CREATE TABLE IF NOT EXISTS monitoring_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id VARCHAR(255) NOT NULL UNIQUE,
    event_type VARCHAR(100) NOT NULL, -- "INCIDENT", "ANOMALY", "PERFORMANCE_DEGRADATION", "USER_COMPLAINT", "SYSTEM_ERROR"
    severity VARCHAR(50) NOT NULL, -- "LOW", "MEDIUM", "HIGH", "CRITICAL"
    system_id VARCHAR(255) NOT NULL, -- Identifier for the AI system
    system_version VARCHAR(100),
    description TEXT NOT NULL,
    affected_users JSONB, -- Array of affected user IDs
    affected_records_count INTEGER,
    detected_at TIMESTAMP NOT NULL,
    resolved_at TIMESTAMP,
    resolution_status VARCHAR(50) DEFAULT 'OPEN', -- "OPEN", "INVESTIGATING", "RESOLVED", "FALSE_POSITIVE"
    resolution_notes TEXT,
    reported_to_authority BOOLEAN DEFAULT false,
    authority_report_date TIMESTAMP,
    corrective_action_taken TEXT,
    preventive_measures TEXT,
    created_by VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Monitoring Metrics (performance, accuracy, compliance metrics)
CREATE TABLE IF NOT EXISTS monitoring_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id VARCHAR(255) NOT NULL,
    metric_name VARCHAR(100) NOT NULL, -- "ACCURACY", "LATENCY", "ERROR_RATE", "COMPLIANCE_SCORE", "BIAS_DETECTED"
    metric_value DECIMAL(10, 4) NOT NULL,
    metric_unit VARCHAR(50), -- "PERCENTAGE", "MILLISECONDS", "COUNT", "SCORE"
    threshold_min DECIMAL(10, 4),
    threshold_max DECIMAL(10, 4),
    is_within_threshold BOOLEAN,
    measured_at TIMESTAMP NOT NULL,
    system_version VARCHAR(100),
    metadata JSONB, -- Additional context
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- System Health Status (aggregated health status per system)
CREATE TABLE IF NOT EXISTS system_health_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id VARCHAR(255) NOT NULL UNIQUE,
    system_version VARCHAR(100),
    overall_status VARCHAR(50) NOT NULL, -- "HEALTHY", "DEGRADED", "CRITICAL", "OFFLINE"
    compliance_status VARCHAR(50) NOT NULL, -- "COMPLIANT", "NON_COMPLIANT", "AT_RISK"
    last_health_check TIMESTAMP NOT NULL,
    active_incidents_count INTEGER DEFAULT 0,
    critical_incidents_count INTEGER DEFAULT 0,
    last_incident_at TIMESTAMP,
    performance_score DECIMAL(5, 2), -- 0-100 score
    compliance_score DECIMAL(5, 2), -- 0-100 score
    notes TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Monitoring Reports (periodic compliance and performance reports)
CREATE TABLE IF NOT EXISTS monitoring_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id VARCHAR(255) NOT NULL UNIQUE,
    system_id VARCHAR(255) NOT NULL,
    report_type VARCHAR(100) NOT NULL, -- "PERIODIC", "INCIDENT", "COMPLIANCE_AUDIT", "PERFORMANCE_REVIEW"
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    summary TEXT NOT NULL,
    findings JSONB NOT NULL, -- Array of findings
    recommendations JSONB, -- Array of recommendations
    incidents_count INTEGER DEFAULT 0,
    metrics_summary JSONB, -- Summary of key metrics
    compliance_status VARCHAR(50) NOT NULL,
    generated_by VARCHAR(255),
    generated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    approved_by VARCHAR(255),
    approved_at TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_monitoring_events_system_id ON monitoring_events(system_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_event_type ON monitoring_events(event_type);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_severity ON monitoring_events(severity);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_detected_at ON monitoring_events(detected_at);
CREATE INDEX IF NOT EXISTS idx_monitoring_events_resolution_status ON monitoring_events(resolution_status);
CREATE INDEX IF NOT EXISTS idx_monitoring_metrics_system_id ON monitoring_metrics(system_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_metrics_measured_at ON monitoring_metrics(measured_at);
CREATE INDEX IF NOT EXISTS idx_monitoring_metrics_metric_name ON monitoring_metrics(metric_name);
CREATE INDEX IF NOT EXISTS idx_system_health_status_system_id ON system_health_status(system_id);
CREATE INDEX IF NOT EXISTS idx_system_health_status_overall_status ON system_health_status(overall_status);
CREATE INDEX IF NOT EXISTS idx_monitoring_reports_system_id ON monitoring_reports(system_id);
CREATE INDEX IF NOT EXISTS idx_monitoring_reports_report_type ON monitoring_reports(report_type);
CREATE INDEX IF NOT EXISTS idx_monitoring_reports_period_end ON monitoring_reports(period_end);

-- Trigger for updated_at
CREATE TRIGGER update_monitoring_events_updated_at BEFORE UPDATE ON monitoring_events
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_system_health_status_updated_at BEFORE UPDATE ON system_health_status
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to update system health status when events occur
CREATE OR REPLACE FUNCTION update_system_health_from_event()
RETURNS TRIGGER AS $$
BEGIN
    -- Update or insert system health status
    INSERT INTO system_health_status (
        system_id, system_version, overall_status, compliance_status,
        last_health_check, active_incidents_count, critical_incidents_count,
        last_incident_at
    )
    VALUES (
        NEW.system_id,
        NEW.system_version,
        CASE 
            WHEN NEW.severity = 'CRITICAL' THEN 'CRITICAL'
            WHEN NEW.severity = 'HIGH' THEN 'DEGRADED'
            ELSE 'HEALTHY'
        END,
        CASE 
            WHEN NEW.reported_to_authority = true THEN 'NON_COMPLIANT'
            WHEN NEW.severity IN ('HIGH', 'CRITICAL') THEN 'AT_RISK'
            ELSE 'COMPLIANT'
        END,
        CURRENT_TIMESTAMP,
        CASE WHEN NEW.resolution_status != 'RESOLVED' THEN 1 ELSE 0 END,
        CASE WHEN NEW.severity = 'CRITICAL' AND NEW.resolution_status != 'RESOLVED' THEN 1 ELSE 0 END,
        CASE WHEN NEW.severity IN ('HIGH', 'CRITICAL') THEN NEW.detected_at ELSE NULL END
    )
    ON CONFLICT (system_id) DO UPDATE SET
        overall_status = CASE 
            WHEN NEW.severity = 'CRITICAL' THEN 'CRITICAL'
            WHEN NEW.severity = 'HIGH' THEN 'DEGRADED'
            WHEN EXCLUDED.overall_status = 'CRITICAL' THEN 'CRITICAL'
            WHEN EXCLUDED.overall_status = 'DEGRADED' THEN 'DEGRADED'
            ELSE 'HEALTHY'
        END,
        compliance_status = CASE 
            WHEN NEW.reported_to_authority = true THEN 'NON_COMPLIANT'
            WHEN NEW.severity IN ('HIGH', 'CRITICAL') THEN 'AT_RISK'
            WHEN EXCLUDED.compliance_status = 'NON_COMPLIANT' THEN 'NON_COMPLIANT'
            ELSE 'COMPLIANT'
        END,
        last_health_check = CURRENT_TIMESTAMP,
        active_incidents_count = system_health_status.active_incidents_count + 
            CASE WHEN NEW.resolution_status != 'RESOLVED' THEN 1 ELSE 0 END,
        critical_incidents_count = system_health_status.critical_incidents_count + 
            CASE WHEN NEW.severity = 'CRITICAL' AND NEW.resolution_status != 'RESOLVED' THEN 1 ELSE 0 END,
        last_incident_at = CASE 
            WHEN NEW.severity IN ('HIGH', 'CRITICAL') THEN NEW.detected_at 
            ELSE system_health_status.last_incident_at 
        END,
        updated_at = CURRENT_TIMESTAMP;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to automatically update system health when events are created/updated
CREATE TRIGGER monitoring_event_health_update
    AFTER INSERT OR UPDATE ON monitoring_events
    FOR EACH ROW
    EXECUTE FUNCTION update_system_health_from_event();

