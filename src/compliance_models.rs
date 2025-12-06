use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Risk Assessment for EU AI Act Article 9
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct RiskAssessment {
    /// Risk level: LOW, MEDIUM, HIGH
    #[schema(example = "MEDIUM")]
    pub risk_level: String,
    /// List of identified risk factors
    #[schema(example = r#"["Data sovereignty", "Bias potential"]"#)]
    pub risk_factors: Vec<String>,
    /// Mitigation actions taken
    #[schema(example = r#"["Sovereign lock enabled", "Human oversight required"]"#)]
    pub mitigation_actions: Vec<String>,
    /// Timestamp of assessment
    #[schema(example = "2024-01-15 14:30:00")]
    pub assessed_at: String,
}

/// Human Oversight Request (EU AI Act Article 14)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HumanOversightRequest {
    /// Seal ID of the action requiring oversight
    #[schema(example = "SEAL-2024-01-15-ABC123")]
    pub seal_id: String,
    /// Reason for requiring human oversight
    #[schema(example = "High-risk financial decision")]
    pub reason: Option<String>,
}

/// Human Oversight Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HumanOversightResponse {
    /// Approval status: APPROVED, REJECTED, PENDING
    #[schema(example = "APPROVED")]
    pub status: String,
    /// Human reviewer ID
    #[schema(example = "reviewer-001")]
    pub reviewer_id: Option<String>,
    /// Timestamp of decision
    #[schema(example = "2024-01-15 14:35:00")]
    pub decided_at: String,
    /// Comments from reviewer
    #[schema(example = "Approved after risk review")]
    pub comments: Option<String>,
}

/// Data Subject Access Request (GDPR Article 15)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataSubjectAccessRequest {
    /// User ID requesting access
    #[schema(example = "user-123")]
    pub user_id: String,
}

/// Data Subject Access Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataSubjectAccessResponse {
    /// All records associated with the user
    pub records: Vec<DataSubjectRecord>,
    /// Export format: JSON
    #[schema(example = "json")]
    pub format: String,
    /// Timestamp of export
    #[schema(example = "2024-01-15 14:30:00")]
    pub exported_at: String,
}

/// Individual record in data subject export
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataSubjectRecord {
    pub timestamp: String,
    pub action_summary: String,
    pub seal_id: String,
    pub status: String,
    pub risk_level: Option<String>,
}

/// Data Subject Rectification Request (GDPR Article 16)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataSubjectRectificationRequest {
    /// User ID
    #[schema(example = "user-123")]
    pub user_id: String,
    /// Seal ID of record to rectify
    #[schema(example = "SEAL-2024-01-15-ABC123")]
    pub seal_id: String,
    /// Corrected data
    #[schema(example = "Corrected action description")]
    pub corrected_data: String,
}

/// Data Breach Report (GDPR Article 33-34)
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct DataBreachReport {
    /// Description of the breach
    #[schema(example = "Unauthorized access detected")]
    pub description: String,
    /// Type of breach
    #[schema(example = "UNAUTHORIZED_ACCESS")]
    pub breach_type: String, // "UNAUTHORIZED_ACCESS", "DATA_LEAK", "SYSTEM_COMPROMISE"
    /// Affected user IDs
    #[schema(example = r#"["user-123", "user-456"]"#)]
    pub affected_users: Vec<String>,
    /// Timestamp when breach was detected
    #[schema(example = "2024-01-15 14:30:00")]
    pub detected_at: String,
    /// Estimated number of affected records
    #[schema(example = 42)]
    pub affected_records_count: Option<u32>,
}

/// Data Breach Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataBreachResponse {
    /// Breach ID for tracking
    #[schema(example = "BREACH-2024-01-15-001")]
    pub breach_id: String,
    /// Notification status
    #[schema(example = "REPORTED")]
    pub status: String, // "REPORTED", "NOTIFIED_AUTHORITY", "NOTIFIED_USERS"
    /// Timestamp when reported to authority
    #[schema(example = "2024-01-15 14:35:00")]
    pub authority_notified_at: Option<String>,
    /// Timestamp when users were notified
    #[schema(example = "2024-01-15 14:40:00")]
    pub users_notified_at: Option<String>,
}

// ========== PRIORITY 2: CONSENT MANAGEMENT (GDPR Articles 6, 7) ==========

