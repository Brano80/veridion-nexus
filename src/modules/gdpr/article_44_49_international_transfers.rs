// GDPR Article 44-49 - International Transfers Module
// Implements requirements for transferring personal data outside EU/EEA

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalTransfersConfig {
    pub block_non_eu_transfers: bool,
    pub require_sccs: bool,
    pub scc_type: String, // 'controller_to_controller', 'controller_to_processor', 'processor_to_processor'
    pub adequacy_decision_check: bool,
    pub require_explicit_consent: bool,
    pub transfer_impact_assessment_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransfer {
    pub id: Uuid,
    pub company_id: Uuid,
    pub destination_country: String,
    pub destination_entity: String,
    pub transfer_purpose: String,
    pub data_categories: Vec<String>,
    pub legal_basis: String, // 'adequacy_decision', 'sccs', 'binding_corporate_rules', 'explicit_consent'
    pub scc_document_url: Option<String>,
    pub adequacy_decision_applies: bool,
    pub transfer_impact_assessment_url: Option<String>,
    pub approved: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String, // 'pending', 'approved', 'rejected', 'expired'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// EU/EEA countries (whitelist)
const EU_EEA_COUNTRIES: &[&str] = &[
    "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR", "DE", "GR",
    "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL", "PL", "PT", "RO", "SK",
    "SI", "ES", "SE", "IS", "LI", "NO",
];

// Countries with adequacy decisions
const ADEQUACY_DECISION_COUNTRIES: &[&str] = &[
    "AD", "AR", "CA", "FO", "GG", "IL", "JP", "JE", "NZ", "KR", "GB", "UY",
];

pub struct GDPRArticle4449Module {
    pool: PgPool,
}

impl GDPRArticle4449Module {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if module is enabled for a company
    pub async fn is_enabled(&self, company_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<bool> = sqlx::query_scalar(
            "SELECT is_module_enabled_for_company($1, 'gdpr_article_44_49')",
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Get module configuration for a company
    pub async fn get_config(
        &self,
        company_id: Uuid,
    ) -> Result<Option<InternationalTransfersConfig>, sqlx::Error> {
        let config_json: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT get_company_module_config($1, 'gdpr_article_44_49')",
        )
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(json) = config_json {
            serde_json::from_value(json)
                .map(Some)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))
        } else {
            Ok(None)
        }
    }

    /// Check if a country is in EU/EEA
    pub fn is_eu_eea_country(country_code: &str) -> bool {
        EU_EEA_COUNTRIES.contains(&country_code.to_uppercase().as_str())
    }

    /// Check if a country has an adequacy decision
    pub fn has_adequacy_decision(country_code: &str) -> bool {
        ADEQUACY_DECISION_COUNTRIES.contains(&country_code.to_uppercase().as_str())
    }

    /// Validate if a transfer is allowed
    pub async fn validate_transfer(
        &self,
        company_id: Uuid,
        destination_country: &str,
        legal_basis: &str,
    ) -> Result<(bool, String), sqlx::Error> {
        // Check if module is enabled
        if !self.is_enabled(company_id).await? {
            return Ok((true, "Module not enabled".to_string()));
        }

        let config = self.get_config(company_id).await?;

        // If blocking is enabled and country is not EU/EEA
        if let Some(cfg) = &config {
            if cfg.block_non_eu_transfers && !Self::is_eu_eea_country(destination_country) {
                // Check if adequacy decision applies
                if Self::has_adequacy_decision(destination_country) {
                    return Ok((true, "Adequacy decision applies".to_string()));
                }

                // Check if SCCs are required and provided
                if cfg.require_sccs {
                    if legal_basis != "sccs" {
                        return Ok((false, "SCCs required for transfers to non-EU countries".to_string()));
                    }
                }

                // Check if explicit consent is required
                if cfg.require_explicit_consent && legal_basis != "explicit_consent" {
                    return Ok((false, "Explicit consent required for transfers to non-EU countries".to_string()));
                }
            }
        }

        Ok((true, "Transfer allowed".to_string()))
    }

    /// Register a data transfer
    pub async fn register_transfer(
        &self,
        company_id: Uuid,
        transfer: DataTransfer,
    ) -> Result<Uuid, sqlx::Error> {
        // Validate transfer first
        let (allowed, reason) = self.validate_transfer(
            company_id,
            &transfer.destination_country,
            &transfer.legal_basis,
        ).await?;

        if !allowed {
            return Err(sqlx::Error::RowNotFound);
        }

        let transfer_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO data_transfers (
                company_id, destination_country, destination_entity,
                transfer_purpose, data_categories, legal_basis,
                scc_document_url, adequacy_decision_applies,
                transfer_impact_assessment_url, approved, approved_by,
                approved_at, expires_at, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#,
        )
        .bind(company_id)
        .bind(&transfer.destination_country)
        .bind(&transfer.destination_entity)
        .bind(&transfer.transfer_purpose)
        .bind(&transfer.data_categories)
        .bind(&transfer.legal_basis)
        .bind(&transfer.scc_document_url)
        .bind(transfer.adequacy_decision_applies)
        .bind(&transfer.transfer_impact_assessment_url)
        .bind(transfer.approved)
        .bind(transfer.approved_by)
        .bind(transfer.approved_at)
        .bind(transfer.expires_at)
        .bind(&transfer.status)
        .fetch_one(&self.pool)
        .await?;

        Ok(transfer_id)
    }

    /// Get all transfers for a company
    pub async fn get_transfers(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<DataTransfer>, sqlx::Error> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, company_id, destination_country, destination_entity,
                   transfer_purpose, data_categories, legal_basis,
                   scc_document_url, adequacy_decision_applies,
                   transfer_impact_assessment_url, approved, approved_by,
                   approved_at, expires_at, status, created_at, updated_at
            FROM data_transfers
            WHERE company_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut transfers = Vec::new();
        for row in rows {
            transfers.push(DataTransfer {
                id: row.get("id"),
                company_id: row.get("company_id"),
                destination_country: row.get("destination_country"),
                destination_entity: row.get("destination_entity"),
                transfer_purpose: row.get("transfer_purpose"),
                data_categories: row.get("data_categories"),
                legal_basis: row.get("legal_basis"),
                scc_document_url: row.get("scc_document_url"),
                adequacy_decision_applies: row.get("adequacy_decision_applies"),
                transfer_impact_assessment_url: row.get("transfer_impact_assessment_url"),
                approved: row.get("approved"),
                approved_by: row.get("approved_by"),
                approved_at: row.get("approved_at"),
                expires_at: row.get("expires_at"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(transfers)
    }

    /// Check if there is an approved transfer for a destination country
    /// This is used by the proxy to allow requests if DPA/SCCs are in place
    pub async fn has_approved_transfer(
        &self,
        company_id: Uuid,
        destination_country: &str,
    ) -> Result<Option<(String, Option<String>)>, sqlx::Error> {
        // Normalize country code to uppercase
        let country_upper = destination_country.to_uppercase();
        
        // Check for approved transfer with valid legal basis (SCCs, adequacy, etc.)
        let result: Option<(String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT legal_basis, scc_document_url
            FROM data_transfers
            WHERE company_id = $1
              AND UPPER(destination_country) = $2
              AND approved = true
              AND status = 'approved'
              AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            ORDER BY approved_at DESC
            LIMIT 1
            "#,
        )
        .bind(company_id)
        .bind(&country_upper)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Check if transfer to country is allowed (for proxy use)
    /// Returns (allowed, reason, requires_dpa_alert)
    pub async fn check_transfer_allowed_for_proxy(
        &self,
        company_id: Uuid,
        destination_country: &str,
    ) -> Result<(bool, String, bool), sqlx::Error> {
        // If EU/EEA, always allowed
        if Self::is_eu_eea_country(destination_country) {
            return Ok((true, "EU/EEA country - transfer allowed".to_string(), false));
        }

        // Check if adequacy decision applies
        if Self::has_adequacy_decision(destination_country) {
            return Ok((true, "Adequacy decision applies".to_string(), false));
        }

        // Check if module is enabled
        if !self.is_enabled(company_id).await? {
            // Module not enabled - allow but warn
            return Ok((true, "Module not enabled - transfer allowed".to_string(), true));
        }

        let config = self.get_config(company_id).await?;

        // If blocking is disabled, allow
        if let Some(cfg) = &config {
            if !cfg.block_non_eu_transfers {
                return Ok((true, "Blocking disabled in configuration".to_string(), true));
            }
        }

        // Check for approved transfer with DPA/SCCs
        if let Some((legal_basis, scc_url)) = self.has_approved_transfer(company_id, destination_country).await? {
            let reason = match legal_basis.as_str() {
                "sccs" => {
                    if scc_url.is_some() {
                        "Approved transfer with SCCs in place".to_string()
                    } else {
                        "Approved transfer with SCCs (document pending)".to_string()
                    }
                }
                "adequacy_decision" => "Adequacy decision applies".to_string(),
                "binding_corporate_rules" => "Approved transfer with Binding Corporate Rules".to_string(),
                "explicit_consent" => "Approved transfer with explicit consent".to_string(),
                _ => format!("Approved transfer with legal basis: {}", legal_basis),
            };
            return Ok((true, reason, false));
        }

        // No approved transfer found - check config to determine if we should block or alert
        if let Some(cfg) = &config {
            if cfg.require_sccs {
                // SCCs required but not found - block with alert
                return Ok((
                    false,
                    "SCCs required for transfers to non-EU countries. Please register a data transfer with DPA/SCCs in the Wizard.".to_string(),
                    true,
                ));
            }
        }

        // Default: block with alert
        Ok((
            false,
            "Transfer to non-EU country requires approved data transfer agreement (DPA/SCCs). Please configure in Settings.".to_string(),
            true,
        ))
    }

    /// Get configuration schema for this module
    pub fn get_config_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "block_non_eu_transfers": {
                    "type": "boolean",
                    "default": true,
                    "description": "Block transfers to non-EU/EEA countries without proper safeguards"
                },
                "require_sccs": {
                    "type": "boolean",
                    "default": true,
                    "description": "Require Standard Contractual Clauses for transfers"
                },
                "scc_type": {
                    "type": "string",
                    "enum": ["controller_to_controller", "controller_to_processor", "processor_to_processor"],
                    "default": "controller_to_processor",
                    "description": "Type of SCCs to use"
                },
                "adequacy_decision_check": {
                    "type": "boolean",
                    "default": true,
                    "description": "Check if adequacy decision applies to destination country"
                },
                "require_explicit_consent": {
                    "type": "boolean",
                    "default": false,
                    "description": "Require explicit consent for transfers"
                },
                "transfer_impact_assessment_required": {
                    "type": "boolean",
                    "default": true,
                    "description": "Require Transfer Impact Assessment for high-risk transfers"
                }
            },
            "required": ["block_non_eu_transfers", "require_sccs", "scc_type"]
        })
    }
}

