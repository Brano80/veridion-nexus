// Notification Service for GDPR Article 33 and EU AI Act Article 13
// Handles email, SMS, and in-app notifications with retry logic

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{Mailbox, SinglePart};
use lettre::transport::smtp::authentication::Credentials;

/// Notification channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Sms,
    InApp,
    Slack,
}

impl ToString for NotificationChannel {
    fn to_string(&self) -> String {
        match self {
            NotificationChannel::Email => "EMAIL".to_string(),
            NotificationChannel::Sms => "SMS".to_string(),
            NotificationChannel::InApp => "IN_APP".to_string(),
            NotificationChannel::Slack => "SLACK".to_string(),
        }
    }
}

/// Notification types for compliance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    DataBreach,           // GDPR Article 33
    HighRiskAiAction,     // EU AI Act Article 13
    AutomatedDecision,    // GDPR Article 22
    RestrictionApplied,   // GDPR Article 18
    ObjectionReceived,   // GDPR Article 21
    RectificationDone,   // GDPR Article 19
    ErasureDone,         // GDPR Article 17
    ShadowModeViolation,  // Shadow mode detected violation
    CircuitBreakerOpened,
    PolicyHealthDegraded,
    PolicyHealthCritical, // Circuit breaker opened
    PolicyApprovalPending, // Policy requires approval
    PolicyApprovalCompleted, // Policy approved/rejected
    PolicyAutoRollback, // Policy auto-rolled back
    CanaryPromotion, // Canary deployment auto-promoted
    CanaryRollback, // Canary deployment auto-rolled back
    ComplianceViolation, // Compliance violation detected
}

impl ToString for NotificationType {
    fn to_string(&self) -> String {
        match self {
            NotificationType::DataBreach => "DATA_BREACH".to_string(),
            NotificationType::HighRiskAiAction => "HIGH_RISK_AI_ACTION".to_string(),
            NotificationType::AutomatedDecision => "AUTOMATED_DECISION".to_string(),
            NotificationType::RestrictionApplied => "RESTRICTION_APPLIED".to_string(),
            NotificationType::ObjectionReceived => "OBJECTION_RECEIVED".to_string(),
            NotificationType::RectificationDone => "RECTIFICATION_DONE".to_string(),
            NotificationType::ErasureDone => "ERASURE_DONE".to_string(),
            NotificationType::ShadowModeViolation => "SHADOW_MODE_VIOLATION".to_string(),
            NotificationType::CircuitBreakerOpened => "CIRCUIT_BREAKER_OPENED".to_string(),
            NotificationType::PolicyHealthDegraded => "POLICY_HEALTH_DEGRADED".to_string(),
            NotificationType::PolicyHealthCritical => "POLICY_HEALTH_CRITICAL".to_string(),
            NotificationType::PolicyApprovalPending => "POLICY_APPROVAL_PENDING".to_string(),
            NotificationType::PolicyApprovalCompleted => "POLICY_APPROVAL_COMPLETED".to_string(),
            NotificationType::PolicyAutoRollback => "POLICY_AUTO_ROLLBACK".to_string(),
            NotificationType::CanaryPromotion => "CANARY_PROMOTION".to_string(),
            NotificationType::CanaryRollback => "CANARY_ROLLBACK".to_string(),
            NotificationType::ComplianceViolation => "COMPLIANCE_VIOLATION".to_string(),
        }
    }
}

/// Notification request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub user_id: String,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub subject: Option<String>,
    pub body: String,
    pub language: Option<String>, // ISO 639-1 code, default: "en"
    pub related_entity_type: Option<String>, // "BREACH", "COMPLIANCE_RECORD", etc.
    pub related_entity_id: Option<String>, // breach_id, seal_id, etc.
}

