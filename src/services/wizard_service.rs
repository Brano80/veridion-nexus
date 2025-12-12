// Wizard Service: Handles company profiles, module recommendations, pricing, and trial management

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{Utc, DateTime, NaiveDateTime};
use std::collections::HashMap;
use rust_decimal::Decimal;
use serde_json;

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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecommendedModule {
    pub module_name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub recommendation_reason: String,
    pub priority: String, // 'REQUIRED', 'RECOMMENDED', 'OPTIONAL'
    pub requires_license: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModuleRecommendationResponse {
    pub core_modules: Vec<RecommendedModule>,
    pub recommended_modules: Vec<RecommendedModule>,
    pub required_count: i32,
    pub recommended_count: i32,
    pub optional_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
            created_at: row.try_get::<chrono::DateTime<Utc>, _>("created_at")
                .or_else(|_| {
                    // Fallback: if TIMESTAMP (without timezone), parse as UTC
                    row.try_get::<chrono::NaiveDateTime, _>("created_at")
                        .map(|dt| dt.and_utc())
                })?,
            updated_at: row.try_get::<chrono::DateTime<Utc>, _>("updated_at")
                .or_else(|_| {
                    // Fallback: if TIMESTAMP (without timezone), parse as UTC
                    row.try_get::<chrono::NaiveDateTime, _>("updated_at")
                        .map(|dt| dt.and_utc())
                })?,
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
                created_at: row.try_get::<DateTime<Utc>, _>("created_at")
                    .or_else(|_| {
                        row.try_get::<NaiveDateTime, _>("created_at")
                            .map(|dt| dt.and_utc())
                    })?,
                updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")
                    .or_else(|_| {
                        row.try_get::<NaiveDateTime, _>("updated_at")
                            .map(|dt| dt.and_utc())
                    })?,
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
    /// This is a simplified version that uses the new schema
    pub async fn get_recommended_modules(
        &self,
        industry: &str,
        regulatory_requirements: &[String],
        ai_use_cases: &[String],
    ) -> Result<ModuleRecommendationResponse, sqlx::Error> {
        // Use the enhanced method but with default country and company_size
        // This ensures we use the new schema with module_id
        self.get_recommended_modules_enhanced(
            industry,
            "", // country - not used in basic version
            regulatory_requirements,
            ai_use_cases,
            "", // company_size - not used in basic version
        )
        .await
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
            ("gdpr_article_44_49".to_string(), 200.0),
            ("gdpr_article_28".to_string(), 150.0),
            ("gdpr_article_12".to_string(), 75.0),
        ]);

        // Calculate module costs
        let module_cost: f64 = selected_modules
            .iter()
            .filter_map(|m| {
                // Try exact match first
                if let Some(&price) = module_prices.get(m) {
                    return Some(price);
                }
                // Debug: log if module not found
                eprintln!("Warning: Module '{}' not found in pricing HashMap", m);
                None
            })
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
            created_at: row.try_get::<chrono::DateTime<Utc>, _>("created_at")
                .or_else(|_| {
                    // Fallback: if TIMESTAMP (without timezone), parse as UTC
                    row.try_get::<chrono::NaiveDateTime, _>("created_at")
                        .map(|dt| dt.and_utc())
                })?,
            updated_at: row.try_get::<chrono::DateTime<Utc>, _>("updated_at")
                .or_else(|_| {
                    // Fallback: if TIMESTAMP (without timezone), parse as UTC
                    row.try_get::<chrono::NaiveDateTime, _>("updated_at")
                        .map(|dt| dt.and_utc())
                })?,
        };

        // Get company profile for auto-enable logic
        let profile = self.get_company_profile(request.company_id).await?;
        
        // 1. Auto-enable modules based on company profile conditions
        if let Some(ref company_profile) = profile {
            let auto_enabled = self.auto_enable_modules(request.company_id, company_profile).await?;
            eprintln!("Auto-enabled {} modules based on profile conditions", auto_enabled.len());
        }

        // 2. Enable selected modules for this subscription (billing)
        for module_name in &request.selected_modules {
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

        // 3. Activate selected modules for company (runtime activation)
        // Use the same pattern as auto_enable_modules - direct SQL insert
        for module_name in &request.selected_modules {
            sqlx::query(
                r#"
                INSERT INTO company_module_configs (company_id, module_id, enabled, configured_by)
                SELECT $1, id, true, NULL
                FROM modules
                WHERE name = $2
                ON CONFLICT (company_id, module_id)
                DO UPDATE SET
                    enabled = true,
                    configured_at = CURRENT_TIMESTAMP,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(request.company_id)
            .bind(module_name)
            .execute(&self.pool)
            .await?;
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
                created_at: row.try_get::<DateTime<Utc>, _>("created_at")
                    .or_else(|_| {
                        row.try_get::<NaiveDateTime, _>("created_at")
                            .map(|dt| dt.and_utc())
                    })?,
                updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")
                    .or_else(|_| {
                        row.try_get::<NaiveDateTime, _>("updated_at")
                            .map(|dt| dt.and_utc())
                    })?,
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
            created_at: row.try_get::<chrono::DateTime<Utc>, _>("created_at")
                .or_else(|_| {
                    // Fallback: if TIMESTAMP (without timezone), parse as UTC
                    row.try_get::<chrono::NaiveDateTime, _>("created_at")
                        .map(|dt| dt.and_utc())
                })?,
            updated_at: row.try_get::<chrono::DateTime<Utc>, _>("updated_at")
                .or_else(|_| {
                    // Fallback: if TIMESTAMP (without timezone), parse as UTC
                    row.try_get::<chrono::NaiveDateTime, _>("updated_at")
                        .map(|dt| dt.and_utc())
                })?,
        };

        Ok(subscription)
    }

    // ============================================================================
    // NEW METHODS: Enhanced Module System
    // ============================================================================

    /// Get recommended modules with enhanced logic (uses new module_regulation_mapping)
    pub async fn get_recommended_modules_enhanced(
        &self,
        industry: &str,
        country: &str,
        regulatory_requirements: &[String],
        ai_use_cases: &[String],
        company_size: &str,
    ) -> Result<ModuleRecommendationResponse, sqlx::Error> {
        use sqlx::Row;

        // 0. Get core modules (always included, no pricing)
        let core_modules_rows = sqlx::query(
            r#"
            SELECT m.id, m.name, m.display_name, m.description, m.category, m.requires_license
            FROM modules m
            WHERE m.category = 'core'
            AND m.name IN ('core_sovereign_lock', 'core_crypto_shredder', 'core_privacy_bridge', 'core_annex_iv')
            ORDER BY m.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let core_modules: Vec<RecommendedModule> = core_modules_rows
            .iter()
            .map(|row| RecommendedModule {
                module_name: row.get("name"),
                display_name: row.get("display_name"),
                description: row.get("description"),
                category: row.get("category"),
                recommendation_reason: "Core module - always included".to_string(),
                priority: "CORE".to_string(),
                requires_license: row.get("requires_license"),
            })
            .collect();

        // 1. Get industry-based recommendations
        let industry_modules = sqlx::query(
            r#"
            SELECT m.id, m.name, m.display_name, m.description, m.category, m.requires_license,
                   imr.priority, imr.recommendation_reason
            FROM modules m
            JOIN industry_module_recommendations imr ON m.id = imr.module_id
            WHERE imr.industry = $1
            ORDER BY 
                CASE imr.priority
                    WHEN 'REQUIRED' THEN 1
                    WHEN 'RECOMMENDED' THEN 2
                    WHEN 'OPTIONAL' THEN 3
                END,
                m.name
            "#,
        )
        .bind(industry)
        .fetch_all(&self.pool)
        .await?;

        // 2. Get regulation-based requirements
        let mut regulation_modules = Vec::new();
        for regulation in regulatory_requirements {
            let modules = sqlx::query(
                r#"
                SELECT m.id, m.name, m.display_name, m.description, m.category, m.requires_license,
                       mrm.requirement_level as priority,
                       'Required for ' || mrm.regulation || ' ' || COALESCE(mrm.article_number, '') as recommendation_reason
                FROM modules m
                JOIN module_regulation_mapping mrm ON m.id = mrm.module_id
                WHERE mrm.regulation = $1
                ORDER BY 
                    CASE mrm.requirement_level
                        WHEN 'MANDATORY' THEN 1
                        WHEN 'RECOMMENDED' THEN 2
                        WHEN 'OPTIONAL' THEN 3
                    END,
                    m.name
                "#,
            )
            .bind(regulation)
            .fetch_all(&self.pool)
            .await?;
            regulation_modules.extend(modules);
        }

        // 3. Get use case-based recommendations
        let mut use_case_modules = Vec::new();
        for use_case in ai_use_cases {
            let modules = sqlx::query(
                r#"
                SELECT m.id, m.name, m.display_name, m.description, m.category, m.requires_license,
                       ucmr.priority,
                       ucmr.recommendation_reason
                FROM modules m
                JOIN use_case_module_recommendations ucmr ON m.id = ucmr.module_id
                WHERE ucmr.use_case = $1
                ORDER BY 
                    CASE ucmr.priority
                        WHEN 'REQUIRED' THEN 1
                        WHEN 'RECOMMENDED' THEN 2
                        WHEN 'OPTIONAL' THEN 3
                    END,
                    m.name
                "#,
            )
            .bind(use_case)
            .fetch_all(&self.pool)
            .await?;
            use_case_modules.extend(modules);
        }

        // 4. Merge and deduplicate modules (prioritize by requirement level)
        let mut module_map: HashMap<String, RecommendedModule> = HashMap::new();
        let mut required_count = 0;
        let mut recommended_count = 0;
        let mut optional_count = 0;

        // Process industry modules
        for row in industry_modules {
            let module_name: String = row.get("name");
            let priority: String = row.get("priority");
            
            if !module_map.contains_key(&module_name) {
                match priority.as_str() {
                    "REQUIRED" => required_count += 1,
                    "RECOMMENDED" => recommended_count += 1,
                    _ => optional_count += 1,
                }

                module_map.insert(module_name.clone(), RecommendedModule {
                    module_name: module_name.clone(),
                    display_name: row.get("display_name"),
                    description: row.get("description"),
                    category: row.get("category"),
                    recommendation_reason: row.get("recommendation_reason"),
                    priority: priority.clone(),
                    requires_license: row.get("requires_license"),
                });
            }
        }

        // Process regulation modules (override with higher priority if needed)
        for row in regulation_modules {
            let module_name: String = row.get("name");
            let priority: String = row.get("priority");
            
            if let Some(existing) = module_map.get_mut(&module_name) {
                // Upgrade priority if regulation requirement is higher
                if priority == "MANDATORY" && existing.priority != "REQUIRED" {
                    if existing.priority == "RECOMMENDED" { recommended_count -= 1; }
                    if existing.priority == "OPTIONAL" { optional_count -= 1; }
                    required_count += 1;
                    existing.priority = "REQUIRED".to_string();
                }
                existing.recommendation_reason = row.get("recommendation_reason");
            } else {
                match priority.as_str() {
                    "MANDATORY" | "REQUIRED" => required_count += 1,
                    "RECOMMENDED" => recommended_count += 1,
                    _ => optional_count += 1,
                }

                module_map.insert(module_name.clone(), RecommendedModule {
                    module_name: module_name.clone(),
                    display_name: row.get("display_name"),
                    description: row.get("description"),
                    category: row.get("category"),
                    recommendation_reason: row.get("recommendation_reason"),
                    priority: if priority == "MANDATORY" { "REQUIRED".to_string() } else { priority },
                    requires_license: row.get("requires_license"),
                });
            }
        }

        // Process use case modules
        for row in use_case_modules {
            let module_name: String = row.get("name");
            let priority: String = row.get("priority");
            
            if !module_map.contains_key(&module_name) {
                match priority.as_str() {
                    "REQUIRED" => required_count += 1,
                    "RECOMMENDED" => recommended_count += 1,
                    _ => optional_count += 1,
                }

                module_map.insert(module_name.clone(), RecommendedModule {
                    module_name: module_name.clone(),
                    display_name: row.get("display_name"),
                    description: row.get("description"),
                    category: row.get("category"),
                    recommendation_reason: row.get("recommendation_reason"),
                    priority: priority.clone(),
                    requires_license: row.get("requires_license"),
                });
            }
        }

        let mut recommended_modules: Vec<RecommendedModule> = module_map.into_values().collect();
        
        // Filter out core modules from recommended modules (they should not appear twice)
        let core_module_names: std::collections::HashSet<String> = core_modules
            .iter()
            .map(|m| m.module_name.clone())
            .collect();
        recommended_modules.retain(|m| !core_module_names.contains(&m.module_name));

        Ok(ModuleRecommendationResponse {
            core_modules,
            recommended_modules,
            required_count,
            recommended_count,
            optional_count,
        })
    }

    /// Auto-enable modules based on company profile conditions
    pub async fn auto_enable_modules(
        &self,
        company_id: Uuid,
        profile: &CompanyProfile,
    ) -> Result<Vec<String>, sqlx::Error> {
        use sqlx::Row;

        // Get modules with auto-enable conditions
        let rows = sqlx::query(
            r#"
            SELECT name, auto_enable_conditions
            FROM modules
            WHERE auto_enable_conditions IS NOT NULL
            AND jsonb_typeof(auto_enable_conditions) = 'object'
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut to_enable = Vec::new();

        for row in rows {
            let module_name: String = row.get("name");
            let conditions: serde_json::Value = row.get("auto_enable_conditions");

            // Check if conditions match profile
            // Support multiple condition types (industry array, regulations array, country)
            let should_enable = {
                let mut matches = false;
                
                // Check industry (supports both string and array)
                if let Some(industry) = conditions.get("industry") {
                    if let Some(industry_str) = industry.as_str() {
                        matches = matches || profile.industry == industry_str;
                    } else if let Some(industry_array) = industry.as_array() {
                        matches = matches || industry_array.iter().any(|i| {
                            i.as_str().map(|s| profile.industry == s).unwrap_or(false)
                        });
                    }
                }
                
                // Check regulations (array)
                if let Some(regulations) = conditions.get("regulations") {
                    if let Some(reg_array) = regulations.as_array() {
                        matches = matches || reg_array.iter().any(|r| {
                            r.as_str().map(|s| profile.regulatory_requirements.contains(&s.to_string())).unwrap_or(false)
                        });
                    }
                }
                
                // Check country (string)
                if let Some(country) = conditions.get("country") {
                    if let Some(country_str) = country.as_str() {
                        matches = matches || profile.country == country_str;
                    }
                }
                
                matches
            };

            if should_enable {
                // Enable module for company
                sqlx::query(
                    r#"
                    INSERT INTO company_module_configs (company_id, module_id, enabled)
                    SELECT $1, id, true
                    FROM modules
                    WHERE name = $2
                    ON CONFLICT (company_id, module_id)
                    DO UPDATE SET enabled = true, updated_at = CURRENT_TIMESTAMP
                    "#,
                )
                .bind(company_id)
                .bind(&module_name)
                .execute(&self.pool)
                .await?;

                to_enable.push(module_name);
            }
        }

        Ok(to_enable)
    }

    /// Apply policy templates to a company based on their profile
    pub async fn apply_policy_templates(
        &self,
        company_id: Uuid,
        profile: &CompanyProfile,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        use sqlx::Row;
        use uuid::Uuid as UuidType;

        // Get relevant policy templates
        let rows = sqlx::query(
            r#"
            SELECT pt.id, pt.name, pt.regulation, pt.policy_config, pt.template_type
            FROM policy_templates pt
            WHERE (pt.industry IS NULL OR pt.industry = $1)
            AND pt.regulation = ANY($2)
            ORDER BY 
                CASE pt.template_type
                    WHEN 'STRICT' THEN 1
                    WHEN 'DEFAULT' THEN 2
                    WHEN 'LENIENT' THEN 3
                END
            "#,
        )
        .bind(&profile.industry)
        .bind(&profile.regulatory_requirements)
        .fetch_all(&self.pool)
        .await?;

        let mut applied_policies = Vec::new();

        for row in rows {
            let template_id: UuidType = row.get("id");
            let policy_config: serde_json::Value = row.get("policy_config");
            let regulation: String = row.get("regulation");
            let template_name: String = row.get("name");

            // Create policy from template
            // Note: This assumes you have a policies table - adjust based on your schema
            let policy_id_result = sqlx::query_scalar::<_, UuidType>(
                r#"
                INSERT INTO policy_versions (
                    policy_type, policy_config, is_active, created_by, company_id
                )
                VALUES ($1, $2, true, NULL, $3)
                RETURNING id
                "#,
            )
            .bind(format!("{}_POLICY", regulation))
            .bind(policy_config)
            .bind(company_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some(policy_id) = policy_id_result {
                applied_policies.push(policy_id);
            }
        }

        Ok(applied_policies)
    }

    /// Configure modules for a company
    pub async fn configure_modules(
        &self,
        company_id: Uuid,
        module_configs: Vec<(String, serde_json::Value)>,
        configured_by: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        for (module_name, config) in module_configs {
            sqlx::query(
                r#"
                INSERT INTO company_module_configs (company_id, module_id, enabled, configuration, configured_by)
                SELECT $1, id, true, $3, $4
                FROM modules
                WHERE name = $2
                ON CONFLICT (company_id, module_id)
                DO UPDATE SET
                    configuration = EXCLUDED.configuration,
                    enabled = true,
                    configured_by = EXCLUDED.configured_by,
                    configured_at = CURRENT_TIMESTAMP,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(company_id)
            .bind(&module_name)
            .bind(config)
            .bind(configured_by)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get policy templates for a regulation
    pub async fn get_policy_templates(
        &self,
        regulation: Option<&str>,
        industry: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        use sqlx::Row;

        let query = if let Some(reg) = regulation {
            if let Some(ind) = industry {
                sqlx::query(
                    r#"
                    SELECT id, name, regulation, article_number, template_type, 
                           policy_config, description, use_cases, industry
                    FROM policy_templates
                    WHERE regulation = $1 AND (industry IS NULL OR industry = $2)
                    ORDER BY template_type, name
                    "#,
                )
                .bind(reg)
                .bind(ind)
            } else {
                sqlx::query(
                    r#"
                    SELECT id, name, regulation, article_number, template_type, 
                           policy_config, description, use_cases, industry
                    FROM policy_templates
                    WHERE regulation = $1
                    ORDER BY template_type, name
                    "#,
                )
                .bind(reg)
            }
        } else {
            sqlx::query(
                r#"
                SELECT id, name, regulation, article_number, template_type, 
                       policy_config, description, use_cases, industry
                FROM policy_templates
                ORDER BY regulation, template_type, name
                "#,
            )
        };

        let rows = query.fetch_all(&self.pool).await?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(serde_json::json!({
                "id": row.get::<uuid::Uuid, _>("id"),
                "name": row.get::<String, _>("name"),
                "regulation": row.get::<String, _>("regulation"),
                "article_number": row.get::<Option<String>, _>("article_number"),
                "template_type": row.get::<String, _>("template_type"),
                "policy_config": row.get::<serde_json::Value, _>("policy_config"),
                "description": row.get::<Option<String>, _>("description"),
                "use_cases": row.get::<Option<Vec<String>>, _>("use_cases"),
                "industry": row.get::<Option<String>, _>("industry"),
            }));
        }

        Ok(templates)
    }
}

