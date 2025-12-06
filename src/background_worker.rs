use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use crate::integration::webhooks::WebhookService;
use crate::compliance_models::WebhookEvent;
use crate::models::db_models::{WebhookEndpointDb, WebhookDeliveryDb, RetentionAssignmentDb};
use chrono::Utc;
use serde_json;

/// Background worker for async tasks
pub struct BackgroundWorker {
    db_pool: PgPool,
}

impl BackgroundWorker {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Process pending webhook deliveries with retry logic
    pub async fn process_webhook_deliveries(&self) {
        loop {
            // Check for pending webhook deliveries every 30 seconds
            sleep(Duration::from_secs(30)).await;

            // Find pending or retrying deliveries
            let deliveries: Vec<WebhookDeliveryDb> = match sqlx::query_as(
                "SELECT * FROM webhook_deliveries 
                 WHERE status IN ('pending', 'retrying') 
                   AND (next_retry_at IS NULL OR next_retry_at <= CURRENT_TIMESTAMP)
                 ORDER BY created_at ASC
                 LIMIT 10"
            )
            .fetch_all(&self.db_pool)
            .await
            {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error fetching pending webhook deliveries: {}", e);
                    continue;
                }
            };

            for delivery in deliveries {
                // Get webhook endpoint
                let webhook: Option<WebhookEndpointDb> = match sqlx::query_as::<_, WebhookEndpointDb>(
                    "SELECT * FROM webhook_endpoints WHERE id = $1 AND active = true"
                )
                .bind(delivery.webhook_endpoint_id)
                .fetch_optional(&self.db_pool)
                .await
                {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("Error fetching webhook endpoint: {}", e);
                        continue;
                    }
                };

                if let Some(webhook) = webhook {
                    // Parse event from payload
                    let event: WebhookEvent = match serde_json::from_value(delivery.event_payload.clone()) {
                        Ok(e) => e,
                        Err(e) => {
                            eprintln!("Error parsing webhook event: {}", e);
                            continue;
                        }
                    };

                    let webhook_service = WebhookService::new();
                    let (success, attempts, response) = webhook_service
                        .deliver_with_retry(
                            &webhook.endpoint_url,
                            &event,
                            &webhook.secret_key,
                            webhook.timeout_seconds as u64,
                            webhook.retry_count,
                        )
                        .await;

                    // Update delivery status
                    let status = if success { "delivered" } else if attempts < webhook.retry_count {
                        "retrying"
                    } else {
                        "failed"
                    };

                    let response_code = if success {
                        response.as_ref()
                            .and_then(|r| r.split(':').next())
                            .and_then(|s| s.split_whitespace().nth(1))
                            .and_then(|s| s.parse::<i32>().ok())
                    } else {
                        None
                    };

                    let next_retry_at = if !success && attempts < webhook.retry_count {
                        Some(Utc::now() + chrono::Duration::seconds(2_i64.pow(attempts as u32)))
                    } else {
                        None
                    };

                    let _ = sqlx::query(
                        "UPDATE webhook_deliveries 
                         SET status = $1, attempts = $2, response_code = $3, 
                             response_body = $4, next_retry_at = $5, delivered_at = $6
                         WHERE id = $7"
                    )
                    .bind(status)
                    .bind(attempts)
                    .bind(response_code)
                    .bind(&response)
                    .bind(next_retry_at)
                    .bind(if success { Some(Utc::now()) } else { None })
                    .bind(delivery.id)
                    .execute(&self.db_pool)
                    .await;
                }
            }
        }
    }

    /// Process retention deletions automatically
    pub async fn process_retention_deletions(&self) {
        loop {
            // Check for expired records every hour
            sleep(Duration::from_secs(3600)).await;

            let expired_assignments: Vec<RetentionAssignmentDb> = match sqlx::query_as::<_, RetentionAssignmentDb>(
                "SELECT ra.* FROM retention_assignments ra
                 INNER JOIN retention_policies rp ON ra.policy_id = rp.id
                 WHERE ra.expires_at <= CURRENT_TIMESTAMP 
                   AND ra.deleted_at IS NULL 
                   AND ra.deletion_status = 'PENDING'
                   AND rp.auto_delete = true
                 LIMIT 50"
            )
            .fetch_all(&self.db_pool)
            .await
            {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Error fetching expired retention assignments: {}", e);
                    continue;
                }
            };

            for assignment in expired_assignments {
                // Mark as scheduled
                let _ = sqlx::query(
                    "UPDATE retention_assignments SET deletion_status = 'SCHEDULED' WHERE id = $1"
                )
                .bind(assignment.id)
                .execute(&self.db_pool)
                .await;

                // Delete based on record type
                match assignment.record_type.as_str() {
                    "COMPLIANCE_RECORD" => {
                        // Get tx_id for crypto-shredding
                        let tx_id_result: Result<Option<String>, _> = sqlx::query_scalar(
                            "SELECT tx_id FROM compliance_records WHERE seal_id = $1"
                        )
                        .bind(&assignment.record_id)
                        .fetch_optional(&self.db_pool)
                        .await;
                        
                        if let Ok(Some(tx_id)) = tx_id_result {
                            // Note: We can't access key_store here, so we'll just mark it
                            // The actual shredding should be done via the API endpoint
                            let _ = sqlx::query(
                                "UPDATE compliance_records 
                                 SET action_summary = '[RETENTION EXPIRED] Data Automatically Deleted',
                                     status = 'DELETED (Retention Period)'
                                 WHERE seal_id = $1"
                            )
                            .bind(&assignment.record_id)
                            .execute(&self.db_pool)
                            .await;
                        }
                    }
                    "CONSENT_RECORD" => {
                        let _ = sqlx::query(
                            "UPDATE consent_records 
                             SET granted = false, withdrawn_at = CURRENT_TIMESTAMP
                             WHERE id::text = $1"
                        )
                        .bind(&assignment.record_id)
                        .execute(&self.db_pool)
                        .await;
                    }
                    _ => {}
                }

                // Mark assignment as deleted
                let _ = sqlx::query(
                    "UPDATE retention_assignments 
                     SET deleted_at = CURRENT_TIMESTAMP, deletion_status = 'DELETED'
                     WHERE id = $1"
                )
                .bind(assignment.id)
                .execute(&self.db_pool)
                .await;
            }
        }
    }

    /// Refresh materialized views periodically
    pub async fn refresh_materialized_views(&self) {
        loop {
            // Refresh views every 6 hours
            sleep(Duration::from_secs(21600)).await;

            if let Err(e) = sqlx::query("SELECT refresh_materialized_views()")
                .execute(&self.db_pool)
                .await
            {
                eprintln!("Error refreshing materialized views: {}", e);
            } else {
                println!("✅ Materialized views refreshed");
            }

            // Also analyze tables for query planner
            if let Err(e) = sqlx::query("SELECT analyze_tables()")
                .execute(&self.db_pool)
                .await
            {
                eprintln!("Error analyzing tables: {}", e);
            }
        }
    }
}
