// Integration Layer
// SDKs, Webhooks, and API integrations

pub mod webhooks;
pub mod proxy;

// Re-export for convenience
pub use webhooks::WebhookService;
pub use proxy::{ProxyMiddleware, ProxyConfig, create_proxy_middleware};

