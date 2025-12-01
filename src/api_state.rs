use crate::crypto_shredder::VeridionKeyStore;
use crate::privacy_bridge::SignicatClient;
use crate::annex_iv_compiler::ComplianceRecord;
use std::sync::Mutex;

/// Application state shared across all request handlers
pub struct AppState {
    /// Key store for crypto-shredding (GDPR compliance)
    pub key_store: Mutex<VeridionKeyStore>,
    /// Signicat client for eIDAS sealing
    pub signicat: SignicatClient,
    /// In-memory compliance log for Annex IV documentation
    pub compliance_log: Mutex<Vec<ComplianceRecord>>,
    /// Lockdown flag - when true, all new agent actions are blocked
    pub is_locked_down: Mutex<bool>,
}

impl AppState {
    /// Create a new AppState with initialized components
    pub fn new() -> Self {
        Self {
            key_store: Mutex::new(VeridionKeyStore::new()),
            signicat: SignicatClient::new(),
            compliance_log: Mutex::new(Vec::new()),
            is_locked_down: Mutex::new(false),
        }
    }
}