/// Consent Request (GDPR Article 7)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ConsentRequest {
    /// User ID
    #[schema(example = "user-123")]
    pub user_id: String,
    /// Type of consent: PROCESSING, STORAGE, TRANSFER, MARKETING
    #[schema(example = "PROCESSING")]
    pub consent_type: String,
    /// Purpose of processing
    #[schema(example = "AI model training and inference")]
    pub purpose: String,
    /// Legal basis: CONSENT, CONTRACT, LEGAL_OBLIGATION, VITAL_INTERESTS, PUBLIC_TASK, LEGITIMATE_INTERESTS
    #[schema(example = "CONSENT")]
    pub legal_basis: String,
    /// Consent method: EXPLICIT, IMPLICIT, OPT_IN, OPT_OUT
    #[schema(example = "EXPLICIT")]
    pub consent_method: Option<String>,
    /// Consent text shown to user
    #[schema(example = "I consent to processing of my data for AI purposes")]
    pub consent_text: Option<String>,
    /// Expiration date (optional)
    #[schema(example = "2025-12-31 23:59:59")]
    pub expires_at: Option<String>,
    /// IP address (for audit)
    #[schema(example = "192.168.1.1")]
    pub ip_address: Option<String>,
    /// User agent (for audit)
    #[schema(example = "Mozilla/5.0...")]
    pub user_agent: Option<String>,
}

/// Consent Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ConsentResponse {
    /// Consent record ID
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub consent_id: String,
    /// User ID
    #[schema(example = "user-123")]
    pub user_id: String,
    /// Consent type
    #[schema(example = "PROCESSING")]
    pub consent_type: String,
    /// Whether consent is granted
    #[schema(example = true)]
    pub granted: bool,
    /// When consent was granted
    #[schema(example = "2024-01-15 14:30:00")]
    pub granted_at: Option<String>,
    /// When consent expires
    #[schema(example = "2025-12-31 23:59:59")]
    pub expires_at: Option<String>,
    /// Version of consent text
    #[schema(example = 1)]
    pub version: i32,
}

/// Withdraw Consent Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WithdrawConsentRequest {
    /// User ID
    #[schema(example = "user-123")]
    pub user_id: String,
    /// Consent type to withdraw
    #[schema(example = "PROCESSING")]
    pub consent_type: Option<String>, // If None, withdraws all consents
    /// Reason for withdrawal (optional)
    #[schema(example = "No longer needed")]
    pub reason: Option<String>,
}

/// Get User Consents Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserConsentsResponse {
    /// User ID
    #[schema(example = "user-123")]
    pub user_id: String,
    /// List of all consents
    pub consents: Vec<ConsentResponse>,
}

// ========== PRIORITY 2: DPIA TRACKING (GDPR Article 35) ==========

/// DPIA Request (Data Protection Impact Assessment)
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DpiaRequest {
    /// Activity name
    #[schema(example = "AI Credit Scoring System")]
    pub activity_name: String,
    /// Description of processing activity
    #[schema(example = "Automated credit scoring using machine learning")]
    pub description: String,
    /// Legal basis for processing
    #[schema(example = "LEGITIMATE_INTERESTS")]
    pub legal_basis: String,
    /// Categories of personal data processed
    #[schema(example = r#"["Financial data", "Credit history", "Employment data"]"#)]
    pub data_categories: Vec<String>,
    /// Categories of data subjects
    #[schema(example = r#"["Loan applicants", "Existing customers"]"#)]
    pub data_subject_categories: Vec<String>,
    /// Purposes of processing
    #[schema(example = r#"["Credit assessment", "Risk evaluation"]"#)]
    pub processing_purposes: Vec<String>,
    /// Initial risk level assessment
    #[schema(example = "HIGH")]
    pub risk_level: String,
    /// Identified risks
    #[schema(example = r#"["Discrimination risk", "Data breach risk", "Incorrect scoring"]"#)]
    pub identified_risks: Vec<String>,
    /// Proposed mitigation measures
    #[schema(example = r#"["Bias testing", "Encryption", "Human oversight"]"#)]
    pub mitigation_measures: Vec<String>,
    /// Created by (user ID or system)
    #[schema(example = "dpia-manager-001")]
    pub created_by: String,
}

/// DPIA Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DpiaResponse {
    /// DPIA ID
    #[schema(example = "DPIA-2024-001")]
    pub dpia_id: String,
    /// Activity name
    pub activity_name: String,
    /// Status
    #[schema(example = "DRAFT")]
    pub status: String,
    /// Risk level
    #[schema(example = "HIGH")]
    pub risk_level: String,
    /// Whether consultation is required
    #[schema(example = true)]
    pub consultation_required: bool,
    /// Created at
    #[schema(example = "2024-01-15 14:30:00")]
    pub created_at: String,
}

