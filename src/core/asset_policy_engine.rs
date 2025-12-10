// Asset-Based Policy Engine
// Creates security policies based on business function, location, and third-party risk profile

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Asset context for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetContext {
    pub asset_id: Option<String>,
    pub business_function: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub risk_profile: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Policy match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMatch {
    pub policy_id: uuid::Uuid,
    pub policy_name: String,
    pub policy_type: String,
    pub priority: i32,
    pub policy_config: serde_json::Value,
}

/// Asset-Based Policy Engine
pub struct AssetPolicyEngine;

impl AssetPolicyEngine {
    /// Get applicable policies for an asset based on context
    pub async fn get_applicable_policies(
        db_pool: &PgPool,
        context: &AssetContext,
    ) -> Result<Vec<PolicyMatch>, String> {
        // Build query to find matching policies
        let mut query = sqlx::QueryBuilder::new(
            "SELECT id, policy_name, policy_type, priority, policy_config
             FROM asset_policies
             WHERE is_active = true"
        );

        // Add filters based on context
        if let Some(bf) = &context.business_function {
            query.push(" AND (business_function_filter IS NULL OR business_function_filter = ");
            query.push_bind(bf);
            query.push(")");
        } else {
            query.push(" AND business_function_filter IS NULL");
        }

        if let Some(dept) = &context.department {
            query.push(" AND (department_filter IS NULL OR department_filter = ");
            query.push_bind(dept);
            query.push(")");
        } else {
            query.push(" AND department_filter IS NULL");
        }

        if let Some(loc) = &context.location {
            query.push(" AND (location_filter IS NULL OR location_filter = ");
            query.push_bind(loc);
            query.push(")");
        } else {
            query.push(" AND location_filter IS NULL");
        }

        if let Some(rp) = &context.risk_profile {
            query.push(" AND (risk_profile_filter IS NULL OR risk_profile_filter = ");
            query.push_bind(rp);
            query.push(")");
        } else {
            query.push(" AND risk_profile_filter IS NULL");
        }

        // Tag filter (if asset has tags)
        if let Some(asset_tags) = &context.tags {
            if !asset_tags.is_empty() {
                query.push(" AND (asset_tags_filter IS NULL OR asset_tags_filter && ");
                query.push_bind(asset_tags);
                query.push(")");
            } else {
                query.push(" AND asset_tags_filter IS NULL");
            }
        } else {
            query.push(" AND asset_tags_filter IS NULL");
        }

        query.push(" ORDER BY priority ASC, created_at DESC");

        #[derive(sqlx::FromRow)]
        struct PolicyRow {
            id: uuid::Uuid,
            policy_name: String,
            policy_type: String,
            priority: i32,
            policy_config: serde_json::Value,
        }

        let policies: Vec<PolicyRow> = query
            .build_query_as()
            .fetch_all(db_pool)
            .await
            .map_err(|e| format!("Database query failed: {}", e))?;

        Ok(policies
            .into_iter()
            .map(|p| PolicyMatch {
                policy_id: p.id,
                policy_name: p.policy_name,
                policy_type: p.policy_type,
                priority: p.priority,
                policy_config: p.policy_config,
            })
            .collect())
    }

    /// Get asset context from agent_id
    pub async fn get_asset_context_from_agent(
        db_pool: &PgPool,
        agent_id: &str,
    ) -> Result<Option<AssetContext>, String> {
        // Try to find asset by agent mapping
        let asset: Option<(String, Option<String>, Option<String>, Option<String>, String, Option<Vec<String>>)> = sqlx::query_as(
            "SELECT a.asset_id, a.business_function, a.department, a.location, a.risk_profile, a.tags
             FROM assets a
             JOIN asset_agent_mapping aam ON a.id = aam.asset_id
             WHERE aam.agent_id = $1 AND a.is_active = true
             LIMIT 1"
        )
        .bind(agent_id)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

        match asset {
            Some((asset_id, business_function, department, location, risk_profile, tags)) => {
                Ok(Some(AssetContext {
                    asset_id: Some(asset_id),
                    business_function,
                    department,
                    location,
                    risk_profile: Some(risk_profile),
                    tags,
                }))
            }
            None => Ok(None),
        }
    }

    /// Infer business function from action type (fallback when asset not registered)
    pub fn infer_business_function_from_action(action: &str) -> Option<String> {
        let action_lower = action.to_lowercase();
        
        // Map action patterns to business functions
        if action_lower.contains("credit") || action_lower.contains("loan") || action_lower.contains("scoring") {
            Some("CREDIT_SCORING".to_string())
        } else if action_lower.contains("fraud") || action_lower.contains("suspicious") {
            Some("FRAUD_DETECTION".to_string())
        } else if action_lower.contains("customer") || action_lower.contains("support") || action_lower.contains("chatbot") {
            Some("CUSTOMER_SERVICE".to_string())
        } else if action_lower.contains("marketing") || action_lower.contains("personalization") {
            Some("MARKETING".to_string())
        } else if action_lower.contains("hiring") || action_lower.contains("recruitment") || action_lower.contains("hr") {
            Some("HR_RECRUITMENT".to_string())
        } else if action_lower.contains("risk") {
            Some("RISK_ASSESSMENT".to_string())
        } else if action_lower.contains("compliance") || action_lower.contains("monitoring") {
            Some("COMPLIANCE_MONITORING".to_string())
        } else if action_lower.contains("decision") || action_lower.contains("automated") {
            Some("AUTOMATED_DECISIONS".to_string())
        } else {
            None
        }
    }

    /// Get default risk profile for business function
    pub async fn get_default_risk_profile(
        db_pool: &PgPool,
        business_function: &str,
    ) -> Result<String, String> {
        let risk: Option<String> = sqlx::query_scalar(
            "SELECT default_risk_level FROM business_functions WHERE function_code = $1"
        )
        .bind(business_function)
        .fetch_optional(db_pool)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?
        .flatten();

        Ok(risk.unwrap_or_else(|| "MEDIUM".to_string()))
    }
}

