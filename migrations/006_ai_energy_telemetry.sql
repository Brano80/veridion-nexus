-- AI Energy Telemetry (EU AI Act Article 40 - Energy Efficiency)
-- Migration for tracking energy consumption and carbon footprint of AI operations

-- AI Energy Telemetry
CREATE TABLE IF NOT EXISTS ai_energy_telemetry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seal_id VARCHAR(255) NOT NULL REFERENCES compliance_records(seal_id) ON DELETE CASCADE,
    agent_id VARCHAR(255) NOT NULL,
    system_id VARCHAR(255),
    inference_time_ms BIGINT,
    gpu_power_rating_watts DECIMAL(10, 2),
    cpu_power_rating_watts DECIMAL(10, 2),
    energy_estimate_kwh DECIMAL(10, 6) NOT NULL,
    carbon_grams DECIMAL(10, 4), -- CO2 equivalent
    carbon_intensity_g_per_kwh DECIMAL(10, 4) DEFAULT 475.0, -- EU average grid intensity
    model_name VARCHAR(255),
    model_version VARCHAR(100),
    hardware_type VARCHAR(100), -- "GPU", "CPU", "TPU", "EDGE"
    region VARCHAR(50), -- For region-specific carbon intensity
    measured_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- AI System Inventory (for AI-BOM export)
CREATE TABLE IF NOT EXISTS ai_system_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    system_id VARCHAR(255) NOT NULL UNIQUE,
    system_name VARCHAR(255) NOT NULL,
    system_version VARCHAR(100),
    system_type VARCHAR(100) NOT NULL, -- "MODEL", "FRAMEWORK", "DATASET", "SERVICE"
    description TEXT,
    vendor VARCHAR(255),
    license VARCHAR(100),
    source_url TEXT,
    checksum_sha256 VARCHAR(64),
    dependencies JSONB, -- Array of dependency system_ids
    training_data_info JSONB, -- Information about training data
    compliance_status VARCHAR(50), -- "COMPLIANT", "REVIEW_REQUIRED", "NON_COMPLIANT"
    risk_level VARCHAR(20),
    dpia_id VARCHAR(255) REFERENCES dpia_records(dpia_id) ON DELETE SET NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_ai_energy_telemetry_seal_id ON ai_energy_telemetry(seal_id);
CREATE INDEX IF NOT EXISTS idx_ai_energy_telemetry_system_id ON ai_energy_telemetry(system_id);
CREATE INDEX IF NOT EXISTS idx_ai_energy_telemetry_measured_at ON ai_energy_telemetry(measured_at);
CREATE INDEX IF NOT EXISTS idx_ai_system_inventory_system_id ON ai_system_inventory(system_id);
CREATE INDEX IF NOT EXISTS idx_ai_system_inventory_system_type ON ai_system_inventory(system_type);

-- Trigger for updated_at
CREATE TRIGGER update_ai_system_inventory_updated_at BEFORE UPDATE ON ai_system_inventory
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate energy from inference time and power rating
CREATE OR REPLACE FUNCTION calculate_energy_kwh(
    inference_time_ms BIGINT,
    gpu_power_watts DECIMAL DEFAULT NULL,
    cpu_power_watts DECIMAL DEFAULT NULL
) RETURNS DECIMAL AS $$
DECLARE
    total_power_watts DECIMAL;
    time_hours DECIMAL;
BEGIN
    -- Default power ratings if not provided (typical values)
    total_power_watts := COALESCE(gpu_power_watts, 0) + COALESCE(cpu_power_watts, 0);
    
    -- If no power provided, use default GPU estimate (250W for typical inference)
    IF total_power_watts = 0 THEN
        total_power_watts := 250.0;
    END IF;
    
    -- Convert milliseconds to hours
    time_hours := inference_time_ms::DECIMAL / 1000.0 / 3600.0;
    
    -- Calculate energy: power (W) * time (h) / 1000 = kWh
    RETURN (total_power_watts * time_hours) / 1000.0;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to calculate carbon footprint
CREATE OR REPLACE FUNCTION calculate_carbon_grams(
    energy_kwh DECIMAL,
    carbon_intensity_g_per_kwh DECIMAL DEFAULT 475.0
) RETURNS DECIMAL AS $$
BEGIN
    -- Default: EU average grid carbon intensity (475 g CO2/kWh)
    RETURN energy_kwh * carbon_intensity_g_per_kwh;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