/// Update DPIA Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateDpiaRequest {
    /// Status update
    #[schema(example = "APPROVED")]
    pub status: Option<String>,
    /// Risk level update
    #[schema(example = "MEDIUM")]
    pub risk_level: Option<String>,
    /// Residual risks after mitigation
    #[schema(example = r#"["Low residual discrimination risk"]"#)]
    pub residual_risks: Option<Vec<String>>,
    /// Review comments
    #[schema(example = "DPIA approved after mitigation measures implemented")]
    pub review_comments: Option<String>,
    /// Reviewed by
    #[schema(example = "reviewer-001")]
    pub reviewed_by: Option<String>,
    /// Next review date
    #[schema(example = "2025-01-15 14:30:00")]
    pub next_review_date: Option<String>,
}

/// Get All DPIAs Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DpiasResponse {
    /// List of all DPIAs
    pub dpias: Vec<DpiaResponse>,
}

// ========== PRIORITY 2: RETENTION PERIOD AUTOMATION (GDPR Article 5(1)(e)) ==========

/// Retention Policy Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RetentionPolicyRequest {
    /// Policy name
    #[schema(example = "GDPR_PERSONAL_DATA")]
    pub policy_name: String,
    /// Data category
    #[schema(example = "PERSONAL_DATA")]
    pub data_category: String,
    /// Retention period in days
    #[schema(example = 1095)]
    pub retention_period_days: i32,
    /// Legal basis for retention
    #[schema(example = "GDPR Article 5(1)(e) - Storage Limitation")]
    pub legal_basis: String,
    /// Description
    #[schema(example = "Personal data retention policy")]
    pub description: Option<String>,
    /// Whether to automatically delete after expiration
    #[schema(example = true)]
    pub auto_delete: Option<bool>,
    /// Days before expiration to send notification
    #[schema(example = 30)]
    pub notification_days_before: Option<i32>,
}

/// Retention Policy Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RetentionPolicyResponse {
    /// Policy ID
    pub policy_id: String,
    /// Policy name
    pub policy_name: String,
    /// Data category
    pub data_category: String,
    /// Retention period in days
    pub retention_period_days: i32,
    /// Auto delete enabled
    pub auto_delete: bool,
    /// Created at
    pub created_at: String,
}

/// Assign Retention Policy Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct AssignRetentionRequest {
    /// Record type: COMPLIANCE_RECORD, CONSENT_RECORD, DPIA, etc.
    #[schema(example = "COMPLIANCE_RECORD")]
    pub record_type: String,
    /// Record ID (seal_id, consent_id, dpia_id, etc.)
    #[schema(example = "SEAL-2024-01-15-ABC123")]
    pub record_id: String,
    /// Policy name to assign
    #[schema(example = "GDPR_COMPLIANCE_RECORDS")]
    pub policy_name: String,
}

/// Retention Status Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RetentionStatusResponse {
    /// Record type
    pub record_type: String,
    /// Record ID
    pub record_id: String,
    /// Policy name
    pub policy_name: String,
    /// Expires at
    pub expires_at: String,
    /// Days until expiration
    pub days_until_expiration: i64,
    /// Deletion status
    pub deletion_status: String,
}

/// Get All Retention Policies Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RetentionPoliciesResponse {
    /// List of all retention policies
    pub policies: Vec<RetentionPolicyResponse>,
}

/// Retention Tracking Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct RetentionTrackingResponse {
    /// Record type
    pub record_type: String,
    /// Record ID
    pub record_id: String,
    /// Created at
    pub created_at: String,
    /// Expires at
    pub expires_at: String,
    /// Deletion status
    pub deletion_status: String,
    /// Days until expiration
    pub days_until_expiration: Option<i64>,
}

/// Get Expiring Records Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ExpiringRecordsResponse {
    /// Records expiring soon
    pub expiring_records: Vec<RetentionTrackingResponse>,
    /// Total count
    pub total_count: i64,
}

// ========== PRIORITY 2: POST-MARKET MONITORING (EU AI Act Article 72) ==========

