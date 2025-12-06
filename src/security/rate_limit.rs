use actix_web::{dev::ServiceRequest, Error};
use actix_web::dev::{ServiceResponse, Transform};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use futures::future::{ok, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Future;

/// Rate limit configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            window_seconds: 60,
        }
    }
}

/// Rate limit entry
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    store: Arc<DashMap<String, RateLimitEntry>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn check(&self, identifier: &str) -> Result<(), String> {
        let now = Instant::now();
        let key = identifier.to_string();

        // Clean up old entries periodically (simple cleanup)
        if self.store.len() > 10000 {
            self.store.retain(|_, entry| {
                now.duration_since(entry.window_start).as_secs() < self.config.window_seconds
            });
        }

        let mut entry = self.store.entry(key.clone()).or_insert_with(|| {
            RateLimitEntry {
                count: 0,
                window_start: now,
            }
        });

        // Reset window if expired
        if now.duration_since(entry.window_start).as_secs() >= self.config.window_seconds {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check limit
        if entry.count >= self.config.requests_per_minute {
            let retry_after = self.config.window_seconds
                - now.duration_since(entry.window_start).as_secs();
            return Err(format!("Rate limit exceeded. Retry after {} seconds", retry_after));
        }

        entry.count += 1;
        Ok(())
    }
}

/// Rate limiting middleware
#[derive(Clone)]
pub struct RateLimit {
    limiter: RateLimiter,
}

impl RateLimit {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: RateLimiter::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service,
            limiter: self.limiter.clone(),
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
    limiter: RateLimiter,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Get identifier (IP address or user ID)
        let identifier = req
            .connection_info()
            .peer_addr()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Check rate limit
        match self.limiter.check(&identifier) {
            Ok(_) => {
                let fut = self.service.call(req);
                Box::pin(async move { fut.await })
            }
            Err(_error_msg) => {
                // Return error directly - actix-web will handle the conversion
                Box::pin(async move {
                    Err(Error::from(actix_web::error::ErrorTooManyRequests(
                        "Rate limit exceeded. Please try again later."
                    )))
                })
            }
        }
    }
}
