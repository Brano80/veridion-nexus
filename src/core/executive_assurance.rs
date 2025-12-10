// Executive Assurance Reporting
// Board-level, non-technical compliance metrics for NIS2/DORA liability protection

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

/// Executive Compliance Scorecard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveComplianceScorecard {
    pub report_date: NaiveDate,
    pub compliance_score: f64, // 0.0-100.0
    pub risk_level: String, // LOW, MEDIUM, HIGH, CRITICAL
    pub liability_protection_status: String, // PROTECTED, AT_RISK, EXPOSED
    pub nis2_readiness: f64, // Percentage: 0.0-100.0
    pub dora_compliance: bool,
    pub total_assets: i32,
    pub compliant_assets: i32,
    pub non_compliant_assets: i32,
    pub critical_issues_count: i32,
    pub high_risk_issues_count: i32,
    pub last_incident_date: Option<DateTime<Utc>>,
    pub days_since_last_incident: Option<i32>,
    pub executive_summary: String,
}

/// Compliance KPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceKPI {
    pub kpi_name: String,
    pub kpi_value: f64,
    pub kpi_unit: String,
    pub kpi_category: String,
    pub target_value: Option<f64>,
    pub status: String, // ON_TRACK, AT_RISK, CRITICAL
}

/// Executive Assurance Service
pub struct ExecutiveAssuranceService;