/// Notification service configuration
pub struct NotificationService {
    client: Client,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_from: Option<String>,
    twilio_account_sid: Option<String>,
    twilio_auth_token: Option<String>,
    twilio_from_number: Option<String>,
    slack_webhook_url: Option<String>,
    slack_channel: Option<String>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            smtp_host: std::env::var("SMTP_HOST").ok(),
            smtp_port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok()),
            smtp_username: std::env::var("SMTP_USERNAME").ok(),
            smtp_password: std::env::var("SMTP_PASSWORD").ok(),
            smtp_from: std::env::var("SMTP_FROM").ok(),
            twilio_account_sid: std::env::var("TWILIO_ACCOUNT_SID").ok(),
            twilio_auth_token: std::env::var("TWILIO_AUTH_TOKEN").ok(),
            twilio_from_number: std::env::var("TWILIO_FROM_NUMBER").ok(),
            slack_webhook_url: std::env::var("SLACK_WEBHOOK_URL").ok(),
            slack_channel: std::env::var("SLACK_CHANNEL").ok(),
        }
    }

    /// Send notification with retry logic
    pub async fn send_notification(
        &self,
        db_pool: &PgPool,
        request: NotificationRequest,
    ) -> Result<String, String> {
        let notification_id = format!("NOTIF-{}", Uuid::new_v4().to_string().replace("-", "").chars().take(12).collect::<String>());
        let language = request.language.as_deref().unwrap_or("en");
        let max_retries = 3;

        // Store notification record in database
        let notification_record_id = Uuid::new_v4();
        let channel_str = request.channel.to_string();
        let type_str = request.notification_type.to_string();
        
        let insert_result = sqlx::query(
            "INSERT INTO user_notifications (
                id, notification_id, user_id, notification_type, channel,
                subject, body, status, language, related_entity_type, related_entity_id,
                max_retries
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id"
        )
        .bind(notification_record_id)
        .bind(&notification_id)
        .bind(&request.user_id)
        .bind(&type_str)
        .bind(&channel_str)
        .bind(&request.subject)
        .bind(&request.body)
        .bind("PENDING")
        .bind(language)
        .bind(&request.related_entity_type)
        .bind(&request.related_entity_id)
        .bind(max_retries as i32)
        .fetch_one(db_pool)
        .await;

        if insert_result.is_err() {
            return Err("Failed to store notification record".to_string());
        }

        // Get user contact information (email/phone) from database
        let user_email: Option<String> = sqlx::query_scalar(
            "SELECT email FROM users WHERE id::text = $1 OR username = $1 OR email = $1 LIMIT 1"
        )
        .bind(&request.user_id)
        .fetch_optional(db_pool)
        .await
        .ok()
        .flatten();
        
        // For SMS, we'd need phone number - for now use user_id as fallback
        // In production, add phone_number column to users table
        let user_phone = user_email.as_ref().map(|_| &request.user_id).map(|s| s.as_str());

        // Attempt to send notification with retry logic
        let mut last_error = None;
        for attempt in 1..=max_retries {
            let send_result = match request.channel {
                NotificationChannel::Email => {
                    let subject = request.subject.as_ref().map(|s| s.as_str()).unwrap_or("");
                    // Use user email if available, otherwise use SMTP_FROM as fallback
                    let recipient = user_email.as_deref().unwrap_or_else(|| {
                        self.smtp_from.as_deref().unwrap_or("noreply@veridion.nexus")
                    });
                    self.send_email(recipient, subject, &request.body).await
                }
                NotificationChannel::Sms => {
                    // Use user_id as phone number (in production, use actual phone from DB)
                    let phone = user_phone.unwrap_or(&request.user_id);
                    self.send_sms(phone, &request.body).await
                }
                NotificationChannel::InApp => {
                    // In-app notifications are stored in DB only, no external service needed
                    Ok("IN_APP".to_string())
                }
                NotificationChannel::Slack => {
                    self.send_slack(&request.body, request.subject.as_deref()).await
                }
            };

            match send_result {
                Ok(_response) => {
                    // Update notification status to SENT
                    let now = Utc::now();
                    let _ = sqlx::query(
                        "UPDATE user_notifications 
                         SET status = 'SENT', sent_at = $1, retry_count = $2
                         WHERE notification_id = $3"
                    )
                    .bind(now)
                    .bind(attempt as i32)
                    .bind(&notification_id)
                    .execute(db_pool)
                    .await;

                    return Ok(notification_id);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // Exponential backoff: 1s, 2s, 4s
                        let delay = Duration::from_secs(2_u64.pow((attempt - 1) as u32));
                        sleep(delay).await;
                    }
                }
            }
        }

        // All retries failed - update status to FAILED
        let _ = sqlx::query(
            "UPDATE user_notifications 
             SET status = 'FAILED', failure_reason = $1, retry_count = $2
             WHERE notification_id = $3"
        )
        .bind(last_error.as_deref().unwrap_or("Unknown error"))
        .bind(max_retries as i32)
        .bind(&notification_id)
        .execute(db_pool)
        .await;

        Err(last_error.unwrap_or_else(|| "Failed after all retries".to_string()))
    }

    /// Send email notification via SMTP using lettre crate
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<String, String> {
        // Check if SMTP configuration is available
        if let (Some(host), Some(port), Some(username), Some(password), Some(from_email)) = (
            &self.smtp_host,
            &self.smtp_port,
            &self.smtp_username,
            &self.smtp_password,
            &self.smtp_from,
        ) {
            // Parse from and to email addresses
            let from_addr: Mailbox = from_email.parse()
                .map_err(|e| format!("Invalid from email address: {}", e))?;
            
            let to_addr: Mailbox = to.parse()
                .map_err(|e| format!("Invalid recipient email address: {}", e))?;
            
            // Build email message
            let email = Message::builder()
                .from(from_addr)
                .to(to_addr)
                .subject(subject)
                .singlepart(SinglePart::plain(body.to_string()))
                .map_err(|e| format!("Failed to build email message: {}", e))?;
            
            // Create SMTP transport
            let creds = Credentials::new(username.clone(), password.clone());
            let mailer = SmtpTransport::relay(host)
                .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
                .port(*port)
                .credentials(creds)
                .build();
            
            // Send email (blocking operation, but we're in async context)
            match mailer.send(&email) {
                Ok(_) => {
                    println!("📧 [EMAIL] Successfully sent: Subject: {}", subject);
                    Ok("EMAIL_SENT".to_string())
                }
                Err(e) => {
                    eprintln!("📧 [EMAIL] Failed to send: {}", e);
                    Err(format!("SMTP error: {}", e))
                }
            }
        } else {
            // Fallback: log notification (for development/testing)
            println!("📧 [EMAIL - MOCK] Subject: {} | Body: {}", subject, body);
            Ok("EMAIL_MOCKED".to_string())
        }
    }

    /// Send SMS notification via Twilio API
    async fn send_sms(&self, to_number: &str, body: &str) -> Result<String, String> {
        if let (Some(account_sid), Some(auth_token), Some(from_number)) = (
            &self.twilio_account_sid,
            &self.twilio_auth_token,
            &self.twilio_from_number,
        ) {
            // Twilio API endpoint
            let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", account_sid);
            
            // Prepare form data
            let mut params = std::collections::HashMap::new();
            params.insert("From", from_number.as_str());
            params.insert("To", to_number);
            params.insert("Body", body);
            
            // Make API call with basic auth
            let response = self.client
                .post(&url)
                .basic_auth(account_sid, Some(auth_token))
                .form(&params)
                .send()
                .await
                .map_err(|e| format!("Twilio API request failed: {}", e))?;
            
            if response.status().is_success() {
                let response_json: serde_json::Value = response.json().await
                    .map_err(|e| format!("Failed to parse Twilio response: {}", e))?;
                
                if let Some(sid) = response_json.get("sid").and_then(|s| s.as_str()) {
                    println!("📱 [SMS] Successfully sent: SID: {}", sid);
                    Ok(format!("SMS_SENT:{}", sid))
                } else {
                    Err("Twilio response missing SID".to_string())
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!("📱 [SMS] Twilio API error: {}", error_text);
                Err(format!("Twilio API error: {}", error_text))
            }
        } else {
            // Fallback: log notification (for development/testing)
            println!("📱 [SMS - MOCK] To: {} | Body: {}", to_number, body);
            Ok("SMS_MOCKED".to_string())
        }
    }

    /// Send Slack notification via webhook
    async fn send_slack(&self, text: &str, subject: Option<&str>) -> Result<String, String> {
        if let Some(webhook_url) = &self.slack_webhook_url {
            let channel = self.slack_channel.as_deref().unwrap_or("#compliance-alerts");
            
            // Format Slack message payload
            let mut blocks = Vec::new();
            
            if let Some(subj) = subject {
                blocks.push(serde_json::json!({
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": subj
                    }
                }));
            }
            
            blocks.push(serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": text
                }
            }));
            
            let payload = serde_json::json!({
                "channel": channel,
                "text": subject.unwrap_or(text),
                "blocks": blocks
            });
            
            let response = self.client
                .post(webhook_url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Slack webhook request failed: {}", e))?;
            
            if response.status().is_success() {
                println!("💬 [Slack] Successfully sent to {}", channel);
                Ok("SLACK_SENT".to_string())
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!("💬 [Slack] Webhook error: {}", error_text);
                Err(format!("Slack webhook error: {}", error_text))
            }
        } else {
            // Fallback: log notification (for development/testing)
            println!("💬 [Slack - MOCK] Channel: {} | Message: {}", 
                self.slack_channel.as_deref().unwrap_or("#compliance-alerts"), text);
            Ok("SLACK_MOCKED".to_string())
        }
    }

    /// Load notification template from database
    pub async fn load_template(
        db_pool: &PgPool,
        template_type: &str,
        channel: &str,
        language: &str,
    ) -> Option<(Option<String>, String)> {
        #[derive(sqlx::FromRow)]
        struct TemplateRow {
            subject_template: Option<String>,
            body_template: String,
        }
        
        let result = sqlx::query_as::<_, TemplateRow>(
            "SELECT subject_template, body_template 
             FROM notification_templates 
             WHERE template_type = $1 AND channel = $2 AND language = $3 AND is_active = true
             LIMIT 1"
        )
        .bind(template_type)
        .bind(channel)
        .bind(language)
        .fetch_optional(db_pool)
        .await
        .ok()?;

        result.map(|row| (row.subject_template, row.body_template))
    }

    /// Render template with variables
    pub fn render_template(template: &str, variables: &Value) -> String {
        let mut rendered = template.to_string();
        
        if let Some(obj) = variables.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement_str = value.as_str().map(|s| s.to_string()).unwrap_or_else(|| value.to_string());
                let replacement = replacement_str.as_str();
                rendered = rendered.replace(&placeholder, replacement);
            }
        }
        
        rendered
    }

    /// Send data breach notification (GDPR Article 33)
    pub async fn notify_data_breach(
        &self,
        db_pool: &PgPool,
        user_id: &str,
        breach_id: &str,
        breach_type: &str,
        detected_at: &DateTime<Utc>,
        description: &str,
        channel: NotificationChannel,
    ) -> Result<String, String> {
        // Load template
        let (subject_template, body_template) = match Self::load_template(
            db_pool,
            "DATA_BREACH",
            &channel.to_string(),
            "en",
        ).await {
            Some(t) => t,
            None => {
                // Fallback template
                (
                    Some("Data Breach Notification".to_string()),
                    format!(
                        "Dear User,\n\nWe are writing to inform you of a data breach.\n\nBreach Details:\n- Type: {}\n- Detected: {}\n- Description: {}\n\nWe have reported this breach to the relevant supervisory authority within 72 hours as required by GDPR Article 33.",
                        breach_type,
                        detected_at.format("%Y-%m-%d %H:%M:%S"),
                        description
                    ),
                )
            }
        };

        // Render template (simplified - in production, use proper templating)
        let subject = subject_template.map(|s| s.replace("{{breach_type}}", breach_type));
        let body = body_template
            .replace("{{breach_type}}", breach_type)
            .replace("{{detected_at}}", &detected_at.format("%Y-%m-%d %H:%M:%S").to_string())
            .replace("{{description}}", description);

        let request = NotificationRequest {
            user_id: user_id.to_string(),
            notification_type: NotificationType::DataBreach,
            channel,
            subject,
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("BREACH".to_string()),
            related_entity_id: Some(breach_id.to_string()),
        };

        self.send_notification(db_pool, request).await
    }

    /// Get user notification preferences
    pub async fn get_user_preferences(
        db_pool: &PgPool,
        user_id: &str,
        notification_type: &str,
    ) -> Option<(Vec<String>, String, bool)> {
        // Try to get user-specific preferences
        #[derive(sqlx::FromRow)]
        struct PreferenceRow {
            preferred_channels: serde_json::Value,
            language: String,
            enabled: bool,
        }

        let result: Option<PreferenceRow> = sqlx::query_as(
            "SELECT preferred_channels, language, enabled
             FROM user_notification_preferences
             WHERE user_id = $1 AND notification_type = $2
             LIMIT 1"
        )
        .bind(user_id)
        .bind(notification_type)
        .fetch_optional(db_pool)
        .await
        .ok()?;

        if let Some(pref) = result {
            let channels: Vec<String> = serde_json::from_value(pref.preferred_channels)
                .unwrap_or_else(|_| vec!["EMAIL".to_string()]);
            return Some((channels, pref.language, pref.enabled));
        }

        // Fallback to default preferences
        let default_result: Option<PreferenceRow> = sqlx::query_as(
            "SELECT preferred_channels, language, enabled
             FROM user_notification_preferences
             WHERE user_id = 'DEFAULT' AND notification_type = $1
             LIMIT 1"
        )
        .bind(notification_type)
        .fetch_optional(db_pool)
        .await
        .ok()?;

        default_result.map(|pref| {
            let channels: Vec<String> = serde_json::from_value(pref.preferred_channels)
                .unwrap_or_else(|_| vec!["EMAIL".to_string()]);
            (channels, pref.language, pref.enabled)
        })
    }

    /// Send high-risk AI action notification (EU AI Act Article 13)
    /// Enhanced version with user preferences, multi-language support, and detailed information
    pub async fn notify_high_risk_ai_action(
        &self,
        db_pool: &PgPool,
        user_id: &str,
        seal_id: &str,
        action_type: &str,
        risk_level: &str,
        timestamp: &DateTime<Utc>,
        purpose: Option<&str>,
        channel: NotificationChannel,
    ) -> Result<String, String> {
        // Get user preferences
        let (preferred_channels, language, enabled) = Self::get_user_preferences(
            db_pool,
            user_id,
            "HIGH_RISK_AI_ACTION",
        ).await.unwrap_or_else(|| {
            // Default preferences if not found
            (vec!["EMAIL".to_string(), "IN_APP".to_string()], "en".to_string(), true)
        });

        // Check if notifications are enabled for this user
        if !enabled {
            return Err("Notifications disabled by user preference".to_string());
        }

        // Determine which channel to use (prefer user preference, fallback to provided channel)
        let channel_to_use = preferred_channels.first()
            .and_then(|ch| match ch.as_str() {
                "EMAIL" => Some(NotificationChannel::Email),
                "SMS" => Some(NotificationChannel::Sms),
                "IN_APP" => Some(NotificationChannel::InApp),
                _ => None,
            })
            .unwrap_or(channel);

        // Enhanced notification content with AI transparency information (EU AI Act Article 13)
        let ai_functioning = format!(
            "The AI system analyzed your data using automated processing algorithms to perform: {}",
            action_type
        );
        let risks_info = match risk_level {
            "HIGH" => "This action involves high-risk processing that may significantly affect your rights. Human oversight has been applied.",
            "MEDIUM" => "This action involves moderate-risk processing. Standard safeguards are in place.",
            _ => "This action involves low-risk processing with standard safeguards.",
        };

        // Load template with user's preferred language
        let (subject_template, body_template) = match Self::load_template(
            db_pool,
            "HIGH_RISK_AI_ACTION",
            &channel_to_use.to_string(),
            &language,
        ).await {
            Some(t) => t,
            None => {
                // Enhanced fallback template with all required information
                (
                    Some(format!("High-Risk AI Action Notification - {}", action_type)),
                    format!(
                        "Dear User,\n\nThis is to inform you that a high-risk AI system has processed your data, as required by EU AI Act Article 13.\n\n=== ACTION DETAILS ===\n- Type: {}\n- Timestamp: {}\n- Risk Level: {}\n- Purpose: {}\n\n=== AI SYSTEM INFORMATION ===\n- How it works: {}\n- Risks: {}\n\n=== YOUR RIGHTS ===\nYou have the right to:\n- Request human review of this decision\n- Object to this processing\n- Request an explanation of the logic involved\n- Lodge a complaint with a supervisory authority\n\nIf you have questions or wish to exercise your rights, please contact us.\n\nBest regards,\nVeridion Nexus Compliance Team",
                        action_type,
                        timestamp.format("%Y-%m-%d %H:%M:%S"),
                        risk_level,
                        purpose.unwrap_or("Not specified"),
                        ai_functioning,
                        risks_info
                    ),
                )
            }
        };

        let subject = subject_template;
        let body = body_template
            .replace("{{action_type}}", action_type)
            .replace("{{timestamp}}", &timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
            .replace("{{risk_level}}", risk_level)
            .replace("{{purpose}}", purpose.unwrap_or("Not specified"))
            .replace("{{ai_functioning}}", &ai_functioning)
            .replace("{{risks}}", risks_info);

        let request = NotificationRequest {
            user_id: user_id.to_string(),
            notification_type: NotificationType::HighRiskAiAction,
            channel: channel_to_use,
            subject,
            body,
            language: Some(language),
            related_entity_type: Some("COMPLIANCE_RECORD".to_string()),
            related_entity_id: Some(seal_id.to_string()),
        };

        // Send to all preferred channels
        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break; // Success, no need to try other channels
            }
        }

        last_result
    }

    /// Send shadow mode violation alert
    /// Alerts when shadow mode detects a violation that would be blocked in enforcing mode
    pub async fn send_shadow_mode_alert(
        &self,
        db_pool: &PgPool,
        agent_id: &str,
        action: &str,
        target_region: &str,
        policy_applied: &str,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        // Get user notification preferences
        let user_id_str = user_id.unwrap_or("system");
        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("Shadow Mode Alert: Violation Detected - {}", policy_applied);
        let body = format!(
            "Shadow Mode has detected a violation that would be blocked in enforcing mode:\n\n\
            Agent ID: {}\n\
            Action: {}\n\
            Target Region: {}\n\
            Policy: {}\n\
            Timestamp: {}\n\n\
            This action was allowed in shadow mode for testing purposes. Review the policy before switching to ENFORCING mode.\n\n\
            View details in the Shadow Mode Analytics dashboard.",
            agent_id,
            action,
            target_region,
            policy_applied,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::ShadowModeViolation,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("SHADOW_MODE_LOG".to_string()),
            related_entity_id: Some(agent_id.to_string()),
        };

        // Send to all preferred channels
        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break; // Success, no need to try other channels
            }
        }

        last_result
    }

    /// Send circuit breaker opened alert
    pub async fn send_circuit_breaker_alert(
        &self,
        db_pool: &PgPool,
        policy_id: &str,
        policy_type: &str,
        error_rate: f64,
        error_count: i64,
        total_requests: i64,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        let user_id_str = user_id.unwrap_or("system");
        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("Circuit Breaker Opened: {} Policy", policy_type);
        let body = format!(
            "Circuit breaker has been automatically opened for policy {}:\n\n\
            Policy Type: {}\n\
            Error Rate: {:.2}%\n\
            Error Count: {}\n\
            Total Requests: {}\n\
            Timestamp: {}\n\n\
            The policy has been temporarily disabled to prevent further errors. Review the policy configuration and error logs before re-enabling.\n\n\
            You can manually close the circuit breaker from the Circuit Breaker dashboard.",
            policy_id,
            policy_type,
            error_rate,
            error_count,
            total_requests,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::CircuitBreakerOpened,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("POLICY_VERSION".to_string()),
            related_entity_id: Some(policy_id.to_string()),
        };

        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break;
            }
        }

        last_result
    }

    /// Send canary promotion alert
    pub async fn send_canary_promotion_alert(
        &self,
        db_pool: &PgPool,
        policy_id: &str,
        policy_type: &str,
        from_percentage: i32,
        to_percentage: i32,
        success_rate: f64,
        total_requests: i64,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        let user_id_str = user_id.unwrap_or("system");
        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("Canary Deployment Promoted: {} Policy", policy_type);
        let body = format!(
            "Canary deployment has been automatically promoted for policy {}:\n\n\
            Policy Type: {}\n\
            From: {}% traffic\n\
            To: {}% traffic\n\
            Success Rate: {:.2}%\n\
            Total Requests: {}\n\
            Timestamp: {}\n\n\
            The policy has been promoted to the next tier based on success metrics. Monitor the deployment to ensure continued success.\n\n\
            You can view canary deployment status in the Canary Deployment dashboard.",
            policy_id,
            policy_type,
            from_percentage,
            to_percentage,
            success_rate,
            total_requests,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::CanaryPromotion,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("POLICY_VERSION".to_string()),
            related_entity_id: Some(policy_id.to_string()),
        };

        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break;
            }
        }

        last_result
    }

    /// Send canary rollback alert
    pub async fn send_canary_rollback_alert(
        &self,
        db_pool: &PgPool,
        policy_id: &str,
        policy_type: &str,
        from_percentage: i32,
        to_percentage: i32,
        success_rate: f64,
        total_requests: i64,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        let user_id_str = user_id.unwrap_or("system");
        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("Canary Deployment Rolled Back: {} Policy", policy_type);
        let body = format!(
            "Canary deployment has been automatically rolled back for policy {}:\n\n\
            Policy Type: {}\n\
            From: {}% traffic\n\
            To: {}% traffic\n\
            Success Rate: {:.2}%\n\
            Total Requests: {}\n\
            Timestamp: {}\n\n\
            The policy has been rolled back to a lower traffic percentage due to low success rate. Review the policy configuration and error logs before attempting to promote again.\n\n\
            You can view canary deployment status in the Canary Deployment dashboard.",
            policy_id,
            policy_type,
            from_percentage,
            to_percentage,
            success_rate,
            total_requests,
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::CanaryRollback,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("POLICY_VERSION".to_string()),
            related_entity_id: Some(policy_id.to_string()),
        };

        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break;
            }
        }

        last_result
    }

    /// Send compliance violation alert
    /// Alerts when a compliance violation is detected (e.g., blocked proxy request)
    pub async fn send_compliance_violation_alert(
        &self,
        db_pool: &PgPool,
        agent_id: &str,
        violation_type: &str,
        violation_reason: &str,
        target_url: Option<&str>,
        policy_name: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        let user_id_str = user_id.unwrap_or("system");
        
        // Rate limiting: Check if we've sent an alert for this agent in the last 5 minutes
        let rate_limit_check: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_notifications
             WHERE related_entity_id = $1
               AND notification_type = 'COMPLIANCE_VIOLATION'
               AND created_at > NOW() - INTERVAL '5 minutes'
               AND status = 'SENT'"
        )
        .bind(agent_id)
        .fetch_optional(db_pool)
        .await
        .ok()
        .flatten();

        if rate_limit_check.unwrap_or(0) > 0 {
            return Err("Rate limit: Alert already sent for this agent in the last 5 minutes".to_string());
        }

        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("Compliance Violation Alert: {}", violation_type);
        let mut body = format!(
            "A compliance violation has been detected:\n\n\
            Violation Type: {}\n\
            Reason: {}\n\
            Agent ID: {}\n",
            violation_type,
            violation_reason,
            agent_id
        );

        if let Some(url) = target_url {
            body.push_str(&format!("Target URL: {}\n", url));
        }

        if let Some(policy) = policy_name {
            body.push_str(&format!("Policy: {}\n", policy));
        }

        body.push_str(&format!(
            "Timestamp: {}\n\n\
            This violation was blocked to maintain compliance. Review the policy configuration and agent behavior.\n\n\
            View details in the Compliance Dashboard.",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::ComplianceViolation,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("COMPLIANCE_RECORD".to_string()),
            related_entity_id: Some(agent_id.to_string()),
        };

        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break;
            }
        }

        last_result
    }

    /// Send policy health degraded alert
    /// Alerts when a policy's health status degrades (error rate >= 5%)
    pub async fn send_policy_health_degraded_alert(
        &self,
        db_pool: &PgPool,
        policy_id: &str,
        policy_name: &str,
        policy_type: &str,
        error_rate: f64,
        success_rate: f64,
        total_requests: i64,
        avg_latency_ms: Option<f64>,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        let user_id_str = user_id.unwrap_or("system");
        
        // Rate limiting: Check if we've sent a degraded alert for this policy in the last 15 minutes
        let rate_limit_check: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_notifications
             WHERE related_entity_id = $1
               AND notification_type = 'POLICY_HEALTH_DEGRADED'
               AND created_at > NOW() - INTERVAL '15 minutes'
               AND status = 'SENT'"
        )
        .bind(policy_id)
        .fetch_optional(db_pool)
        .await
        .ok()
        .flatten();

        if rate_limit_check.unwrap_or(0) > 0 {
            return Err("Rate limit: Alert already sent for this policy in the last 15 minutes".to_string());
        }

        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("Policy Health Degraded: {}", policy_name);
        let mut body = format!(
            "Policy health has degraded below acceptable thresholds:\n\n\
            Policy: {} ({})\n\
            Policy Type: {}\n\
            Error Rate: {:.2}%\n\
            Success Rate: {:.2}%\n\
            Total Requests: {}\n",
            policy_name,
            policy_id,
            policy_type,
            error_rate,
            success_rate,
            total_requests
        );

        if let Some(latency) = avg_latency_ms {
            body.push_str(&format!("Average Latency: {:.2} ms\n", latency));
        }

        body.push_str(&format!(
            "Timestamp: {}\n\n\
            The policy is experiencing elevated error rates. Review the policy configuration, error logs, and consider:\n\
            - Checking circuit breaker status\n\
            - Reviewing recent policy changes\n\
            - Analyzing error patterns\n\
            - Adjusting policy thresholds if needed\n\n\
            View policy health details in the Policy Health Dashboard.",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::PolicyHealthDegraded,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("POLICY_VERSION".to_string()),
            related_entity_id: Some(policy_id.to_string()),
        };

        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break;
            }
        }

        last_result
    }

    /// Send policy health critical alert
    /// Alerts when a policy's health status becomes critical (error rate >= 10% or circuit breaker OPEN)
    pub async fn send_policy_health_critical_alert(
        &self,
        db_pool: &PgPool,
        policy_id: &str,
        policy_name: &str,
        policy_type: &str,
        error_rate: f64,
        success_rate: f64,
        total_requests: i64,
        avg_latency_ms: Option<f64>,
        circuit_breaker_state: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<String, String> {
        let user_id_str = user_id.unwrap_or("system");
        
        // Rate limiting: Check if we've sent a critical alert for this policy in the last 30 minutes
        let rate_limit_check: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_notifications
             WHERE related_entity_id = $1
               AND notification_type = 'POLICY_HEALTH_CRITICAL'
               AND created_at > NOW() - INTERVAL '30 minutes'
               AND status = 'SENT'"
        )
        .bind(policy_id)
        .fetch_optional(db_pool)
        .await
        .ok()
        .flatten();

        if rate_limit_check.unwrap_or(0) > 0 {
            return Err("Rate limit: Alert already sent for this policy in the last 30 minutes".to_string());
        }

        let preferred_channels: Vec<String> = sqlx::query_scalar(
            "SELECT channel FROM user_notification_preferences 
             WHERE user_id::text = $1 AND enabled = true
             UNION SELECT 'EMAIL' WHERE NOT EXISTS (
                 SELECT 1 FROM user_notification_preferences WHERE user_id::text = $1
             )"
        )
        .bind(user_id_str)
        .fetch_all(db_pool)
        .await
        .unwrap_or_else(|_| vec!["EMAIL".to_string()]);

        let channel_to_use = if preferred_channels.contains(&"EMAIL".to_string()) {
            NotificationChannel::Email
        } else if preferred_channels.contains(&"SMS".to_string()) {
            NotificationChannel::Sms
        } else {
            NotificationChannel::InApp
        };

        let subject = format!("🚨 CRITICAL: Policy Health Alert - {}", policy_name);
        let mut body = format!(
            "⚠️ CRITICAL: Policy health has reached critical levels requiring immediate attention:\n\n\
            Policy: {} ({})\n\
            Policy Type: {}\n\
            Error Rate: {:.2}%\n\
            Success Rate: {:.2}%\n\
            Total Requests: {}\n",
            policy_name,
            policy_id,
            policy_type,
            error_rate,
            success_rate,
            total_requests
        );

        if let Some(latency) = avg_latency_ms {
            body.push_str(&format!("Average Latency: {:.2} ms\n", latency));
        }

        if let Some(cb_state) = circuit_breaker_state {
            body.push_str(&format!("Circuit Breaker State: {}\n", cb_state));
        }

        body.push_str(&format!(
            "Timestamp: {}\n\n\
            ⚠️ IMMEDIATE ACTION REQUIRED:\n\
            - The policy is experiencing critical error rates\n\
            - Circuit breaker may be OPEN, disabling the policy\n\
            - Review error logs immediately\n\
            - Consider rolling back recent policy changes\n\
            - Check system health and dependencies\n\n\
            View policy health details in the Policy Health Dashboard.\n\
            Circuit Breaker controls are available in the Circuit Breaker Dashboard.",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));

        let request = NotificationRequest {
            user_id: user_id_str.to_string(),
            notification_type: NotificationType::PolicyHealthCritical,
            channel: channel_to_use,
            subject: Some(subject),
            body,
            language: Some("en".to_string()),
            related_entity_type: Some("POLICY_VERSION".to_string()),
            related_entity_id: Some(policy_id.to_string()),
        };

        let mut last_result = Err("No channels available".to_string());
        for ch_str in &preferred_channels {
            let ch = match ch_str.as_str() {
                "EMAIL" => NotificationChannel::Email,
                "SMS" => NotificationChannel::Sms,
                "IN_APP" => NotificationChannel::InApp,
                _ => continue,
            };

            let mut req = request.clone();
            req.channel = ch;
            last_result = self.send_notification(db_pool, req).await;
            if last_result.is_ok() {
                break;
            }
        }

        last_result
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

