use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Database model for modules
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModuleDb {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub enabled_by_default: bool,
    pub requires_license: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for module activations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct ModuleActivationDb {
    pub id: Uuid,
    pub module_id: Uuid,
    pub enabled: bool,
    pub activated_at: DateTime<Utc>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub activated_by: Option<Uuid>,
    pub notes: Option<String>,
}

/// Database model for feature flags
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct FeatureFlagDb {
    pub id: Uuid,
    pub name: String,
    pub module_id: Option<Uuid>,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for compliance records
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceRecordDb {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub action_summary: String,
    pub seal_id: String,
    pub status: String,
    pub user_notified: Option<bool>,
    pub notification_timestamp: Option<DateTime<Utc>>,
    pub human_oversight_status: Option<String>,
    pub risk_level: Option<String>,
    pub user_id: Option<String>,
    pub tx_id: String,
    pub payload_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for risk assessments
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskAssessmentDb {
    pub id: Uuid,
    pub seal_id: String,
    pub risk_level: String,
    pub risk_factors: serde_json::Value,
    pub mitigation_actions: serde_json::Value,
    pub assessed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Database model for human oversight
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct HumanOversightDb {
    pub id: Uuid,
    pub seal_id: String,
    pub status: String,
    pub reviewer_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub comments: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database model for data breaches
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataBreachDb {
    pub id: Uuid,
    pub breach_id: String,
    pub description: String,
    pub breach_type: String,
    pub affected_users: serde_json::Value,
    pub detected_at: DateTime<Utc>,
    pub affected_records_count: Option<i32>,
    pub status: String,
    pub authority_notified_at: Option<DateTime<Utc>>,
    pub users_notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Database model for user data index
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct UserDataIndexDb {
    pub id: Uuid,
    pub user_id: String,
    pub seal_id: String,
    pub created_at: DateTime<Utc>,
}

/// Database model for encrypted log keys
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct EncryptedLogKeyDb {
    pub id: Uuid,
    pub log_id: String,
    pub wrapped_dek: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub shredded_at: Option<DateTime<Utc>>,
}

/// Database model for consent records
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConsentRecordDb {
    pub id: uuid::Uuid,
    pub user_id: String,
    pub consent_type: String,
    pub purpose: String,
    pub legal_basis: String,
    pub granted: bool,
    pub granted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub withdrawn_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub consent_method: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub consent_text: Option<String>,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for processing activities
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct ProcessingActivityDb {
    pub id: uuid::Uuid,
    pub activity_name: String,
    pub purpose: String,
    pub legal_basis: String,
    pub data_categories: Vec<String>,
    pub data_subject_categories: Vec<String>,
    pub recipients: Vec<String>,
    pub third_country_transfers: bool,
    pub third_countries: Vec<String>,
    pub retention_period_days: Option<i32>,
    pub security_measures: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for DPIA records
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DpiaRecordDb {
    pub id: uuid::Uuid,
    pub dpia_id: String,
    pub activity_name: String,
    pub description: String,
    pub legal_basis: String,
    pub data_categories: Vec<String>,
    pub data_subject_categories: Vec<String>,
    pub processing_purposes: Vec<String>,
    pub risk_level: String,
    pub identified_risks: serde_json::Value,
    pub mitigation_measures: serde_json::Value,
    pub residual_risks: serde_json::Value,
    pub consultation_required: bool,
    pub supervisory_authority_consulted: bool,
    pub consultation_date: Option<chrono::DateTime<chrono::Utc>>,
    pub consultation_response: Option<String>,
    pub status: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approval_date: Option<chrono::DateTime<chrono::Utc>>,
    pub next_review_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for retention policies
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RetentionPolicyDb {
    pub id: uuid::Uuid,
    pub policy_name: String,
    pub data_category: String,
    pub retention_period_days: i32,
    pub legal_basis: String,
    pub description: Option<String>,
    pub auto_delete: bool,
    pub notification_days_before: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for retention assignments
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RetentionAssignmentDb {
    pub id: uuid::Uuid,
    pub record_type: String,
    pub record_id: String,
    pub policy_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deletion_status: String,
    pub exemption_reason: Option<String>,
    pub last_notification_sent: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<String>,
}

/// Database model for monitoring events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MonitoringEventDb {
    pub id: uuid::Uuid,
    pub event_id: String,
    pub event_type: String,
    pub severity: String,
    pub system_id: String,
    pub system_version: Option<String>,
    pub description: String,
    pub affected_users: serde_json::Value,
    pub affected_records_count: Option<i32>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolution_status: String,
    pub resolution_notes: Option<String>,
    pub reported_to_authority: bool,
    pub authority_report_date: Option<chrono::DateTime<chrono::Utc>>,
    pub corrective_action_taken: Option<String>,
    pub preventive_measures: Option<String>,
    pub created_by: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for system health status
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemHealthStatusDb {
    pub id: uuid::Uuid,
    pub system_id: String,
    pub system_version: Option<String>,
    pub overall_status: String,
    pub compliance_status: String,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub active_incidents_count: i32,
    pub critical_incidents_count: i32,
    pub last_incident_at: Option<chrono::DateTime<chrono::Utc>>,
    pub performance_score: Option<f64>,
    pub compliance_score: Option<f64>,
    pub notes: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for AI energy telemetry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct AiEnergyTelemetryDb {
    pub id: uuid::Uuid,
    pub seal_id: String,
    pub agent_id: String,
    pub system_id: Option<String>,
    pub inference_time_ms: Option<i64>,
    pub gpu_power_rating_watts: Option<f64>,
    pub cpu_power_rating_watts: Option<f64>,
    pub energy_estimate_kwh: f64,
    pub carbon_grams: Option<f64>,
    pub carbon_intensity_g_per_kwh: Option<f64>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
    pub hardware_type: Option<String>,
    pub region: Option<String>,
    pub measured_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for webhook endpoints
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookEndpointDb {
    pub id: uuid::Uuid,
    pub endpoint_url: String,
    pub secret_key: String,
    pub event_types: Vec<String>,
    pub active: bool,
    pub retry_count: i32,
    pub timeout_seconds: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Database model for webhook deliveries
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDeliveryDb {
    pub id: uuid::Uuid,
    pub webhook_endpoint_id: uuid::Uuid,
    pub event_type: String,
    pub event_payload: serde_json::Value,
    pub status: String,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: i32,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Helper to convert ComplianceRecordDb to ComplianceRecord (for Annex IV)
impl From<ComplianceRecordDb> for crate::core::annex_iv::ComplianceRecord {
    fn from(db: ComplianceRecordDb) -> Self {
        Self {
            timestamp: db.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            action_summary: db.action_summary,
            seal_id: db.seal_id,
            status: db.status,
            user_notified: db.user_notified,
            notification_timestamp: db.notification_timestamp.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            human_oversight_status: db.human_oversight_status,
            risk_level: db.risk_level,
            user_id: db.user_id,
            // Extended Annex IV fields (defaults, can be populated from joins)
            lifecycle_stage: None,
            training_data_sources: None,
            performance_metrics: None,
            post_market_monitoring: None,
            human_oversight_procedures: None,
            risk_management_measures: None,
        }
    }
}

