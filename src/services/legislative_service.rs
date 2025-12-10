// Legislative Update Service: Tracks regulatory changes and notifies companies

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegislativeUpdate {
    pub id: Uuid,
    pub regulation: String,
    pub article: Option<String>,
    pub update_type: String,
    pub title: String,
    pub description: String,
    pub affected_modules: Vec<String>,
    pub compliance_level: String,
    pub effective_date: NaiveDate,
    pub published_date: NaiveDate,
    pub source_url: Option<String>,
    pub notification_sent: bool,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLegislativeUpdateRequest {
    pub regulation: String,
    pub article: Option<String>,
    pub update_type: String,
    pub title: String,
    pub description: String,
    pub affected_modules: Vec<String>,
    pub compliance_level: String,
    pub effective_date: NaiveDate,
    pub published_date: NaiveDate,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyNotification {
    pub company_id: Uuid,
    pub company_name: String,
    pub update_id: Uuid,
    pub update_title: String,
    pub affected_modules: Vec<String>,
    pub action_required: bool,
}

pub struct LegislativeService {
    pool: PgPool,
}

impl LegislativeService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new legislative update
    pub async fn create_update(
        &self,
        request: CreateLegislativeUpdateRequest,
    ) -> Result<LegislativeUpdate, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO legislative_updates (
                regulation, article, update_type, title, description,
                affected_modules, compliance_level, effective_date,
                published_date, source_url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id, regulation, article, update_type, title, description,
                affected_modules, compliance_level, effective_date,
                published_date, source_url, notification_sent, created_at
            "#
        )
        .bind(&request.regulation)
        .bind(&request.article)
        .bind(&request.update_type)
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.affected_modules)
        .bind(&request.compliance_level)
        .bind(request.effective_date)
        .bind(request.published_date)
        .bind(&request.source_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(LegislativeUpdate {
            id: row.get("id"),
            regulation: row.get("regulation"),
            article: row.get("article"),
            update_type: row.get("update_type"),
            title: row.get("title"),
            description: row.get("description"),
            affected_modules: row.get("affected_modules"),
            compliance_level: row.get("compliance_level"),
            effective_date: row.get("effective_date"),
            published_date: row.get("published_date"),
            source_url: row.get("source_url"),
            notification_sent: row.get("notification_sent"),
            created_at: row.get("created_at"),
        })
    }

    /// Get all legislative updates
    pub async fn get_all_updates(
        &self,
        regulation: Option<&str>,
    ) -> Result<Vec<LegislativeUpdate>, sqlx::Error> {
        let rows = if let Some(reg) = regulation {
            sqlx::query(
                r#"
                SELECT id, regulation, article, update_type, title, description,
                    affected_modules, compliance_level, effective_date,
                    published_date, source_url, notification_sent, created_at
                FROM legislative_updates
                WHERE regulation = $1
                ORDER BY published_date DESC, created_at DESC
                "#
            )
            .bind(reg)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, regulation, article, update_type, title, description,
                    affected_modules, compliance_level, effective_date,
                    published_date, source_url, notification_sent, created_at
                FROM legislative_updates
                ORDER BY published_date DESC, created_at DESC
                "#
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|row| LegislativeUpdate {
            id: row.get("id"),
            regulation: row.get("regulation"),
            article: row.get("article"),
            update_type: row.get("update_type"),
            title: row.get("title"),
            description: row.get("description"),
            affected_modules: row.get("affected_modules"),
            compliance_level: row.get("compliance_level"),
            effective_date: row.get("effective_date"),
            published_date: row.get("published_date"),
            source_url: row.get("source_url"),
            notification_sent: row.get("notification_sent"),
            created_at: row.get("created_at"),
        }).collect())
    }

    /// Get companies that should be notified about a legislative update
    pub async fn get_companies_to_notify(
        &self,
        update_id: Uuid,
    ) -> Result<Vec<CompanyNotification>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                cp.id as company_id,
                cp.company_name,
                lu.id as update_id,
                lu.title as update_title,
                lu.affected_modules,
                CASE 
                    WHEN lu.compliance_level = 'REQUIRED' THEN true
                    ELSE false
                END as action_required
            FROM legislative_updates lu
            CROSS JOIN company_profiles cp
            WHERE lu.id = $1
            AND (
                -- Company has this regulation in their requirements
                lu.regulation = ANY(cp.regulatory_requirements)
                OR
                -- Company uses modules affected by this update
                EXISTS (
                    SELECT 1
                    FROM unnest(lu.affected_modules) AS affected_module
                    WHERE affected_module IN (
                        SELECT m.name
                        FROM modules m
                        JOIN subscription_modules sm ON m.id = sm.module_id
                        JOIN subscriptions s ON sm.subscription_id = s.id
                        WHERE s.company_id = cp.id
                        AND sm.included = true
                        AND s.status IN ('TRIAL', 'ACTIVE')
                    )
                )
            )
            AND cp.wizard_completed = true
            "#
        )
        .bind(update_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| CompanyNotification {
            company_id: row.get("company_id"),
            company_name: row.get("company_name"),
            update_id: row.get("update_id"),
            update_title: row.get("update_title"),
            affected_modules: row.get("affected_modules"),
            action_required: row.get("action_required"),
        }).collect())
    }

    /// Mark notification as sent
    pub async fn mark_notification_sent(
        &self,
        update_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE legislative_updates
            SET notification_sent = true,
                notification_sent_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#
        )
        .bind(update_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get pending updates that need notification
    pub async fn get_pending_notifications(
        &self,
    ) -> Result<Vec<LegislativeUpdate>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, regulation, article, update_type, title, description,
                affected_modules, compliance_level, effective_date,
                published_date, source_url, notification_sent, created_at
            FROM legislative_updates
            WHERE notification_sent = false
            AND published_date <= CURRENT_DATE
            ORDER BY published_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| LegislativeUpdate {
            id: row.get("id"),
            regulation: row.get("regulation"),
            article: row.get("article"),
            update_type: row.get("update_type"),
            title: row.get("title"),
            description: row.get("description"),
            affected_modules: row.get("affected_modules"),
            compliance_level: row.get("compliance_level"),
            effective_date: row.get("effective_date"),
            published_date: row.get("published_date"),
            source_url: row.get("source_url"),
            notification_sent: row.get("notification_sent"),
            created_at: row.get("created_at"),
        }).collect())
    }

    /// Get modules affected by a regulation
    pub async fn get_modules_for_regulation(
        &self,
        regulation: &str,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT m.name
            FROM modules m
            JOIN module_legislative_mapping mlm ON m.id = mlm.module_id
            WHERE mlm.regulation = $1
            ORDER BY m.name
            "#
        )
        .bind(regulation)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("name")).collect())
    }
}

