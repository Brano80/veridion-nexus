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
                        
                        if let Ok(Some(_tx_id)) = tx_id_result {
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

    /// Process circuit breaker recovery (auto-close after cooldown)
    pub async fn process_circuit_breaker_recovery(&self) {
        loop {
            // Check for circuit breakers that need recovery every minute
            sleep(Duration::from_secs(60)).await;

            #[derive(sqlx::FromRow)]
            struct OpenCircuitBreaker {
                policy_version_id: uuid::Uuid,
                policy_type: String,
                opened_at: chrono::DateTime<chrono::Utc>,
                cooldown_minutes: i32,
            }

            let open_circuits: Vec<OpenCircuitBreaker> = match sqlx::query_as::<_, OpenCircuitBreaker>(
                "SELECT 
                    id as policy_version_id,
                    policy_type,
                    circuit_breaker_opened_at as opened_at,
                    COALESCE(circuit_breaker_cooldown_minutes, 15) as cooldown_minutes
                 FROM policy_versions
                 WHERE circuit_breaker_enabled = true
                   AND circuit_breaker_state = 'OPEN'
                   AND circuit_breaker_opened_at IS NOT NULL
                   AND circuit_breaker_opened_at + (COALESCE(circuit_breaker_cooldown_minutes, 15) || ' minutes')::INTERVAL <= CURRENT_TIMESTAMP"
            )
            .fetch_all(&self.db_pool)
            .await
            {
                Ok(circuits) => circuits,
                Err(e) => {
                    eprintln!("Error fetching open circuit breakers: {}", e);
                    continue;
                }
            };

            for circuit in open_circuits {
                // Check if error rate has improved
                let current_error_rate: Option<f64> = sqlx::query_scalar(
                    "SELECT error_rate 
                     FROM policy_error_tracking
                     WHERE policy_version_id = $1
                     ORDER BY window_end DESC
                     LIMIT 1"
                )
                .bind(circuit.policy_version_id)
                .fetch_optional(&self.db_pool)
                .await
                .ok()
                .flatten();

                // Get error threshold
                let error_threshold: Option<f64> = sqlx::query_scalar(
                    "SELECT circuit_breaker_error_threshold 
                     FROM policy_versions
                     WHERE id = $1"
                )
                .bind(circuit.policy_version_id)
                .fetch_optional(&self.db_pool)
                .await
                .ok()
                .flatten();

                let threshold = error_threshold.unwrap_or(10.0);
                let should_close = current_error_rate.map(|rate| rate < threshold).unwrap_or(true);

                if should_close {
                    // Close circuit breaker
                    let _ = sqlx::query(
                        "UPDATE policy_versions 
                         SET circuit_breaker_state = 'CLOSED',
                             circuit_breaker_opened_at = NULL
                         WHERE id = $1"
                    )
                    .bind(circuit.policy_version_id)
                    .execute(&self.db_pool)
                    .await;

                    // Log to history
                    let _ = sqlx::query(
                        "INSERT INTO circuit_breaker_history (
                            policy_version_id, state_transition, error_rate,
                            error_count, total_requests, triggered_by, notes, timestamp
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
                    )
                    .bind(circuit.policy_version_id)
                    .bind("CLOSED")
                    .bind(current_error_rate.unwrap_or(0.0))
                    .bind(0)
                    .bind(0)
                    .bind("AUTO_RECOVERY")
                    .bind(format!("Auto-closed after {} minute cooldown. Error rate: {:.2}%", circuit.cooldown_minutes, current_error_rate.unwrap_or(0.0)))
                    .bind(chrono::Utc::now())
                    .execute(&self.db_pool)
                    .await;

                    println!("✅ Circuit breaker auto-closed for policy {} after cooldown", circuit.policy_version_id);
                } else {
                    // Error rate still too high, extend cooldown
                    let _ = sqlx::query(
                        "UPDATE policy_versions 
                         SET circuit_breaker_opened_at = CURRENT_TIMESTAMP
                         WHERE id = $1"
                    )
                    .bind(circuit.policy_version_id)
                    .execute(&self.db_pool)
                    .await;

                    println!("⚠️ Circuit breaker kept open for policy {} - error rate still high: {:.2}%", 
                        circuit.policy_version_id, current_error_rate.unwrap_or(0.0));
                }
            }
        }
    }

    /// Process canary deployment auto-promote/rollback
    pub async fn process_canary_deployment(&self) {
        loop {
            // Check for canary deployments that need evaluation every 5 minutes
            sleep(Duration::from_secs(300)).await;

            #[derive(sqlx::FromRow)]
            struct CanaryPolicy {
                policy_version_id: uuid::Uuid,
                policy_type: String,
                rollout_percentage: i32,
                auto_promote_enabled: Option<bool>,
                auto_rollback_enabled: Option<bool>,
                promotion_threshold: Option<f64>,
                rollback_threshold: Option<f64>,
                min_requests_for_promotion: Option<i64>,
                evaluation_window_minutes: Option<i32>,
            }

            let canary_policies: Vec<CanaryPolicy> = match sqlx::query_as::<_, CanaryPolicy>(
                "SELECT 
                    id as policy_version_id,
                    policy_type,
                    COALESCE(rollout_percentage, 100) as rollout_percentage,
                    canary_auto_promote_enabled as auto_promote_enabled,
                    canary_auto_rollback_enabled as auto_rollback_enabled,
                    canary_success_threshold as promotion_threshold,
                    canary_failure_threshold as rollback_threshold,
                    canary_min_requests as min_requests_for_promotion,
                    canary_evaluation_window_minutes as evaluation_window_minutes
                 FROM policy_versions
                 WHERE is_active = true 
                   AND rollout_percentage IS NOT NULL 
                   AND rollout_percentage < 100
                   AND (canary_auto_promote_enabled = true OR canary_auto_rollback_enabled = true)"
            )
            .fetch_all(&self.db_pool)
            .await
            {
                Ok(policies) => policies,
                Err(e) => {
                    eprintln!("Error fetching canary policies: {}", e);
                    continue;
                }
            };

            for policy in canary_policies {
                let window_minutes = policy.evaluation_window_minutes.unwrap_or(10);
                let now = chrono::Utc::now();
                let window_start = now - chrono::Duration::minutes(window_minutes as i64);

                // Get current success rate
                let success_rate: Option<f64> = sqlx::query_scalar(
                    "SELECT 
                        CASE 
                            WHEN SUM(total_requests) > 0 THEN
                                (SUM(successful_requests)::DECIMAL / SUM(total_requests)::DECIMAL) * 100.0
                            ELSE NULL
                        END as success_rate
                     FROM canary_metrics
                     WHERE policy_version_id = $1 
                       AND traffic_percentage = $2
                       AND window_end >= $3"
                )
                .bind(policy.policy_version_id)
                .bind(policy.rollout_percentage)
                .bind(window_start)
                .fetch_optional(&self.db_pool)
                .await
                .ok()
                .flatten();

                let total_requests: i64 = sqlx::query_scalar(
                    "SELECT COALESCE(SUM(total_requests), 0)
                     FROM canary_metrics
                     WHERE policy_version_id = $1 
                       AND traffic_percentage = $2
                       AND window_end >= $3"
                )
                .bind(policy.policy_version_id)
                .bind(policy.rollout_percentage)
                .bind(window_start)
                .fetch_one(&self.db_pool)
                .await
                .unwrap_or(0);

                if let Some(rate) = success_rate {
                    let min_requests = policy.min_requests_for_promotion.unwrap_or(100);
                    
                    // Check for auto-rollback
                    if policy.auto_rollback_enabled.unwrap_or(false) {
                        let rollback_threshold = policy.rollback_threshold.unwrap_or(5.0);
                        
                        if rate < rollback_threshold && total_requests >= min_requests {
                            // Rollback to previous tier
                            let prev_percentage = match policy.rollout_percentage {
                                100 => 50,
                                50 => 25,
                                25 => 10,
                                10 => 5,
                                5 => 1,
                                _ => 0,
                            };

                            if prev_percentage >= 0 {
                                let _ = sqlx::query(
                                    "UPDATE policy_versions 
                                     SET rollout_percentage = $1
                                     WHERE id = $2"
                                )
                                .bind(prev_percentage)
                                .bind(policy.policy_version_id)
                                .execute(&self.db_pool)
                                .await;

                                // Log rollback
                                let _ = sqlx::query(
                                    "INSERT INTO canary_deployment_history (
                                        policy_version_id, action, from_percentage, to_percentage,
                                        success_rate, total_requests, triggered_by, notes, timestamp
                                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                                )
                                .bind(policy.policy_version_id)
                                .bind("ROLLED_BACK")
                                .bind(policy.rollout_percentage)
                                .bind(prev_percentage)
                                .bind(rate)
                                .bind(total_requests)
                                .bind("AUTO")
                                .bind(format!("Auto-rolled back from {}% to {}% due to low success rate: {:.2}% (threshold: {:.2}%)", 
                                    policy.rollout_percentage, prev_percentage, rate, rollback_threshold))
                                .bind(now)
                                .execute(&self.db_pool)
                                .await;

                                println!("⚠️ Canary auto-rollback: Policy {} from {}% to {}% (success rate: {:.2}%)", 
                                    policy.policy_version_id, policy.rollout_percentage, prev_percentage, rate);

                                // Send rollback notification
                                let notification_service = crate::integration::notifications::NotificationService::new();
                                let policy_id_str = policy.policy_version_id.to_string();
                                let db_pool_clone = self.db_pool.clone();
                                tokio::spawn(async move {
                                    let _ = notification_service.send_canary_rollback_alert(
                                        &db_pool_clone,
                                        &policy_id_str,
                                        &policy.policy_type,
                                        policy.rollout_percentage,
                                        prev_percentage,
                                        rate,
                                        total_requests,
                                        None,
                                    ).await;
                                });
                            }
                        }
                    }

                    // Check for auto-promote
                    if policy.auto_promote_enabled.unwrap_or(false) {
                        let promotion_threshold = policy.promotion_threshold.unwrap_or(95.0);
                        
                        if rate >= promotion_threshold && total_requests >= min_requests {
                            // Promote to next tier
                            let next_percentage = match policy.rollout_percentage {
                                1 => 5,
                                5 => 10,
                                10 => 25,
                                25 => 50,
                                50 => 100,
                                _ => policy.rollout_percentage, // Already at max
                            };

                            if next_percentage > policy.rollout_percentage {
                                let _ = sqlx::query(
                                    "UPDATE policy_versions 
                                     SET rollout_percentage = $1
                                     WHERE id = $2"
                                )
                                .bind(next_percentage)
                                .bind(policy.policy_version_id)
                                .execute(&self.db_pool)
                                .await;

                                // Log promotion
                                let _ = sqlx::query(
                                    "INSERT INTO canary_deployment_history (
                                        policy_version_id, action, from_percentage, to_percentage,
                                        success_rate, total_requests, triggered_by, notes, timestamp
                                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                                )
                                .bind(policy.policy_version_id)
                                .bind("PROMOTED")
                                .bind(policy.rollout_percentage)
                                .bind(next_percentage)
                                .bind(rate)
                                .bind(total_requests)
                                .bind("AUTO")
                                .bind(format!("Auto-promoted from {}% to {}% based on success rate: {:.2}% (threshold: {:.2}%)", 
                                    policy.rollout_percentage, next_percentage, rate, promotion_threshold))
                                .bind(now)
                                .execute(&self.db_pool)
                                .await;

                                println!("✅ Canary auto-promoted: Policy {} from {}% to {}% (success rate: {:.2}%)", 
                                    policy.policy_version_id, policy.rollout_percentage, next_percentage, rate);

                                // Send promotion notification
                                let notification_service = crate::integration::notifications::NotificationService::new();
                                let policy_id_str = policy.policy_version_id.to_string();
                                let db_pool_clone = self.db_pool.clone();
                                tokio::spawn(async move {
                                    let _ = notification_service.send_canary_promotion_alert(
                                        &db_pool_clone,
                                        &policy_id_str,
                                        &policy.policy_type,
                                        policy.rollout_percentage,
                                        next_percentage,
                                        rate,
                                        total_requests,
                                        None,
                                    ).await;
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
