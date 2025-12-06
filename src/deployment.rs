// Deployment Modes Configuration
// Supports three deployment modes: Embedded, Proxy, Full

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentMode {
    /// Embedded Mode: SDK-first, lightweight client library
    Embedded,
    /// Proxy Mode: Reverse proxy middleware
    Proxy,
    /// Full Governance Mode: Complete platform
    Full,
}

impl Default for DeploymentMode {
    fn default() -> Self {
        DeploymentMode::Full
    }
}

impl DeploymentMode {
    /// Get deployment mode from environment variable
    pub fn from_env() -> Self {
        std::env::var("DEPLOYMENT_MODE")
            .unwrap_or_else(|_| "full".to_string())
            .to_lowercase()
            .as_str()
            .into()
    }

    /// Check if a feature is available in this deployment mode
    #[allow(dead_code)]
    pub fn has_feature(&self, feature: &str) -> bool {
        match self {
            DeploymentMode::Embedded => {
                // Embedded mode: Only core features + SDKs
                matches!(feature, "core" | "sdks" | "api")
            }
            DeploymentMode::Proxy => {
                // Proxy mode: Core + Proxy + Webhooks
                matches!(feature, "core" | "proxy" | "webhooks" | "api")
            }
            DeploymentMode::Full => {
                // Full mode: Everything
                true
            }
        }
    }

    /// Get available modules for this deployment mode
    #[allow(dead_code)]
    pub fn available_modules(&self) -> Vec<&str> {
        match self {
            DeploymentMode::Embedded => vec!["core"],
            DeploymentMode::Proxy => vec!["core", "proxy", "webhooks"],
            DeploymentMode::Full => vec![
                "core",
                "data_subject_rights",
                "human_oversight",
                "risk_assessment",
                "breach_management",
                "consent",
                "dpia",
                "retention",
                "monitoring",
                "green_ai",
                "ai_bom",
                "webhooks",
                "proxy",
            ],
        }
    }
}

impl From<&str> for DeploymentMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "embedded" => DeploymentMode::Embedded,
            "proxy" => DeploymentMode::Proxy,
            "full" => DeploymentMode::Full,
            _ => DeploymentMode::Full,
        }
    }
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub mode: DeploymentMode,
    pub proxy_target_url: Option<String>,
    pub enable_dashboard: bool,
    pub enable_all_modules: bool,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            mode: DeploymentMode::from_env(),
            proxy_target_url: std::env::var("PROXY_TARGET_URL").ok(),
            enable_dashboard: std::env::var("ENABLE_DASHBOARD")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_all_modules: std::env::var("ENABLE_ALL_MODULES")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }
}

