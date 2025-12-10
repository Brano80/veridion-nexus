// Wizard Service: Handles company profiles, module recommendations, pricing, and trial management

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use std::collections::HashMap;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyProfile {
    pub id: Uuid,
    pub company_name: String,
    pub industry: String,
    pub company_size: String,
    pub country: String,
    pub regulatory_requirements: Vec<String>,
    pub ai_use_cases: Vec<String>,
    pub deployment_preference: String,
    pub estimated_ai_systems: i32,
    pub wizard_completed: bool,
    pub wizard_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompanyProfileRequest {
    pub company_name: String,
    pub industry: String,
    pub company_size: String,
    pub country: String,
    pub regulatory_requirements: Vec<String>,
    pub ai_use_cases: Vec<String>,
    pub deployment_preference: String,
    pub estimated_ai_systems: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedModule {
    pub module_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub recommendation_reason: String,
    pub priority: String, // 'REQUIRED', 'RECOMMENDED', 'OPTIONAL'
    pub requires_license: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRecommendationResponse {
    pub recommended_modules: Vec<RecommendedModule>,
    pub required_count: i32,
    pub recommended_count: i32,
    pub optional_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingBreakdown {
    pub base_price: f64,
    pub per_system_price: f64,
    pub module_prices: HashMap<String, f64>,
    pub total_monthly: f64,
    pub total_annual: f64,
    pub savings_annual: f64, // Savings when paying annually
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub company_id: Uuid,
    pub subscription_type: String,
    pub status: String,
    pub trial_start_date: Option<DateTime<Utc>>,
    pub trial_end_date: Option<DateTime<Utc>>,
    pub license_start_date: Option<DateTime<Utc>>,
    pub license_end_date: Option<DateTime<Utc>>,
    pub monthly_price: Option<Decimal>,
    pub annual_price: Option<Decimal>,
    pub billing_cycle: String,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTrialRequest {
    pub company_id: Uuid,
    pub selected_modules: Vec<String>,
    pub estimated_ai_systems: i32,
}

pub struct WizardService {
    pool: PgPool,
}

impl WizardService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create or update a company profile
    pub async fn create_or_update_company_profile(
        &self,
        request: CreateCompanyProfileRequest,
        created_by: Option<Uuid>,
    ) -> Result<CompanyProfile, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO company_profiles (
                company_name, industry, company_size, country,
                regulatory_requirements, ai_use_cases, deployment_preference,
                estimated_ai_systems, created_by, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, COALESCE($10, '{}'::jsonb))
            ON CONFLICT (company_name) DO UPDATE SET
                industry = EXCLUDED.industry,
                company_size = EXCLUDED.company_size,
                country = EXCLUDED.country,
                regulatory_requirements = EXCLUDED.regulatory_requirements,
                ai_use_cases = EXCLUDED.ai_use_cases,
                deployment_preference = EXCLUDED.deployment_preference,
                estimated_ai_systems = EXCLUDED.estimated_ai_systems,
                metadata = EXCLUDED.metadata,
                updated_at = CURRENT_TIMESTAMP
            RETURNING
                id, company_name, industry, company_size, country,
                regulatory_requirements, ai_use_cases,
                deployment_preference, estimated_ai_systems,
                wizard_completed, wizard_completed_at, created_at, updated_at, metadata
            "#
        )
        .bind(&request.company_name)
        .bind(&request.industry)
        .bind(&request.company_size)
        .bind(&request.country)
        .bind(&request.regulatory_requirements)
        .bind(&request.ai_use_cases)
        .bind(&request.deployment_preference)
        .bind(request.estimated_ai_systems)
        .bind(created_by)
        .bind(request.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(CompanyProfile {
            id: row.get("id"),
            company_name: row.get("company_name"),
            industry: row.get("industry"),
            company_size: row.get("company_size"),
            country: row.get("country"),
            regulatory_requirements: row.get("regulatory_requirements"),
            ai_use_cases: row.get("ai_use_cases"),
            deployment_preference: row.get("deployment_preference"),
            estimated_ai_systems: row.get("estimated_ai_systems"),
            wizard_completed: row.get("wizard_completed"),
            wizard_completed_at: row.get("wizard_completed_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        })
    }

    /// Get company profile by ID
    pub async fn get_company_profile(&self, company_id: Uuid) -> Result<Option<CompanyProfile>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id, company_name, industry, company_size, country,
                regulatory_requirements, ai_use_cases,
                deployment_preference, estimated_ai_systems,
                wizard_completed, wizard_completed_at, created_at, updated_at, metadata
            FROM company_profiles
            WHERE id = $1
            "#
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(CompanyProfile {
                id: row.get("id"),
                company_name: row.get("company_name"),
                industry: row.get("industry"),
                company_size: row.get("company_size"),
                country: row.get("country"),
                regulatory_requirements: row.get("regulatory_requirements"),
                ai_use_cases: row.get("ai_use_cases"),
                deployment_preference: row.get("deployment_preference"),
                estimated_ai_systems: row.get("estimated_ai_systems"),
                wizard_completed: row.get("wizard_completed"),
                wizard_completed_at: row.get("wizard_completed_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                metadata: row.get("metadata"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Mark wizard as completed
    pub async fn mark_wizard_completed(&self, company_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE company_profiles
            SET wizard_completed = true,
                wizard_completed_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            company_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recommended modules based on company profile
    pub async fn get_recommended_modules(
        &self,
        industry: &str,
        regulatory_requirements: &[String],
        ai_use_cases: &[String],
    ) -> Result<ModuleRecommendationResponse, sqlx::Error> {
        // Call the database function to get recommendations
        let rows = sqlx::query!(
            r#"
            SELECT module_name, recommendation_reason, priority
            FROM get_recommended_modules($1, $2, $3)
            "#,
            industry,
            regulatory_requirements,
            ai_use_cases
        )
        .fetch_all(&self.pool)
        .await?;

        // Get module details
        let module_names: Vec<String> = rows.iter().map(|r| r.module_name.clone()).collect();
        
        let module_details = sqlx::query!(
            r#"
            SELECT name, display_name, description, category, requires_license
            FROM modules
            WHERE name = ANY($1)
            "#,
            &module_names
        )
        .fetch_all(&self.pool)
        .await?;

        // Create a map of module details
        let module_map: HashMap<String, _> = module_details
            .into_iter()
            .map(|m| (m.name.clone(), m))
            .collect();

        // Build recommended modules list
        let mut recommended_modules = Vec::new();
        let mut required_count = 0;
        let mut recommended_count = 0;
        let mut optional_count = 0;

        for row in rows {
            if let Some(details) = module_map.get(&row.module_name) {
                match row.priority.as_str() {
                    "REQUIRED" => required_count += 1,
                    "RECOMMENDED" => recommended_count += 1,
                    _ => optional_count += 1,
                }

                recommended_modules.push(RecommendedModule {
                    module_name: row.module_name.clone(),
                    display_name: details.display_name.clone(),
                    description: details.description.clone(),
                    category: details.category.clone(),
                    recommendation_reason: row.recommendation_reason,
                    priority: row.priority,
                    requires_license: details.requires_license,
                });
            }
        }

        Ok(ModuleRecommendationResponse {
            recommended_modules,
            required_count,
            recommended_count,
            optional_count,
        })
    }

    /// Calculate pricing based on modules and number of systems
    pub async fn calculate_pricing(
        &self,
        selected_modules: &[String],
        num_systems: i32,
    ) -> Result<PricingBreakdown, sqlx::Error> {
        // Base pricing structure
        let base_price = 299.0; // €299/month base
        let per_system_price = 100.0; // €100/system/month

        // Module pricing (monthly)
        let module_prices: HashMap<String, f64> = HashMap::from([
            ("module_human_oversight".to_string(), 200.0),
            ("module_consent".to_string(), 150.0),
            ("module_dpia".to_string(), 100.0),
            ("module_retention".to_string(), 100.0),
            ("module_breach_management".to_string(), 150.0),
            ("module_data_subject_rights".to_string(), 150.0),
            ("module_risk_assessment".to_string(), 200.0),
            ("module_monitoring".to_string(), 100.0),
            ("module_green_ai".to_string(), 50.0),
            ("module_ai_bom".to_string(), 100.0),
        ]);

        // Calculate module costs
        let module_cost: f64 = selected_modules
            .iter()
            .filter_map(|m| module_prices.get(m))
            .sum();

        // Calculate total monthly
        let total_monthly = base_price + (num_systems as f64 * per_system_price) + module_cost;

        // Calculate annual (with 15% discount)
        let total_annual = total_monthly * 12.0 * 0.85;
        let savings_annual = (total_monthly * 12.0) - total_annual;

        Ok(PricingBreakdown {
            base_price,
            per_system_price,
            module_prices,
            total_monthly,
            total_annual,
            savings_annual,
        })
    }

    /// Start a free trial (3 months in Shadow Mode)
    pub async fn start_trial(
        &self,
        request: StartTrialRequest,
    ) -> Result<Subscription, sqlx::Error> {
        let trial_start = Utc::now();
        let trial_end = trial_start + chrono::Duration::days(90); // 3 months

        // Calculate pricing
        let pricing = self.calculate_pricing(&request.selected_modules, request.estimated_ai_systems).await?;

        // Create subscription
        let row = sqlx::query(
            r#"
            INSERT INTO subscriptions (
                company_id, subscription_type, status,
                trial_start_date, trial_end_date,
                monthly_price, annual_price, billing_cycle
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id, company_id, subscription_type, status,
                trial_start_date, trial_end_date,
                license_start_date, license_end_date,
                monthly_price, annual_price, billing_cycle,
                auto_renew, created_at, updated_at
            "#
        )
        .bind(request.company_id)
        .bind("TRIAL")
        .bind("TRIAL")
        .bind(trial_start)
        .bind(trial_end)
        .bind(Decimal::try_from(pricing.total_monthly).unwrap_or(Decimal::ZERO))
        .bind(Decimal::try_from(pricing.total_annual).unwrap_or(Decimal::ZERO))
        .bind("MONTHLY")
        .fetch_one(&self.pool)
        .await?;

        let subscription = Subscription {
            id: row.get("id"),
            company_id: row.get("company_id"),
            subscription_type: row.get("subscription_type"),
            status: row.get("status"),
            trial_start_date: row.get("trial_start_date"),
            trial_end_date: row.get("trial_end_date"),
            license_start_date: row.get("license_start_date"),
            license_end_date: row.get("license_end_date"),
            monthly_price: row.get("monthly_price"),
            annual_price: row.get("annual_price"),
            billing_cycle: row.get("billing_cycle"),
            auto_renew: row.get("auto_renew"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        // Enable selected modules for this subscription
        for module_name in request.selected_modules {
            if let Ok(Some(module_id)) = sqlx::query!(
                "SELECT id FROM modules WHERE name = $1",
                module_name
            )
            .fetch_optional(&self.pool)
            .await
            {
                sqlx::query!(
                    r#"
                    INSERT INTO subscription_modules (subscription_id, module_id, included)
                    VALUES ($1, $2, true)
                    ON CONFLICT (subscription_id, module_id) DO UPDATE SET included = true
                    "#,
                    subscription.id,
                    module_id.id
                )
                .execute(&self.pool)
                .await?;
            }
        }

        // Mark wizard as completed
        self.mark_wizard_completed(request.company_id).await?;

        Ok(subscription)
    }

    /// Get current subscription for a company
    pub async fn get_current_subscription(&self, company_id: Uuid) -> Result<Option<Subscription>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id, company_id, subscription_type, status,
                trial_start_date, trial_end_date,
                license_start_date, license_end_date,
                monthly_price, annual_price, billing_cycle,
                auto_renew, created_at, updated_at
            FROM subscriptions
            WHERE company_id = $1
            AND status IN ('TRIAL', 'ACTIVE')
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Subscription {
                id: row.get("id"),
                company_id: row.get("company_id"),
                subscription_type: row.get("subscription_type"),
                status: row.get("status"),
                trial_start_date: row.get("trial_start_date"),
                trial_end_date: row.get("trial_end_date"),
                license_start_date: row.get("license_start_date"),
                license_end_date: row.get("license_end_date"),
                monthly_price: row.get("monthly_price"),
                annual_price: row.get("annual_price"),
                billing_cycle: row.get("billing_cycle"),
                auto_renew: row.get("auto_renew"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if trial is still active
    pub async fn is_trial_active(&self, company_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            "SELECT is_trial_active($1)",
            company_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Upgrade from trial to paid subscription
    pub async fn upgrade_to_paid(
        &self,
        company_id: Uuid,
        subscription_type: &str, // 'DEVELOPER', 'STARTUP', 'PROFESSIONAL', 'ENTERPRISE'
        billing_cycle: &str, // 'MONTHLY', 'ANNUAL'
    ) -> Result<Subscription, sqlx::Error> {
        // Get current subscription
        let current_sub = self.get_current_subscription(company_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let license_start = Utc::now();
        let license_end = match billing_cycle {
            "ANNUAL" => license_start + chrono::Duration::days(365),
            _ => license_start + chrono::Duration::days(30),
        };

        // Update subscription
        let row = sqlx::query(
            r#"
            UPDATE subscriptions
            SET subscription_type = $1,
                status = 'ACTIVE',
                license_start_date = $2,
                license_end_date = $3,
                billing_cycle = $4,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $5
            RETURNING
                id, company_id, subscription_type, status,
                trial_start_date, trial_end_date,
                license_start_date, license_end_date,
                monthly_price, annual_price, billing_cycle,
                auto_renew, created_at, updated_at
            "#
        )
        .bind(subscription_type)
        .bind(license_start)
        .bind(license_end)
        .bind(billing_cycle)
        .bind(current_sub.id)
        .fetch_one(&self.pool)
        .await?;

        let subscription = Subscription {
            id: row.get("id"),
            company_id: row.get("company_id"),
            subscription_type: row.get("subscription_type"),
            status: row.get("status"),
            trial_start_date: row.get("trial_start_date"),
            trial_end_date: row.get("trial_end_date"),
            license_start_date: row.get("license_start_date"),
            license_end_date: row.get("license_end_date"),
            monthly_price: row.get("monthly_price"),
            annual_price: row.get("annual_price"),
            billing_cycle: row.get("billing_cycle"),
            auto_renew: row.get("auto_renew"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(subscription)
    }
}

