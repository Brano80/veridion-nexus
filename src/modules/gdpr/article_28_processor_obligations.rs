// GDPR Article 28 - Processor Obligations Module
// Implements requirements for data processors and Data Processing Agreements (DPAs)

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorObligationsConfig {
    pub require_dpa: bool,
    pub dpa_template: String, // 'standard', 'custom'
    pub processor_audit_frequency: String, // 'monthly', 'quarterly', 'annually'
    pub require_processor_registry: bool,
    pub auto_notify_on_processor_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingAgreement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub processor_name: String,
    pub processor_contact: String,
    pub processing_purposes: Vec<String>,
    pub data_categories: Vec<String>,
    pub data_subject_categories: Vec<String>,
    pub security_measures: Vec<String>,
    pub dpa_document_url: Option<String>,
    pub signed_date: Option<DateTime<Utc>>,
    pub expires_date: Option<DateTime<Utc>>,
    pub status: String, // 'draft', 'signed', 'expired', 'terminated'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct GDPRArticle28Module {
    pool: PgPool,
}

impl GDPRArticle28Module {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if module is enabled for a company
    pub async fn is_enabled(&self, company_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<bool> = sqlx::query_scalar(
            "SELECT is_module_enabled_for_company($1, 'gdpr_article_28')",
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
    ) -> Result<Option<ProcessorObligationsConfig>, sqlx::Error> {
        let config_json: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT get_company_module_config($1, 'gdpr_article_28')",
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

    /// Create or update a Data Processing Agreement
    pub async fn create_or_update_dpa(
        &self,
        company_id: Uuid,
        dpa: DataProcessingAgreement,
    ) -> Result<Uuid, sqlx::Error> {
        // Check if module is enabled
        if !self.is_enabled(company_id).await? {
            return Err(sqlx::Error::RowNotFound);
        }

        let dpa_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO data_processing_agreements (
                company_id, processor_name, processor_contact,
                processing_purposes, data_categories, data_subject_categories,
                security_measures, dpa_document_url, signed_date, expires_date, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (company_id, processor_name) DO UPDATE SET
                processor_contact = EXCLUDED.processor_contact,
                processing_purposes = EXCLUDED.processing_purposes,
                data_categories = EXCLUDED.data_categories,
                data_subject_categories = EXCLUDED.data_subject_categories,
                security_measures = EXCLUDED.security_measures,
                dpa_document_url = EXCLUDED.dpa_document_url,
                signed_date = EXCLUDED.signed_date,
                expires_date = EXCLUDED.expires_date,
                status = EXCLUDED.status,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
        )
        .bind(company_id)
        .bind(&dpa.processor_name)
        .bind(&dpa.processor_contact)
        .bind(&dpa.processing_purposes)
        .bind(&dpa.data_categories)
        .bind(&dpa.data_subject_categories)
        .bind(&dpa.security_measures)
        .bind(&dpa.dpa_document_url)
        .bind(dpa.signed_date)
        .bind(dpa.expires_date)
        .bind(&dpa.status)
        .fetch_one(&self.pool)
        .await?;

        Ok(dpa_id)
    }

    /// Get all DPAs for a company
    pub async fn get_dpas(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<DataProcessingAgreement>, sqlx::Error> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, company_id, processor_name, processor_contact,
                   processing_purposes, data_categories, data_subject_categories,
                   security_measures, dpa_document_url, signed_date, expires_date,
                   status, created_at, updated_at
            FROM data_processing_agreements
            WHERE company_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(&self.pool)
        .await?;

        let mut dpas = Vec::new();
        for row in rows {
            dpas.push(DataProcessingAgreement {
                id: row.get("id"),
                company_id: row.get("company_id"),
                processor_name: row.get("processor_name"),
                processor_contact: row.get("processor_contact"),
                processing_purposes: row.get("processing_purposes"),
                data_categories: row.get("data_categories"),
                data_subject_categories: row.get("data_subject_categories"),
                security_measures: row.get("security_measures"),
                dpa_document_url: row.get("dpa_document_url"),
                signed_date: row.get("signed_date"),
                expires_date: row.get("expires_date"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(dpas)
    }

    /// Check if a processor requires a DPA
    pub async fn requires_dpa(
        &self,
        company_id: Uuid,
        processor_name: &str,
    ) -> Result<bool, sqlx::Error> {
        // Check if module is enabled and configured to require DPA
        let config = self.get_config(company_id).await?;
        
        if let Some(cfg) = config {
            if !cfg.require_dpa {
                return Ok(false);
            }
        }

        // Check if DPA already exists
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM data_processing_agreements
                WHERE company_id = $1
                AND processor_name = $2
                AND status IN ('signed', 'draft')
            )
            "#,
        )
        .bind(company_id)
        .bind(processor_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(!exists)
    }

    /// Get configuration schema for this module
    pub fn get_config_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "require_dpa": {
                    "type": "boolean",
                    "default": true,
                    "description": "Require Data Processing Agreement for all processors"
                },
                "dpa_template": {
                    "type": "string",
                    "enum": ["standard", "custom"],
                    "default": "standard",
                    "description": "DPA template to use"
                },
                "processor_audit_frequency": {
                    "type": "string",
                    "enum": ["monthly", "quarterly", "annually"],
                    "default": "quarterly",
                    "description": "How often to audit processors"
                },
                "require_processor_registry": {
                    "type": "boolean",
                    "default": true,
                    "description": "Maintain registry of all processors"
                },
                "auto_notify_on_processor_change": {
                    "type": "boolean",
                    "default": true,
                    "description": "Automatically notify on processor changes"
                }
            },
            "required": ["require_dpa", "dpa_template", "processor_audit_frequency"]
        })
    }
}