/// Monitoring Event Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct MonitoringEventRequest {
    /// Event type
    #[schema(example = "INCIDENT")]
    pub event_type: String,
    /// Severity level
    #[schema(example = "HIGH")]
    pub severity: String,
    /// System ID
    #[schema(example = "AI-SYSTEM-001")]
    pub system_id: String,
    /// System version
    #[schema(example = "v2.1.0")]
    pub system_version: Option<String>,
    /// Description
    #[schema(example = "Unexpected bias detected in credit scoring model")]
    pub description: String,
    /// Affected user IDs
    #[schema(example = r#"["user-123", "user-456"]"#)]
    pub affected_users: Option<Vec<String>>,
    /// Detected at timestamp
    #[schema(example = "2024-01-15 14:30:00")]
    pub detected_at: Option<String>,
}

/// Monitoring Event Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct MonitoringEventResponse {
    /// Event ID
    pub event_id: String,
    /// Event type
    pub event_type: String,
    /// Severity
    pub severity: String,
    /// System ID
    pub system_id: String,
    /// Resolution status
    pub resolution_status: String,
    /// Detected at
    pub detected_at: String,
}

/// Monitoring Metric Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct MonitoringMetricRequest {
    /// System ID
    #[schema(example = "AI-SYSTEM-001")]
    pub system_id: String,
    /// Metric name
    #[schema(example = "ACCURACY")]
    pub metric_name: String,
    /// Metric value
    #[schema(example = 95.5)]
    pub metric_value: f64,
    /// Metric unit
    #[schema(example = "PERCENTAGE")]
    pub metric_unit: Option<String>,
    /// Threshold min
    #[schema(example = 90.0)]
    pub threshold_min: Option<f64>,
    /// Threshold max
    #[schema(example = 100.0)]
    pub threshold_max: Option<f64>,
    /// System version
    #[schema(example = "v2.1.0")]
    pub system_version: Option<String>,
}

/// System Health Status Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SystemHealthStatusResponse {
    /// System ID
    pub system_id: String,
    /// Overall status
    pub overall_status: String,
    /// Compliance status
    pub compliance_status: String,
    /// Active incidents count
    pub active_incidents_count: i32,
    /// Critical incidents count
    pub critical_incidents_count: i32,
    /// Performance score
    pub performance_score: Option<f64>,
    /// Compliance score
    pub compliance_score: Option<f64>,
    /// Last health check
    pub last_health_check: String,
}

/// Get All Monitoring Events Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct MonitoringEventsResponse {
    /// List of monitoring events
    pub events: Vec<MonitoringEventResponse>,
    /// Total count
    pub total_count: i64,
}

/// Update Event Resolution Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateEventResolutionRequest {
    /// Resolution status
    #[schema(example = "RESOLVED")]
    pub resolution_status: String,
    /// Resolution notes
    #[schema(example = "Issue fixed by updating model weights")]
    pub resolution_notes: Option<String>,
    /// Corrective action taken
    #[schema(example = "Model retrained with updated dataset")]
    pub corrective_action_taken: Option<String>,
    /// Preventive measures
    #[schema(example = "Added bias detection monitoring")]
    pub preventive_measures: Option<String>,
}

// ========== AI-BOM (CycloneDX) EXPORT ==========

/// AI System Inventory Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct AiSystemInventoryRequest {
    /// System ID
    #[schema(example = "AI-SYSTEM-001")]
    pub system_id: String,
    /// System name
    #[schema(example = "Credit Scoring Model")]
    pub system_name: String,
    /// System version
    #[schema(example = "v2.1.0")]
    pub system_version: Option<String>,
    /// System type: MODEL, FRAMEWORK, DATASET, SERVICE
    #[schema(example = "MODEL")]
    pub system_type: String,
    /// Description
    #[schema(example = "Machine learning model for credit risk assessment")]
    pub description: Option<String>,
    /// Vendor
    #[schema(example = "Veridion")]
    pub vendor: Option<String>,
    /// License
    #[schema(example = "MIT")]
    pub license: Option<String>,
    /// Source URL
    #[schema(example = "https://github.com/veridion/model")]
    pub source_url: Option<String>,
    /// SHA256 checksum
    #[schema(example = "abc123...")]
    pub checksum_sha256: Option<String>,
    /// Dependencies (array of system_ids)
    #[schema(example = r#"["AI-FRAMEWORK-001", "AI-DATASET-001"]"#)]
    pub dependencies: Option<Vec<String>>,
    /// Training data information
    #[schema(example = r#"{"source": "Internal", "size": "10GB", "anonymized": true}"#)]
    pub training_data_info: Option<serde_json::Value>,
    /// Risk level
    #[schema(example = "MEDIUM")]
    pub risk_level: Option<String>,
    /// DPIA ID if applicable
    #[schema(example = "DPIA-2024-001")]
    pub dpia_id: Option<String>,
}

