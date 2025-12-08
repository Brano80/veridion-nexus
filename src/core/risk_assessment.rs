// Enhanced Risk Assessment Module (EU AI Act Article 9)
// Provides context-aware, dynamic risk assessment with historical data analysis

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ToString for RiskLevel {
    fn to_string(&self) -> String {
        match self {
            RiskLevel::Low => "LOW".to_string(),
            RiskLevel::Medium => "MEDIUM".to_string(),
            RiskLevel::High => "HIGH".to_string(),
            RiskLevel::Critical => "CRITICAL".to_string(),
        }
    }
}

impl From<String> for RiskLevel {
    fn from(s: String) -> Self {
        match s.as_str() {
            "LOW" => RiskLevel::Low,
            "MEDIUM" => RiskLevel::Medium,
            "HIGH" => RiskLevel::High,
            "CRITICAL" => RiskLevel::Critical,
            _ => RiskLevel::Low,
        }
    }
}

/// Risk factor with weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub description: String,
    pub weight: f64, // 0.0 to 1.0
    pub score: f64,  // 0.0 to 1.0
}

/// Context-aware risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentResult {
    pub risk_level: RiskLevel,
    pub overall_score: f64, // 0.0 to 1.0
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_suggestions: Vec<String>,
    pub confidence: f64, // 0.0 to 1.0
    pub historical_context: Option<HistoricalContext>,
}

/// Historical context for risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalContext {
    pub similar_actions_count: i64,
    pub average_risk_score: f64,
    pub trend: String, // "INCREASING", "DECREASING", "STABLE"
    pub last_incident_days_ago: Option<i64>,
}

/// Enhanced Risk Assessment Service
pub struct RiskAssessmentService;

impl RiskAssessmentService {
    /// Perform context-aware risk assessment
    pub async fn assess_risk(
        db_pool: &PgPool,
        action: &str,
        payload: &str,
        user_id: Option<&str>,
        agent_id: Option<&str>,
        is_violation: bool,
    ) -> RiskAssessmentResult {
        // If violation, immediately return critical risk
        if is_violation {
            return RiskAssessmentResult {
                risk_level: RiskLevel::Critical,
                overall_score: 1.0,
                risk_factors: vec![RiskFactor {
                    name: "Compliance Violation".to_string(),
                    description: "Sovereignty or compliance violation detected".to_string(),
                    weight: 1.0,
                    score: 1.0,
                }],
                mitigation_suggestions: vec![
                    "Immediate human review required".to_string(),
                    "Block action execution".to_string(),
                    "Notify compliance team".to_string(),
                ],
                confidence: 1.0,
                historical_context: None,
            };
        }

        // Collect risk factors with dynamic weighting
        let mut risk_factors = Vec::new();

        // 1. Action-based risk factors
        let action_risk = Self::assess_action_risk(action, payload);
        risk_factors.push(action_risk);

        // 2. Data sensitivity risk
        let data_risk = Self::assess_data_sensitivity(payload);
        risk_factors.push(data_risk);

        // 3. User behavior risk (if user_id provided)
        let user_risk = if let Some(uid) = user_id {
            Self::assess_user_behavior_risk(db_pool, uid).await
        } else {
            RiskFactor {
                name: "User Behavior".to_string(),
                description: "No user context available".to_string(),
                weight: 0.1,
                score: 0.0,
            }
        };
        risk_factors.push(user_risk);

        // 4. Historical pattern risk
        let historical_risk = if let Some(aid) = agent_id {
            Self::assess_historical_patterns(db_pool, aid, action).await
        } else {
            RiskFactor {
                name: "Historical Patterns".to_string(),
                description: "No agent context available".to_string(),
                weight: 0.1,
                score: 0.0,
            }
        };
        risk_factors.push(historical_risk);

        // Calculate weighted overall score
        let overall_score = risk_factors.iter()
            .map(|rf| rf.weight * rf.score)
            .sum::<f64>()
            / risk_factors.iter().map(|rf| rf.weight).sum::<f64>().max(0.01);

        // Determine risk level
        let risk_level = if overall_score >= 0.8 {
            RiskLevel::High
        } else if overall_score >= 0.5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Generate mitigation suggestions
        let mitigation_suggestions = Self::generate_mitigation_suggestions(&risk_factors, overall_score);

        // Get historical context
        let historical_context = if let Some(uid) = user_id {
            Self::get_historical_context(db_pool, uid, action).await
        } else {
            None
        };

        // Calculate confidence based on available data
        let confidence = if user_id.is_some() && agent_id.is_some() {
            0.9
        } else if user_id.is_some() || agent_id.is_some() {
            0.7
        } else {
            0.5
        };

        RiskAssessmentResult {
            risk_level,
            overall_score,
            risk_factors,
            mitigation_suggestions,
            confidence,
            historical_context,
        }
    }

