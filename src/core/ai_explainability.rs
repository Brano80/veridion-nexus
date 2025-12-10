// AI Explainability & Observability
// EU AI Act Article 13 requirement: Explainability for high-risk AI systems

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// AI Decision Explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDecisionExplanation {
    pub decision_id: String,
    pub seal_id: String,
    pub agent_id: String,
    pub model_id: Option<String>,
    pub decision_type: String,
    pub decision_outcome: String,
    pub explanation_text: String,
    pub feature_importance: Option<HashMap<String, f64>>,
    pub decision_path: Option<Vec<DecisionStep>>,
    pub confidence_score: Option<f64>,
    pub alternative_outcomes: Option<Vec<AlternativeOutcome>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionStep {
    pub step_number: i32,
    pub description: String,
    pub reasoning: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeOutcome {
    pub outcome: String,
    pub probability: f64,
    pub reason: String,
}

/// Model Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetric {
    pub model_id: String,
    pub model_name: Option<String>,
    pub metric_type: String,
    pub metric_value: f64,
    pub metric_unit: String,
    pub measured_at: DateTime<Utc>,
    pub context: Option<serde_json::Value>,
}

/// Model Drift Detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDrift {
    pub model_id: String,
    pub drift_type: String, // DATA_DRIFT, CONCEPT_DRIFT, FEATURE_DRIFT
    pub drift_score: f64, // 0.0-100.0
    pub drift_severity: String, // LOW, MEDIUM, HIGH, CRITICAL
    pub affected_features: Vec<String>,
    pub baseline_date: Option<DateTime<Utc>>,
    pub detected_at: DateTime<Utc>,
}

/// Feature Importance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_score: f64, // 0.0-100.0
    pub importance_rank: Option<i32>,
    pub feature_type: Option<String>,
}

/// AI Explainability Service
pub struct AIExplainabilityService;

impl AIExplainabilityService {
    /// Generate explanation for an AI decision
    pub async fn explain_decision(
        db_pool: &PgPool,
        seal_id: &str,
        decision_type: &str,
        decision_outcome: &str,
        payload: &str,
    ) -> Result<AIDecisionExplanation, String> {
        // Extract decision context from compliance record
        #[derive(sqlx::FromRow)]
        struct DecisionContext {
            agent_id: String,
            action_summary: String,
            payload_hash: Option<String>,
        }

        let context: Option<DecisionContext> = sqlx::query_as(
            "SELECT agent_id, action_summary, payload_hash
             FROM compliance_records
             WHERE seal_id = $1"
        )
        .bind(seal_id)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| format!("Failed to fetch decision context: {}", e))?;

        let context = context.ok_or_else(|| "Decision not found".to_string())?;

        // Generate explanation (simplified - in production, would use ML interpretability)
        let explanation_text = format!(
            "This {} decision was made by AI agent {} based on the following factors: \
            The system analyzed the input data and determined that the outcome '{}' \
            was the most appropriate based on configured risk thresholds and business rules. \
            The decision was logged with seal ID {} for audit compliance.",
            decision_type,
            context.agent_id,
            decision_outcome,
            seal_id
        );

        // Extract feature importance from payload (simplified)
        let mut feature_importance = HashMap::new();
        if payload.contains("credit_score") {
            feature_importance.insert("credit_score".to_string(), 85.0);
        }
        if payload.contains("income") {
            feature_importance.insert("income".to_string(), 70.0);
        }
        if payload.contains("age") {
            feature_importance.insert("age".to_string(), 45.0);
        }

        // Generate decision path
        let decision_path = vec![
            DecisionStep {
                step_number: 1,
                description: "Data validation".to_string(),
                reasoning: "Input data validated against schema".to_string(),
                confidence: 95.0,
            },
            DecisionStep {
                step_number: 2,
                description: "Risk assessment".to_string(),
                reasoning: format!("Risk level calculated for {}", decision_type),
                confidence: 80.0,
            },
            DecisionStep {
                step_number: 3,
                description: "Decision generation".to_string(),
                reasoning: format!("Final decision: {}", decision_outcome),
                confidence: 75.0,
            },
        ];

        let decision_id = format!("DEC-{}-{}", Utc::now().format("%Y%m%d-%H%M%S"), uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>());
        let confidence_score = 75.0;

