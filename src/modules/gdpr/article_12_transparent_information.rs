// GDPR Article 12 - Transparent Information Module
// Implements requirements for transparent information, communication, and privacy notices
// GDPR Article 12: Transparent information, communication and modalities for the exercise of the rights of the data subject

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentInformationConfig {
    pub require_multi_language: bool,
    pub supported_languages: Vec<String>, // ISO 639-1 codes: 'en', 'de', 'fr', etc.
    pub auto_update_on_regulation_change: bool,
    pub notification_required_languages: Vec<String>, // Languages that must have notifications
    pub default_language: String, // Default language code
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyNotice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub language_code: String, // ISO 639-1 code
    pub notice_type: String, // 'privacy_policy', 'cookie_policy', 'terms_of_service', 'data_protection_notice'
    pub content: String,
    pub version: i32,
    pub published_at: Option<DateTime<Utc>>,
    pub status: String, // 'draft', 'published', 'archived'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrivacyNoticeRequest {
    pub language_code: String,
    pub notice_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePrivacyNoticeRequest {
    pub content: Option<String>,
    pub status: Option<String>,
}

// EU official languages (24 languages)
pub const EU_LANGUAGES: &[&str] = &[
    "bg", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "de", "el", "hu",
    "ga", "it", "lv", "lt", "mt", "pl", "pt", "ro", "sk", "sl", "es", "sv",
];

pub struct GDPRArticle12Module {
    pool: PgPool,
}

impl GDPRArticle12Module {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if module is enabled for a company
    pub async fn is_enabled(&self, company_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<bool> = sqlx::query_scalar(
            "SELECT is_module_enabled_for_company($1, 'gdpr_article_12')",
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
    ) -> Result<Option<TransparentInformationConfig>, sqlx::Error> {
        let config_json: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT get_company_module_config($1, 'gdpr_article_12')",
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

    /// Get configuration schema for dynamic UI generation
    pub fn get_config_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "require_multi_language": {
                    "type": "boolean",
                    "default": true,
                    "description": "Require privacy notices in multiple languages"
                },
                "supported_languages": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "default": ["en", "de", "fr"],
                    "description": "List of supported language codes (ISO 639-1)"
                },
                "auto_update_on_regulation_change": {
                    "type": "boolean",
                    "default": true,
                    "description": "Automatically update notices when regulations change"
                },
                "notification_required_languages": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "default": ["en"],
                    "description": "Languages that must have notification support"
                },
                "default_language": {
                    "type": "string",
                    "default": "en",
                    "description": "Default language code"
                }
            },
            "required": ["require_multi_language", "supported_languages", "default_language"]
        })
    }

    /// Create a privacy notice
    pub async fn create_privacy_notice(
        &self,
        company_id: Uuid,
        request: CreatePrivacyNoticeRequest,
    ) -> Result<PrivacyNotice, sqlx::Error> {
        // Check if module is enabled
        if !self.is_enabled(company_id).await? {
            return Err(sqlx::Error::RowNotFound);
        }

        let notice_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO privacy_notices (
                id, company_id, language_code, notice_type, content, version, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 1, 'draft', $6, $6)
            "#,
        )
        .bind(notice_id)
        .bind(company_id)
        .bind(&request.language_code)
        .bind(&request.notice_type)
        .bind(&request.content)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_privacy_notice(company_id, notice_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// Get a privacy notice by ID
    pub async fn get_privacy_notice(
        &self,
        company_id: Uuid,
        notice_id: Uuid,
    ) -> Result<Option<PrivacyNotice>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, company_id, language_code, notice_type, content, version,
                   published_at, status, created_at, updated_at
            FROM privacy_notices
            WHERE id = $1 AND company_id = $2
            "#,
        )
        .bind(notice_id)
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(PrivacyNotice {
                id: row.get("id"),
                company_id: row.get("company_id"),
                language_code: row.get("language_code"),
                notice_type: row.get("notice_type"),
                content: row.get("content"),
                version: row.get("version"),
                published_at: row.get("published_at"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// List all privacy notices for a company
    pub async fn list_privacy_notices(
        &self,
        company_id: Uuid,
        notice_type: Option<&str>,
        language_code: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<PrivacyNotice>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT id, company_id, language_code, notice_type, content, version,
                   published_at, status, created_at, updated_at
            FROM privacy_notices
            WHERE company_id = $1
            "#,
        );

        let mut param_count = 1;
        let mut params: Vec<String> = vec![company_id.to_string()];

        if let Some(nt) = notice_type {
            param_count += 1;
            query.push_str(&format!(" AND notice_type = ${}", param_count));
            params.push(nt.to_string());
        }

        if let Some(lc) = language_code {
            param_count += 1;
            query.push_str(&format!(" AND language_code = ${}", param_count));
            params.push(lc.to_string());
        }

        if let Some(st) = status {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
            params.push(st.to_string());
        }

        query.push_str(" ORDER BY created_at DESC");

        // For simplicity, using a simpler approach
        let rows = if let Some(nt) = notice_type {
            sqlx::query(
                r#"
                SELECT id, company_id, language_code, notice_type, content, version,
                       published_at, status, created_at, updated_at
                FROM privacy_notices
                WHERE company_id = $1 AND notice_type = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(company_id)
            .bind(nt)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, company_id, language_code, notice_type, content, version,
                       published_at, status, created_at, updated_at
                FROM privacy_notices
                WHERE company_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(company_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows
            .into_iter()
            .map(|row| PrivacyNotice {
                id: row.get("id"),
                company_id: row.get("company_id"),
                language_code: row.get("language_code"),
                notice_type: row.get("notice_type"),
                content: row.get("content"),
                version: row.get("version"),
                published_at: row.get("published_at"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    /// Update a privacy notice
    pub async fn update_privacy_notice(
        &self,
        company_id: Uuid,
        notice_id: Uuid,
        request: UpdatePrivacyNoticeRequest,
    ) -> Result<PrivacyNotice, sqlx::Error> {
        // Check if module is enabled
        if !self.is_enabled(company_id).await? {
            return Err(sqlx::Error::RowNotFound);
        }

        let now = Utc::now();

        if let Some(content) = &request.content {
            sqlx::query(
                r#"
                UPDATE privacy_notices
                SET content = $1, updated_at = $2, version = version + 1
                WHERE id = $3 AND company_id = $4
                "#,
            )
            .bind(content)
            .bind(now)
            .bind(notice_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;
        }

        if let Some(status) = &request.status {
            sqlx::query(
                r#"
                UPDATE privacy_notices
                SET status = $1, updated_at = $2
                WHERE id = $3 AND company_id = $4
                "#,
            )
            .bind(status)
            .bind(now)
            .bind(notice_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;
        }

        self.get_privacy_notice(company_id, notice_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// Publish a privacy notice
    pub async fn publish_privacy_notice(
        &self,
        company_id: Uuid,
        notice_id: Uuid,
    ) -> Result<PrivacyNotice, sqlx::Error> {
        // Check if module is enabled
        if !self.is_enabled(company_id).await? {
            return Err(sqlx::Error::RowNotFound);
        }

        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE privacy_notices
            SET status = 'published', published_at = $1, updated_at = $1
            WHERE id = $2 AND company_id = $3
            "#,
        )
        .bind(now)
        .bind(notice_id)
        .bind(company_id)
        .execute(&self.pool)
        .await?;

        self.get_privacy_notice(company_id, notice_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    /// Get privacy notice templates
    pub fn get_templates() -> serde_json::Value {
        serde_json::json!({
            "privacy_policy": {
                "en": "Privacy Policy Template (English)",
                "de": "Datenschutzerklärung Vorlage (Deutsch)",
                "fr": "Modèle de Politique de Confidentialité (Français)"
            },
            "cookie_policy": {
                "en": "Cookie Policy Template (English)",
                "de": "Cookie-Richtlinie Vorlage (Deutsch)",
                "fr": "Modèle de Politique de Cookies (Français)"
            },
            "data_protection_notice": {
                "en": "Data Protection Notice Template (English)",
                "de": "Datenschutzhinweis Vorlage (Deutsch)",
                "fr": "Modèle d'Avis de Protection des Données (Français)"
            }
        })
    }

    /// Validate language code
    pub fn is_valid_language_code(code: &str) -> bool {
        EU_LANGUAGES.contains(&code.to_lowercase().as_str())
    }
}