    /// Assess risk based on action type and payload content
    fn assess_action_risk(action: &str, payload: &str) -> RiskFactor {
        let action_lower = action.to_lowercase();
        let payload_lower = payload.to_lowercase();

        // High-risk keywords with weights
        let high_risk_patterns = vec![
            ("credit", 0.9),
            ("loan", 0.9),
            ("medical", 0.95),
            ("diagnosis", 0.95),
            ("criminal", 1.0),
            ("hiring", 0.8),
            ("firing", 0.85),
            ("insurance", 0.75),
            ("legal", 0.8),
        ];

        let mut max_score = 0.0;
        let mut matched_keywords = Vec::new();

        for (keyword, weight) in high_risk_patterns {
            if action_lower.contains(keyword) || payload_lower.contains(keyword) {
                max_score = max_score.max(weight);
                matched_keywords.push(keyword);
            }
        }

        // Medium-risk patterns
        if max_score == 0.0 {
            let medium_patterns = vec!["transaction", "payment", "financial", "personal"];
            for pattern in medium_patterns {
                if action_lower.contains(pattern) || payload_lower.contains(pattern) {
                    max_score = 0.5;
                    matched_keywords.push(pattern);
                    break;
                }
            }
        }

        RiskFactor {
            name: "Action Type Risk".to_string(),
            description: format!(
                "Action: {} | Matched patterns: {}",
                action,
                if matched_keywords.is_empty() {
                    "None (low risk)".to_string()
                } else {
                    matched_keywords.join(", ")
                }
            ),
            weight: 0.4,
            score: max_score,
        }
    }

    /// Assess data sensitivity risk
    fn assess_data_sensitivity(payload: &str) -> RiskFactor {
        let payload_lower = payload.to_lowercase();

        // Sensitive data patterns
        let sensitive_patterns = vec![
            ("ssn", 0.9),
            ("social security", 0.9),
            ("passport", 0.85),
            ("credit card", 0.95),
            ("bank account", 0.9),
            ("biometric", 0.95),
            ("health record", 0.95),
            ("genetic", 0.9),
        ];

        let mut max_score = 0.0;
        let mut matched = Vec::new();

        for (pattern, weight) in sensitive_patterns {
            if payload_lower.contains(pattern) {
                max_score = max_score.max(weight);
                matched.push(pattern);
            }
        }

        // Check payload size (larger payloads may contain more sensitive data)
        let size_score = if payload.len() > 10000 {
            0.3
        } else if payload.len() > 5000 {
            0.2
        } else {
            0.0
        };

        max_score = max_score.max(size_score);

        RiskFactor {
            name: "Data Sensitivity".to_string(),
            description: format!(
                "Sensitive data patterns: {}",
                if matched.is_empty() {
                    "None detected".to_string()
                } else {
                    matched.join(", ")
                }
            ),
            weight: 0.3,
            score: max_score,
        }
    }

