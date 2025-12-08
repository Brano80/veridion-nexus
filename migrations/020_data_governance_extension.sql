-- Data Governance Extension (EU AI Act Article 11)
-- Extends Sovereign Lock with data quality metrics, bias detection, and lineage tracking

CREATE TABLE IF NOT EXISTS data_quality_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seal_id VARCHAR(255) NOT NULL REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    data_source VARCHAR(255),
    metric_type VARCHAR(100) NOT NULL, -- 'COMPLETENESS', 'ACCURACY', 'CONSISTENCY', 'VALIDITY', 'TIMELINESS'
    metric_value DECIMAL(10, 4) NOT NULL, -- 0.0 to 1.0
    threshold DECIMAL(10, 4), -- Alert if below threshold
    measured_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS data_bias_detections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seal_id VARCHAR(255) NOT NULL REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    bias_type VARCHAR(100) NOT NULL, -- 'DEMOGRAPHIC', 'GEOGRAPHIC', 'TEMPORAL', 'REPRESENTATION'
    bias_metric DECIMAL(10, 4) NOT NULL, -- Bias score
    affected_groups JSONB, -- Groups affected by bias
    detected_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    mitigation_applied BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS data_lineage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seal_id VARCHAR(255) NOT NULL REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    source_seal_id VARCHAR(255), -- Parent seal_id if data was derived
    transformation_type VARCHAR(100), -- 'AGGREGATION', 'FILTERING', 'ENRICHMENT', 'ANONYMIZATION'
    transformation_details JSONB,
    lineage_path TEXT[], -- Array of seal_ids showing data flow
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_data_quality_metrics_seal_id ON data_quality_metrics(seal_id);
CREATE INDEX IF NOT EXISTS idx_data_quality_metrics_type ON data_quality_metrics(metric_type);
CREATE INDEX IF NOT EXISTS idx_data_bias_detections_seal_id ON data_bias_detections(seal_id);
CREATE INDEX IF NOT EXISTS idx_data_bias_detections_type ON data_bias_detections(bias_type);
CREATE INDEX IF NOT EXISTS idx_data_lineage_seal_id ON data_lineage(seal_id);
CREATE INDEX IF NOT EXISTS idx_data_lineage_source ON data_lineage(source_seal_id);