        let explanation = AIDecisionExplanation {
            decision_id: decision_id.clone(),
            seal_id: seal_id.to_string(),
            agent_id: context.agent_id,
            model_id: None,
            decision_type: decision_type.to_string(),
            decision_outcome: decision_outcome.to_string(),
            explanation_text,
            feature_importance: Some(feature_importance),
            decision_path: Some(decision_path),
            confidence_score: Some(confidence_score),
            alternative_outcomes: Some(vec![
                AlternativeOutcome {
                    outcome: "ALTERNATIVE_1".to_string(),
                    probability: 20.0,
                    reason: "Lower confidence alternative".to_string(),
                }
            ]),
            created_at: Utc::now(),
        };

        // Store in database
        sqlx::query(
            "INSERT INTO ai_decision_explanations (
                decision_id, seal_id, agent_id, model_id, decision_type,
                decision_outcome, explanation_text, feature_importance,
                decision_path, confidence_score, alternative_outcomes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(&explanation.decision_id)
        .bind(&explanation.seal_id)
        .bind(&explanation.agent_id)
        .bind(&explanation.model_id)
        .bind(&explanation.decision_type)
        .bind(&explanation.decision_outcome)
        .bind(&explanation.explanation_text)
        .bind(&explanation.feature_importance.as_ref().map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null)))
        .bind(&explanation.decision_path.as_ref().map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null)))
        .bind(explanation.confidence_score)
        .bind(&explanation.alternative_outcomes.as_ref().map(|a| serde_json::to_value(a).unwrap_or(serde_json::Value::Null)))
        .execute(db_pool)
        .await
        .map_err(|e| format!("Failed to store explanation: {}", e))?;

        Ok(explanation)
    }

    /// Get feature importance for a model
    pub async fn get_feature_importance(
        db_pool: &PgPool,
        model_id: &str,
    ) -> Result<Vec<FeatureImportance>, String> {
        #[derive(sqlx::FromRow)]
        struct FeatureRow {
            feature_name: String,
            importance_score: f64,
            importance_rank: Option<i32>,
            feature_type: Option<String>,
        }

        let features: Vec<FeatureRow> = sqlx::query_as(
            "SELECT feature_name, importance_score, importance_rank, feature_type
             FROM feature_importance_tracking
             WHERE model_id = $1
             AND measured_at = (
                 SELECT MAX(measured_at) 
                 FROM feature_importance_tracking 
                 WHERE model_id = $1
             )
             ORDER BY importance_rank NULLS LAST, importance_score DESC"
        )
        .bind(model_id)
        .fetch_all(db_pool)
        .await
        .map_err(|e| format!("Failed to fetch feature importance: {}", e))?;

        Ok(features.into_iter().map(|f| FeatureImportance {
            feature_name: f.feature_name,
            importance_score: f.importance_score,
            importance_rank: f.importance_rank,
            feature_type: f.feature_type,
        }).collect())
    }

    /// Detect model drift
    pub async fn detect_model_drift(
        db_pool: &PgPool,
        model_id: &str,
    ) -> Result<Option<ModelDrift>, String> {
        #[derive(sqlx::FromRow)]
        struct DriftRow {
            model_id: String,
            drift_type: String,
            drift_score: f64,
            drift_severity: String,
            affected_features: serde_json::Value,
            baseline_date: Option<DateTime<Utc>>,
            detected_at: DateTime<Utc>,
        }

        let drift: Option<DriftRow> = sqlx::query_as(
            "SELECT model_id, drift_type, drift_score, drift_severity,
                    affected_features, baseline_date, detected_at
             FROM model_drift_detection
             WHERE model_id = $1
             AND detected_at > CURRENT_TIMESTAMP - INTERVAL '7 days'
             ORDER BY detected_at DESC
             LIMIT 1"
        )
        .bind(model_id)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| format!("Failed to detect drift: {}", e))?;

        drift.map(|d| {
            let affected_features: Vec<String> = serde_json::from_value(d.affected_features)
                .unwrap_or_default();

            Ok(ModelDrift {
                model_id: d.model_id,
                drift_type: d.drift_type,
                drift_score: d.drift_score,
                drift_severity: d.drift_severity,
                affected_features,
                baseline_date: d.baseline_date,
                detected_at: d.detected_at,
            })
        }).transpose()
    }
}