    /// Assess user behavior risk based on historical actions
    async fn assess_user_behavior_risk(db_pool: &PgPool, user_id: &str) -> RiskFactor {
        // Check for recent high-risk actions by this user
        let recent_high_risk: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM compliance_records cr
             JOIN risk_assessments ra ON cr.seal_id = ra.seal_id
             WHERE cr.user_id = $1
               AND ra.risk_level IN ('HIGH', 'CRITICAL')
               AND cr.timestamp > NOW() - INTERVAL '30 days'"
        )
        .bind(user_id)
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None);

        let score = if let Some(count) = recent_high_risk {
            if count > 10 {
                0.8
            } else if count > 5 {
                0.6
            } else if count > 0 {
                0.4
            } else {
                0.1
            }
        } else {
            0.1
        };

        RiskFactor {
            name: "User Behavior Risk".to_string(),
            description: format!(
                "User has {} high-risk actions in last 30 days",
                recent_high_risk.unwrap_or(0)
            ),
            weight: 0.2,
            score,
        }
    }

    /// Assess historical patterns for agent
    async fn assess_historical_patterns(db_pool: &PgPool, agent_id: &str, action: &str) -> RiskFactor {
        // Check frequency of similar actions
        let similar_count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM compliance_records
             WHERE agent_id = $1
               AND action LIKE $2
               AND timestamp > NOW() - INTERVAL '7 days'"
        )
        .bind(agent_id)
        .bind(format!("%{}%", action))
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None);

        let score = if let Some(count) = similar_count {
            // High frequency may indicate automation or systematic risk
            if count > 100 {
                0.7
            } else if count > 50 {
                0.5
            } else if count > 10 {
                0.3
            } else {
                0.1
            }
        } else {
            0.1
        };

        RiskFactor {
            name: "Historical Pattern Risk".to_string(),
            description: format!(
                "Agent performed {} similar actions in last 7 days",
                similar_count.unwrap_or(0)
            ),
            weight: 0.1,
            score,
        }
    }

    /// Generate mitigation suggestions based on risk factors
    fn generate_mitigation_suggestions(risk_factors: &[RiskFactor], overall_score: f64) -> Vec<String> {
        let mut suggestions = Vec::new();

        if overall_score >= 0.8 {
            suggestions.push("Require human oversight before execution".to_string());
            suggestions.push("Implement additional verification steps".to_string());
            suggestions.push("Notify compliance team".to_string());
        } else if overall_score >= 0.5 {
            suggestions.push("Consider human review for this action".to_string());
            suggestions.push("Monitor action outcome closely".to_string());
        }

        // Check for specific high-risk factors
        for factor in risk_factors {
            if factor.score >= 0.8 {
                match factor.name.as_str() {
                    "Action Type Risk" => {
                        suggestions.push("Apply enhanced logging and audit trail".to_string());
                    }
                    "Data Sensitivity" => {
                        suggestions.push("Ensure data encryption and access controls".to_string());
                    }
                    "User Behavior Risk" => {
                        suggestions.push("Review user's recent activity patterns".to_string());
                    }
                    _ => {}
                }
            }
        }

        if suggestions.is_empty() {
            suggestions.push("Standard safeguards are sufficient".to_string());
        }

        suggestions
    }

    /// Get historical context for risk assessment
    async fn get_historical_context(
        db_pool: &PgPool,
        user_id: &str,
        action: &str,
    ) -> Option<HistoricalContext> {
        // Get similar actions count
        let similar_count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM compliance_records
             WHERE user_id = $1
               AND action LIKE $2
               AND timestamp > NOW() - INTERVAL '90 days'"
        )
        .bind(user_id)
        .bind(format!("%{}%", action))
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None)?;

        // Get average risk score
        let avg_score: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(
                CASE ra.risk_level
                    WHEN 'LOW' THEN 0.25
                    WHEN 'MEDIUM' THEN 0.5
                    WHEN 'HIGH' THEN 0.75
                    WHEN 'CRITICAL' THEN 1.0
                    ELSE 0.0
                END
             )
             FROM compliance_records cr
             JOIN risk_assessments ra ON cr.seal_id = ra.seal_id
             WHERE cr.user_id = $1
               AND cr.action LIKE $2
               AND cr.timestamp > NOW() - INTERVAL '90 days'"
        )
        .bind(user_id)
        .bind(format!("%{}%", action))
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None)?;

        // Get last incident
        let last_incident: Option<chrono::DateTime<Utc>> = sqlx::query_scalar(
            "SELECT MAX(cr.timestamp)
             FROM compliance_records cr
             JOIN risk_assessments ra ON cr.seal_id = ra.seal_id
             WHERE cr.user_id = $1
               AND ra.risk_level IN ('HIGH', 'CRITICAL')
               AND cr.timestamp > NOW() - INTERVAL '365 days'"
        )
        .bind(user_id)
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None)?;

        let last_incident_days_ago = last_incident.map(|dt| {
            (Utc::now() - dt).num_days()
        });

        // Determine trend (simplified - compare last 30 days to previous 30 days)
        let recent_count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM compliance_records
             WHERE user_id = $1
               AND action LIKE $2
               AND timestamp > NOW() - INTERVAL '30 days'"
        )
        .bind(user_id)
        .bind(format!("%{}%", action))
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None)?;

        let previous_count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM compliance_records
             WHERE user_id = $1
               AND action LIKE $2
               AND timestamp BETWEEN NOW() - INTERVAL '60 days' AND NOW() - INTERVAL '30 days'"
        )
        .bind(user_id)
        .bind(format!("%{}%", action))
        .fetch_optional(db_pool)
        .await
        .unwrap_or(None)?;

        let trend = match (recent_count, previous_count) {
            (Some(rec), Some(prev)) if rec > prev => "INCREASING",
            (Some(rec), Some(prev)) if rec < prev => "DECREASING",
            _ => "STABLE",
        };

        Some(HistoricalContext {
            similar_actions_count: similar_count.unwrap_or(0),
            average_risk_score: avg_score.unwrap_or(0.0),
            trend: trend.to_string(),
            last_incident_days_ago,
        })
    }

    /// Predict risk for future actions (simplified ML-like approach)
    pub async fn predict_risk(
        db_pool: &PgPool,
        action: &str,
        user_id: Option<&str>,
    ) -> f64 {
        // Use historical data to predict risk
        if let Some(uid) = user_id {
            let historical_avg: Option<f64> = sqlx::query_scalar(
                "SELECT AVG(
                    CASE ra.risk_level
                        WHEN 'LOW' THEN 0.25
                        WHEN 'MEDIUM' THEN 0.5
                        WHEN 'HIGH' THEN 0.75
                        WHEN 'CRITICAL' THEN 1.0
                        ELSE 0.0
                    END
                 )
                 FROM compliance_records cr
                 JOIN risk_assessments ra ON cr.seal_id = ra.seal_id
                 WHERE cr.user_id = $1
                   AND cr.action LIKE $2
                   AND cr.timestamp > NOW() - INTERVAL '90 days'"
            )
            .bind(uid)
            .bind(format!("%{}%", action))
            .fetch_optional(db_pool)
            .await
            .unwrap_or(None);

            historical_avg.unwrap_or(0.5)
        } else {
            // Fallback to action-based risk
            let action_risk = Self::assess_action_risk(action, "");
            action_risk.score
        }
    }
}

