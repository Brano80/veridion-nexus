// Core Runtime Compliance Engine
// These modules are mandatory and always enabled

pub mod sovereign_lock;
pub mod crypto_shredder;
pub mod privacy_bridge;
pub mod annex_iv;

// Re-export for convenience
pub use crypto_shredder::VeridionKeyStore;
pub use privacy_bridge::SignicatClient;
pub use annex_iv::ComplianceRecord;

