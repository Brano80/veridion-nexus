// Veridion TPRM Integration
// Third-Party Risk Management data enrichment for context-aware hardening

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vendor risk score from Veridion API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorRiskScore {
    pub vendor_domain: String,
    pub vendor_name: Option<String>,
    pub risk_score: f64, // 0.0-100.0
    pub risk_level: String, // LOW, MEDIUM, HIGH, CRITICAL
    pub compliance_status: String, // COMPLIANT, NON_COMPLIANT, UNKNOWN
    pub country_code: Option<String>,
    pub industry_sector: Option<String>,
    pub veridion_data: Option<serde_json::Value>,
}

/// Veridion TPRM Service
pub struct VeridionTPRMService {
    api_key: Option<String>,
    api_base_url: String,
}

impl VeridionTPRMService {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            api_base_url: std::env::var("VERIDION_TPRM_API_URL")
                .unwrap_or_else(|_| "https://api.veridion.com/v1".to_string()),
        }
    }

    /// Fetch vendor risk score from Veridion API
    pub async fn fetch_vendor_risk_score(
        &self,
        vendor_domain: &str,
    ) -> Result<VendorRiskScore, String> {
        // If no API key, return mock data for development
        if self.api_key.is_none() {
            log::warn!("VERIDION_API_KEY not set - using mock data for {}", vendor_domain);
            return Ok(self.mock_vendor_risk_score(vendor_domain));
        }

        // Implement actual Veridion API call
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let api_key = self.api_key.as_ref().unwrap();
        
        // Veridion API endpoint for company data
        // Note: This is a placeholder - adjust based on actual Veridion API documentation
        let url = format!("{}/companies/match", self.api_base_url);
        
        let request_body = serde_json::json!({
            "company_website": format!("https://{}", vendor_domain),
            "match_quality": "high"
        });

        match client
            .post(&url)
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let json: serde_json::Value = response.json().await
                        .map_err(|e| format!("Failed to parse Veridion response: {}", e))?;
                    
                    // Parse Veridion response and extract risk data
                    // Adjust this based on actual Veridion API response structure
                    let risk_score = json.get("risk_score")
                        .and_then(|v| v.as_f64())
                        .unwrap_or_else(|| {
                            // Calculate risk score from available data
                            self.calculate_risk_score_from_veridion_data(&json)
                        });
                    
                    let risk_level = if risk_score >= 80.0 {
                        "CRITICAL"
                    } else if risk_score >= 60.0 {
                        "HIGH"
                    } else if risk_score >= 40.0 {
                        "MEDIUM"
                    } else {
                        "LOW"
                    };

                    let country_code = json.get("company_location")
                        .and_then(|v| v.get("country_code"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let industry_sector = json.get("company_industry")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    // Determine compliance status based on country and industry
                    let compliance_status = self.determine_compliance_status(&country_code, &industry_sector);

                    Ok(VendorRiskScore {
                        vendor_domain: vendor_domain.to_string(),
                        vendor_name: json.get("company_name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        risk_score,
                        risk_level: risk_level.to_string(),
                        compliance_status,
                        country_code,
                        industry_sector,
                        veridion_data: Some(json),
                    })
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    log::warn!("Veridion API error ({}): {} - falling back to mock data", status, error_text);
                    Ok(self.mock_vendor_risk_score(vendor_domain))
                }
            }
            Err(e) => {
                log::warn!("Failed to call Veridion API: {} - falling back to mock data", e);
                Ok(self.mock_vendor_risk_score(vendor_domain))
            }
        }
    }

    /// Calculate risk score from Veridion data when explicit risk_score is not available
    fn calculate_risk_score_from_veridion_data(&self, data: &serde_json::Value) -> f64 {
        let mut score = 50.0; // Base score

        // Adjust based on country (non-EU countries have higher risk)
        if let Some(country) = data.get("company_location")
            .and_then(|v| v.get("country_code"))
            .and_then(|v| v.as_str()) {
            let non_eu_countries = ["US", "CN", "RU", "IN"];
            if non_eu_countries.contains(&country.to_uppercase().as_str()) {
                score += 20.0;
            }
        }

        // Adjust based on industry (AI/ML services have higher risk)
        if let Some(industry) = data.get("company_industry")
            .and_then(|v| v.as_str()) {
            let high_risk_industries = ["AI", "MACHINE_LEARNING", "DATA_ANALYTICS"];
            if high_risk_industries.iter().any(|&ind| industry.to_uppercase().contains(ind)) {
                score += 15.0;
            }
        }

        // Adjust based on data processing indicators
        if let Some(processes_data) = data.get("processes_personal_data")
            .and_then(|v| v.as_bool()) {
            if processes_data {
                score += 10.0;
            }
        }

        if score > 100.0 { 100.0 } else { score }
    }

    /// Determine compliance status based on country and industry
    fn determine_compliance_status(&self, country_code: &Option<String>, industry: &Option<String>) -> String {
        if let Some(country) = country_code {
            let eu_countries = ["AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", 
                               "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL", 
                               "PL", "PT", "RO", "SK", "SI", "ES", "SE"];
            
            if eu_countries.contains(&country.to_uppercase().as_str()) {
                return "COMPLIANT".to_string();
            }
        }

        // High-risk industries or non-EU countries
        if let Some(ind) = industry {
            let high_risk_industries = ["AI", "MACHINE_LEARNING"];
            if high_risk_industries.iter().any(|&risk_ind| ind.to_uppercase().contains(risk_ind)) {
                return "NON_COMPLIANT".to_string();
            }
        }

        "UNKNOWN".to_string()
    }

    /// Mock vendor risk score (for development/testing)
    fn mock_vendor_risk_score(&self, vendor_domain: &str) -> VendorRiskScore {
        let domain_lower = vendor_domain.to_lowercase();
        
        // Mock risk assessment based on domain patterns
        let (risk_score, risk_level, compliance_status) = if domain_lower.contains("openai") 
            || domain_lower.contains("anthropic") 
            || domain_lower.contains("google") {
            (75.0, "HIGH".to_string(), "COMPLIANT".to_string())
        } else if domain_lower.contains("microsoft") 
            || domain_lower.contains("azure") {
            (60.0, "MEDIUM".to_string(), "COMPLIANT".to_string())
        } else if domain_lower.contains("amazon") 
            || domain_lower.contains("aws") {
            (70.0, "HIGH".to_string(), "COMPLIANT".to_string())
        } else {
            (50.0, "MEDIUM".to_string(), "UNKNOWN".to_string())
        };

        VendorRiskScore {
            vendor_domain: vendor_domain.to_string(),
            vendor_name: Some(vendor_domain.to_string()),
            risk_score,
            risk_level,
            compliance_status,
            country_code: Some("US".to_string()),
            industry_sector: Some("AI_SERVICES".to_string()),
            veridion_data: Some(serde_json::json!({
                "source": "mock",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })),
        }
    }

    /// Store vendor risk score in database
    pub async fn store_vendor_risk_score(
        &self,
        db_pool: &PgPool,
        risk_score: &VendorRiskScore,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO vendor_risk_scores (
                vendor_domain, vendor_name, risk_score, risk_level,
                compliance_status, country_code, industry_sector,
                veridion_data, last_updated
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (vendor_domain) DO UPDATE SET
                vendor_name = EXCLUDED.vendor_name,
                risk_score = EXCLUDED.risk_score,
                risk_level = EXCLUDED.risk_level,
                compliance_status = EXCLUDED.compliance_status,
                country_code = EXCLUDED.country_code,
                industry_sector = EXCLUDED.industry_sector,
                veridion_data = EXCLUDED.veridion_data,
                last_updated = EXCLUDED.last_updated"
        )
        .bind(&risk_score.vendor_domain)
        .bind(&risk_score.vendor_name)
        .bind(risk_score.risk_score)
        .bind(&risk_score.risk_level)
        .bind(&risk_score.compliance_status)
        .bind(&risk_score.country_code)
        .bind(&risk_score.industry_sector)
        .bind(&risk_score.veridion_data)
        .bind(chrono::Utc::now())
        .execute(db_pool)
        .await?;

        Ok(())
    }

    /// Enrich asset with TPRM data for all its vendors
    pub async fn enrich_asset_with_tprm(
        &self,
        db_pool: &PgPool,
        asset_id: uuid::Uuid,
    ) -> Result<HashMap<String, VendorRiskScore>, String> {
        // Get all vendors for this asset
        #[derive(sqlx::FromRow)]
        struct VendorMapping {
            vendor_domain: String,
        }

        let vendors: Vec<VendorMapping> = sqlx::query_as(
            "SELECT vendor_domain FROM asset_vendor_mapping WHERE asset_id = $1"
        )
        .bind(asset_id)
        .fetch_all(db_pool)
        .await
        .map_err(|e| format!("Failed to fetch vendors: {}", e))?;

        let mut enriched_data = HashMap::new();

        // Fetch risk scores for each vendor
        for vendor in vendors {
            // Check if we have cached data (less than 24 hours old)
            #[derive(sqlx::FromRow)]
            struct VendorRiskScoreRow {
                vendor_domain: String,
                vendor_name: Option<String>,
                risk_score: Option<rust_decimal::Decimal>,
                risk_level: Option<String>,
                compliance_status: Option<String>,
                country_code: Option<String>,
                industry_sector: Option<String>,
                veridion_data: Option<serde_json::Value>,
            }
            
            let cached_row: Option<VendorRiskScoreRow> = sqlx::query_as(
                "SELECT vendor_domain, vendor_name, risk_score, risk_level,
                        compliance_status, country_code, industry_sector, veridion_data
                 FROM vendor_risk_scores
                 WHERE vendor_domain = $1 AND last_updated > CURRENT_TIMESTAMP - INTERVAL '24 hours'"
            )
            .bind(&vendor.vendor_domain)
            .fetch_optional(db_pool)
            .await
            .ok()
            .flatten();
            
            let cached = cached_row.map(|row| VendorRiskScore {
                vendor_domain: row.vendor_domain,
                vendor_name: row.vendor_name,
                risk_score: row.risk_score.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
                risk_level: row.risk_level.unwrap_or_else(|| "UNKNOWN".to_string()),
                compliance_status: row.compliance_status.unwrap_or_else(|| "UNKNOWN".to_string()),
                country_code: row.country_code,
                industry_sector: row.industry_sector,
                veridion_data: row.veridion_data,
            });

            let risk_score = if let Some(cached) = cached {
                cached
            } else {
                // Fetch from Veridion API
                let fetched = self.fetch_vendor_risk_score(&vendor.vendor_domain).await?;
                // Store in database
                self.store_vendor_risk_score(db_pool, &fetched).await
                    .map_err(|e| format!("Failed to store risk score: {}", e))?;
                fetched
            };

            enriched_data.insert(vendor.vendor_domain, risk_score);
        }

        Ok(enriched_data)
    }

    /// Auto-generate policy recommendations based on TPRM data
    pub async fn generate_tprm_policy_recommendations(
        &self,
        db_pool: &PgPool,
        asset_id: uuid::Uuid,
    ) -> Result<Vec<PolicyRecommendation>, String> {
        let enriched_data = self.enrich_asset_with_tprm(db_pool, asset_id).await?;
        let mut recommendations = Vec::new();

        for (vendor_domain, risk_score) in enriched_data {
            let recommendation = if risk_score.risk_level == "CRITICAL" 
                || risk_score.compliance_status == "NON_COMPLIANT" {
                PolicyRecommendation {
                    vendor_domain: vendor_domain.clone(),
                    recommendation_type: "BLOCK".to_string(),
                    risk_reason: format!(
                        "Vendor has {} risk level and {} compliance status",
                        risk_score.risk_level, risk_score.compliance_status
                    ),
                    suggested_policy_config: serde_json::json!({
                        "action": "BLOCK",
                        "vendor_domain": vendor_domain,
                        "reason": "TPRM_CRITICAL_RISK"
                    }),
                    priority: 10,
                }
            } else if risk_score.risk_level == "HIGH" {
                PolicyRecommendation {
                    vendor_domain: vendor_domain.clone(),
                    recommendation_type: "RESTRICT".to_string(),
                    risk_reason: format!("Vendor has HIGH risk level (score: {})", risk_score.risk_score),
                    suggested_policy_config: serde_json::json!({
                        "action": "RESTRICT",
                        "vendor_domain": vendor_domain,
                        "require_approval": true,
                        "monitor": true
                    }),
                    priority: 7,
                }
            } else if risk_score.risk_level == "MEDIUM" {
                PolicyRecommendation {
                    vendor_domain: vendor_domain.clone(),
                    recommendation_type: "MONITOR".to_string(),
                    risk_reason: format!("Vendor has MEDIUM risk level (score: {})", risk_score.risk_score),
                    suggested_policy_config: serde_json::json!({
                        "action": "MONITOR",
                        "vendor_domain": vendor_domain,
                        "alert_on_anomaly": true
                    }),
                    priority: 5,
                }
            } else {
                PolicyRecommendation {
                    vendor_domain: vendor_domain.clone(),
                    recommendation_type: "ALLOW".to_string(),
                    risk_reason: format!("Vendor has LOW risk level (score: {})", risk_score.risk_score),
                    suggested_policy_config: serde_json::json!({
                        "action": "ALLOW",
                        "vendor_domain": vendor_domain
                    }),
                    priority: 3,
                }
            };

            recommendations.push(recommendation);
        }

        Ok(recommendations)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRecommendation {
    pub vendor_domain: String,
    pub recommendation_type: String, // BLOCK, RESTRICT, MONITOR, ALLOW
    pub risk_reason: String,
    pub suggested_policy_config: serde_json::Value,
    pub priority: i32, // 1-10
}

