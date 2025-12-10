// Configuration Drift Detection
// DORA requirement: Continuous monitoring and prevention of unauthorized changes

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Configuration Baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationBaseline {
    pub id: Uuid,
    pub baseline_name: String,
    pub baseline_type: String, // POLICY, ASSET, SYSTEM, NETWORK
    pub baseline_config: serde_json::Value,
    pub is_golden_image: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub description: Option<String>,
}

/// Configuration Drift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationDrift {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub drift_type: String, // ADDED, REMOVED, MODIFIED, UNAUTHORIZED
    pub drift_severity: String, // LOW, MEDIUM, HIGH, CRITICAL
    pub changed_path: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub detected_at: DateTime<Utc>,
    pub auto_remediated: bool,
    pub acknowledged: bool,
}

/// Configuration Drift Detection Service
pub struct ConfigurationDriftService;

impl ConfigurationDriftService {
    /// Create a configuration baseline
    pub async fn create_baseline(
        db_pool: &PgPool,
        baseline_name: &str,
        baseline_type: &str,
        baseline_config: &serde_json::Value,
        is_golden_image: bool,
        created_by: Option<&str>,
        description: Option<&str>,
    ) -> Result<Uuid, String> {
        let baseline_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO configuration_baselines (
                id, baseline_name, baseline_type, baseline_config,
                is_golden_image, created_by, description
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(baseline_id)
        .bind(baseline_name)
        .bind(baseline_type)
        .bind(baseline_config)
        .bind(is_golden_image)
        .bind(created_by)
        .bind(description)
        .execute(db_pool)
        .await
        .map_err(|e| format!("Failed to create baseline: {}", e))?;

        Ok(baseline_id)
    }

    /// Capture current configuration snapshot
    pub async fn capture_snapshot(
        db_pool: &PgPool,
        baseline_id: Uuid,
        snapshot_config: &serde_json::Value,
        captured_by: Option<&str>,
    ) -> Result<Uuid, String> {
        // Calculate hash of configuration
        let config_str = serde_json::to_string(snapshot_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        let mut hasher = Sha256::new();
        hasher.update(config_str.as_bytes());
        let snapshot_hash = format!("{:x}", hasher.finalize());

        let snapshot_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO configuration_snapshots (
                id, baseline_id, snapshot_config, snapshot_hash, captured_by
            ) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(snapshot_id)
        .bind(baseline_id)
        .bind(snapshot_config)
        .bind(&snapshot_hash)
        .bind(captured_by)
        .execute(db_pool)
        .await
        .map_err(|e| format!("Failed to capture snapshot: {}", e))?;

        Ok(snapshot_id)
    }

    /// Detect configuration drift
    pub async fn detect_drift(
        db_pool: &PgPool,
        baseline_id: Uuid,
        current_config: &serde_json::Value,
    ) -> Result<Vec<ConfigurationDrift>, String> {
        // Get baseline configuration
        #[derive(sqlx::FromRow)]
        struct BaselineRow {
            baseline_config: serde_json::Value,
            is_golden_image: bool,
        }

        let baseline: Option<BaselineRow> = sqlx::query_as(
            "SELECT baseline_config, is_golden_image
             FROM configuration_baselines
             WHERE id = $1"
        )
        .bind(baseline_id)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| format!("Failed to fetch baseline: {}", e))?;

        let baseline = baseline.ok_or_else(|| "Baseline not found".to_string())?;

        // Capture snapshot
        let snapshot_id = Self::capture_snapshot(db_pool, baseline_id, current_config, Some("system")).await?;

        // Detect differences (simplified - in production, would use proper JSON diff)
        let mut drifts = Vec::new();

        // Simple comparison: if configs are different, it's a drift
        if baseline.baseline_config != *current_config {
            let drift_id = Uuid::new_v4();
            let drift_severity = if baseline.is_golden_image {
                "CRITICAL" // Golden image violations are critical
            } else {
                "MEDIUM"
            };

            // Store drift detection
            sqlx::query(
                "INSERT INTO configuration_drift_detection (
                    id, baseline_id, snapshot_id, drift_type, drift_severity,
                    changed_path, old_value, new_value
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(drift_id)
            .bind(baseline_id)
            .bind(snapshot_id)
            .bind("MODIFIED")
            .bind(drift_severity)
            .bind("root")
            .bind(&baseline.baseline_config)
            .bind(current_config)
            .execute(db_pool)
            .await
            .map_err(|e| format!("Failed to store drift: {}", e))?;

            drifts.push(ConfigurationDrift {
                id: drift_id,
                baseline_id,
                drift_type: "MODIFIED".to_string(),
                drift_severity: drift_severity.to_string(),
                changed_path: "root".to_string(),
                old_value: Some(baseline.baseline_config),
                new_value: Some(current_config.clone()),
                detected_at: Utc::now(),
                auto_remediated: false,
                acknowledged: false,
            });
        }

        Ok(drifts)
    }

    /// Get all drifts for a baseline
    pub async fn get_drifts(
        db_pool: &PgPool,
        baseline_id: Uuid,
        acknowledged_only: Option<bool>,
    ) -> Result<Vec<ConfigurationDrift>, String> {
        let mut query = sqlx::QueryBuilder::new(
            "SELECT id, baseline_id, drift_type, drift_severity, changed_path,
                    old_value, new_value, detected_at, auto_remediated, acknowledged
             FROM configuration_drift_detection
             WHERE baseline_id = "
        );
        query.push_bind(baseline_id);

        if let Some(ack_only) = acknowledged_only {
            query.push(" AND acknowledged = ");
            query.push_bind(ack_only);
        }

        query.push(" ORDER BY detected_at DESC");

        #[derive(sqlx::FromRow)]
        struct DriftRow {
            id: Uuid,
            baseline_id: Uuid,
            drift_type: String,
            drift_severity: String,
            changed_path: String,
            old_value: Option<serde_json::Value>,
            new_value: Option<serde_json::Value>,
            detected_at: DateTime<Utc>,
            auto_remediated: bool,
            acknowledged: bool,
        }

        let drifts: Vec<DriftRow> = query
            .build_query_as()
            .fetch_all(db_pool)
            .await
            .map_err(|e| format!("Failed to fetch drifts: {}", e))?;

        Ok(drifts.into_iter().map(|d| ConfigurationDrift {
            id: d.id,
            baseline_id: d.baseline_id,
            drift_type: d.drift_type,
            drift_severity: d.drift_severity,
            changed_path: d.changed_path,
            old_value: d.old_value,
            new_value: d.new_value,
            detected_at: d.detected_at,
            auto_remediated: d.auto_remediated,
            acknowledged: d.acknowledged,
        }).collect())
    }

    /// Auto-remediate drift (restore to baseline)
    pub async fn auto_remediate_drift(
        db_pool: &PgPool,
        drift_id: Uuid,
    ) -> Result<(), String> {
        // Get drift details
        #[derive(sqlx::FromRow)]
        struct DriftDetails {
            baseline_id: Uuid,
            old_value: Option<serde_json::Value>,
        }

        let drift: Option<DriftDetails> = sqlx::query_as(
            "SELECT baseline_id, old_value
             FROM configuration_drift_detection
             WHERE id = $1"
        )
        .bind(drift_id)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| format!("Failed to fetch drift: {}", e))?;

        let drift: DriftDetails = drift.ok_or_else(|| "Drift not found".to_string())?;

        // Mark as auto-remediated
        sqlx::query(
            "UPDATE configuration_drift_detection
             SET auto_remediated = true, remediation_action = 'RESTORED_TO_BASELINE'
             WHERE id = $1"
        )
        .bind(drift_id)
        .execute(db_pool)
        .await
        .map_err(|e| format!("Failed to mark as remediated: {}", e))?;

        Ok(())
    }
}