/// CycloneDX AI-BOM Component
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CycloneDxComponent {
    #[serde(rename = "type")]
    pub component_type: String, // "application", "library", "container", etc.
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub purl: Option<String>, // Package URL
    pub properties: Option<Vec<CycloneDxProperty>>,
    pub bom_ref: Option<String>,
}

/// CycloneDX Property
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CycloneDxProperty {
    pub name: String,
    pub value: String,
}

/// CycloneDX AI-BOM Export
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CycloneDxBom {
    #[serde(rename = "bomFormat")]
    pub bom_format: String, // "CycloneDX"
    #[serde(rename = "specVersion")]
    pub spec_version: String, // "1.5"
    pub version: i32,
    pub metadata: CycloneDxMetadata,
    pub components: Vec<CycloneDxComponent>,
}

/// CycloneDX Metadata
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CycloneDxMetadata {
    pub timestamp: String,
    pub tools: Option<Vec<CycloneDxTool>>,
    pub properties: Option<Vec<CycloneDxProperty>>,
}

/// CycloneDX Tool
#[derive(Serialize, Deserialize, ToSchema)]
pub struct CycloneDxTool {
    pub name: String,
    pub version: String,
    pub vendor: Option<String>,
}

// ========== WEBHOOK SUPPORT ==========

/// Webhook Endpoint Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WebhookEndpointRequest {
    /// Webhook URL endpoint
    #[schema(example = "https://example.com/webhooks/compliance")]
    pub endpoint_url: String,
    /// Event types to subscribe to
    #[schema(example = r#"["compliance.action", "data_breach.detected", "human_oversight.required"]"#)]
    pub event_types: Vec<String>,
    /// Secret key for HMAC signing (optional, auto-generated if not provided)
    #[schema(example = "optional_custom_secret_key")]
    pub secret_key: Option<String>,
    /// Retry count (default: 3)
    #[schema(example = 3)]
    pub retry_count: Option<i32>,
    /// Timeout in seconds (default: 30)
    #[schema(example = 30)]
    pub timeout_seconds: Option<i32>,
}

/// Webhook Endpoint Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WebhookEndpointResponse {
    /// Webhook endpoint ID
    pub id: String,
    /// Webhook URL endpoint
    pub endpoint_url: String,
    /// Event types subscribed to
    pub event_types: Vec<String>,
    /// Whether webhook is active
    pub active: bool,
    /// Retry count
    pub retry_count: i32,
    /// Timeout in seconds
    pub timeout_seconds: i32,
    /// Created timestamp
    pub created_at: String,
}

/// Webhook Endpoints List Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WebhookEndpointsResponse {
    /// List of webhook endpoints
    pub endpoints: Vec<WebhookEndpointResponse>,
    /// Total count
    pub total_count: i64,
}

/// Update Webhook Endpoint Request
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateWebhookEndpointRequest {
    /// Webhook URL endpoint (optional)
    pub endpoint_url: Option<String>,
    /// Event types to subscribe to (optional)
    pub event_types: Option<Vec<String>>,
    /// Whether webhook is active (optional)
    pub active: Option<bool>,
    /// Retry count (optional)
    pub retry_count: Option<i32>,
    /// Timeout in seconds (optional)
    pub timeout_seconds: Option<i32>,
}

/// Webhook Delivery Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WebhookDeliveryResponse {
    /// Delivery ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Status
    pub status: String,
    /// Response code
    pub response_code: Option<i32>,
    /// Attempts
    pub attempts: i32,
    /// Created timestamp
    pub created_at: String,
    /// Delivered timestamp
    pub delivered_at: Option<String>,
}

/// Webhook Deliveries List Response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WebhookDeliveriesResponse {
    /// List of webhook deliveries
    pub deliveries: Vec<WebhookDeliveryResponse>,
    /// Total count
    pub total_count: i64,
}

/// Webhook Event (internal structure for delivery)
#[derive(Serialize, Deserialize, Clone)]
pub struct WebhookEvent {
    /// Event type
    pub event_type: String,
    /// Timestamp
    pub timestamp: String,
    /// Event data payload
    pub data: serde_json::Value,
    /// HMAC signature (added by webhook service)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