impl ExecutiveAssuranceService {
    /// Generate executive compliance scorecard
    pub async fn generate_scorecard(
        db_pool: &PgPool,
        report_date: Option<NaiveDate>,
    ) -> Result<ExecutiveComplianceScorecard, String> {
        let report_date = report_date.unwrap_or_else(|| Utc::now().date_naive());

        // Calculate compliance score
        let compliance_score: f64 = sqlx::query_scalar::<_, f64>("SELECT COALESCE(calculate_compliance_score(), 100.0)")
            .fetch_one(db_pool)
            .await
            .map_err(|e| format!("Failed to calculate compliance score: {}", e))
            .unwrap_or(100.0);

        // Calculate NIS2 readiness
        let nis2_readiness: f64 = sqlx::query_scalar::<_, f64>("SELECT COALESCE(calculate_nis2_readiness(), 0.0)")
            .fetch_one(db_pool)
            .await
            .map_err(|e| format!("Failed to calculate NIS2 readiness: {}", e))
            .unwrap_or(0.0);

        // Get asset counts
        #[derive(sqlx::FromRow)]
        struct AssetCounts {
            total: i64,
            compliant: i64,
        }

        let asset_counts: AssetCounts = sqlx::query_as(
            "SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE NOT EXISTS (
                    SELECT 1 FROM compliance_records cr
                    JOIN asset_agent_mapping aam ON cr.agent_id = aam.agent_id
                    WHERE aam.asset_id = assets.id
                    AND cr.status LIKE '%BLOCKED%'
                    AND cr.timestamp > CURRENT_TIMESTAMP - INTERVAL '30 days'
                )) as compliant
             FROM assets
             WHERE is_active = true"
        )
        .fetch_one(db_pool)
        .await
        .map_err(|e| format!("Failed to fetch asset counts: {}", e))?;

        // Get issue counts
        #[derive(sqlx::FromRow)]
        struct IssueCounts {
            critical: i64,
            high: i64,
        }

        let issue_counts: IssueCounts = sqlx::query_as(
            "SELECT 
                COUNT(*) FILTER (WHERE risk_level = 'CRITICAL') as critical,
                COUNT(*) FILTER (WHERE risk_level = 'HIGH') as high
             FROM risk_assessments
             WHERE assessed_at > CURRENT_TIMESTAMP - INTERVAL '30 days'"
        )
        .fetch_one(db_pool)
        .await
        .unwrap_or(IssueCounts { critical: 0, high: 0 });

        // Get last incident date
        let last_incident: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT MAX(detected_at) FROM monitoring_events WHERE severity IN ('HIGH', 'CRITICAL')"
        )
        .fetch_optional(db_pool)
        .await
        .ok()
        .flatten();

        let days_since_last_incident = last_incident.map(|dt| {
            (Utc::now() - dt).num_days() as i32
        });

        // Determine risk level
        let risk_level = if compliance_score >= 90.0 && issue_counts.critical == 0 {
            "LOW"
        } else if compliance_score >= 75.0 && issue_counts.critical == 0 {
            "MEDIUM"
        } else if compliance_score >= 50.0 || issue_counts.critical > 0 {
            "HIGH"
        } else {
            "CRITICAL"
        };

        // Determine liability protection status
        let liability_protection_status = if compliance_score >= 90.0 
            && nis2_readiness >= 90.0 
            && issue_counts.critical == 0 {
            "PROTECTED"
        } else if compliance_score >= 75.0 && nis2_readiness >= 75.0 {
            "AT_RISK"
        } else {
            "EXPOSED"
        };

        // Check DORA compliance
        let dora_compliance = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM compliance_records
                WHERE timestamp > CURRENT_TIMESTAMP - INTERVAL '30 days'
                AND payload_hash LIKE 'region:%'
            )"
        )
        .fetch_one(db_pool)
        .await
        .unwrap_or(false);

        // Generate executive summary
        let executive_summary = format!(
            "Compliance Score: {:.1}% | NIS2 Readiness: {:.1}% | Risk Level: {} | \
            Total Assets: {} | Compliant: {} | Non-Compliant: {} | \
            Critical Issues: {} | High Risk Issues: {} | \
            Liability Protection: {} | DORA Compliance: {}",
            compliance_score,
            nis2_readiness,
            risk_level,
            asset_counts.total,
            asset_counts.compliant,
            asset_counts.total - asset_counts.compliant,
            issue_counts.critical,
            issue_counts.high,
            liability_protection_status,
            if dora_compliance { "Yes" } else { "No" }
        );

        let scorecard = ExecutiveComplianceScorecard {
            report_date,
            compliance_score,
            risk_level: risk_level.to_string(),
            liability_protection_status: liability_protection_status.to_string(),
            nis2_readiness,
            dora_compliance,
            total_assets: asset_counts.total as i32,
            compliant_assets: asset_counts.compliant as i32,
            non_compliant_assets: (asset_counts.total - asset_counts.compliant) as i32,
            critical_issues_count: issue_counts.critical as i32,
            high_risk_issues_count: issue_counts.high as i32,
            last_incident_date: last_incident,
            days_since_last_incident,
            executive_summary,
        };

        // Store in database
        sqlx::query(
            "INSERT INTO executive_compliance_scorecard (
                report_date, compliance_score, risk_level, liability_protection_status,
                nis2_readiness, dora_compliance, total_assets, compliant_assets,
                non_compliant_assets, critical_issues_count, high_risk_issues_count,
                last_incident_date, days_since_last_incident, executive_summary
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (report_date) DO UPDATE SET
                compliance_score = EXCLUDED.compliance_score,
                risk_level = EXCLUDED.risk_level,
                liability_protection_status = EXCLUDED.liability_protection_status,
                nis2_readiness = EXCLUDED.nis2_readiness,
                dora_compliance = EXCLUDED.dora_compliance,
                total_assets = EXCLUDED.total_assets,
                compliant_assets = EXCLUDED.compliant_assets,
                non_compliant_assets = EXCLUDED.non_compliant_assets,
                critical_issues_count = EXCLUDED.critical_issues_count,
                high_risk_issues_count = EXCLUDED.high_risk_issues_count,
                last_incident_date = EXCLUDED.last_incident_date,
                days_since_last_incident = EXCLUDED.days_since_last_incident,
                executive_summary = EXCLUDED.executive_summary,
                generated_at = CURRENT_TIMESTAMP"
        )
        .bind(report_date)
        .bind(scorecard.compliance_score)
        .bind(&scorecard.risk_level)
        .bind(&scorecard.liability_protection_status)
        .bind(scorecard.nis2_readiness)
        .bind(scorecard.dora_compliance)
        .bind(scorecard.total_assets)
        .bind(scorecard.compliant_assets)
        .bind(scorecard.non_compliant_assets)
        .bind(scorecard.critical_issues_count)
        .bind(scorecard.high_risk_issues_count)
        .bind(scorecard.last_incident_date)
        .bind(scorecard.days_since_last_incident)
        .bind(&scorecard.executive_summary)
        .execute(db_pool)
        .await
        .map_err(|e| format!("Failed to store scorecard: {}", e))?;

        Ok(scorecard)
    }

    /// Get compliance KPIs
    pub async fn get_compliance_kpis(
        db_pool: &PgPool,
        category: Option<&str>,
    ) -> Result<Vec<ComplianceKPI>, String> {
        let mut query = sqlx::QueryBuilder::new(
            "SELECT kpi_name, kpi_value, kpi_unit, kpi_category, target_value, status
             FROM compliance_kpis"
        );

        if let Some(cat) = category {
            query.push(" WHERE kpi_category = ");
            query.push_bind(cat);
        }

        query.push(" ORDER BY kpi_category, kpi_name");

        #[derive(sqlx::FromRow)]
        struct KPIRow {
            kpi_name: String,
            kpi_value: Option<f64>,
            kpi_unit: String,
            kpi_category: String,
            target_value: Option<f64>,
            status: String,
        }

        let rows: Vec<KPIRow> = query
            .build_query_as()
            .fetch_all(db_pool)
            .await
            .map_err(|e| format!("Failed to fetch KPIs: {}", e))?;

        let kpis = rows.into_iter().map(|row| ComplianceKPI {
            kpi_name: row.kpi_name,
            kpi_value: row.kpi_value.unwrap_or(0.0),
            kpi_unit: row.kpi_unit,
            kpi_category: row.kpi_category,
            target_value: row.target_value,
            status: row.status,
        }).collect();

        Ok(kpis)
    }
}

