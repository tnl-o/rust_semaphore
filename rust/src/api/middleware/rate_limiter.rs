//! Rate Limiting Middleware
//!
//! Защита от злоупотреблений и DDoS
//!
//! Конфигурация через переменные окружения:
//! - VELUM_RATE_LIMIT_MAX_REQUESTS (по умолчанию: 100)
//! - VELUM_RATE_LIMIT_PERIOD_SECS (по умолчанию: 60)
//! - VELUM_RATE_LIMIT_AUTH_MAX_REQUESTS (по умолчанию: 5)
//! - VELUM_RATE_LIMIT_AUTH_PERIOD_SECS (по умолчанию: 60)

use axum::{
    extract::State,
    http::{Request, StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
    body::Body,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Конфигурация rate limiter
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u64,
    pub period_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        // Чтение из переменных окружения
        let max_requests = std::env::var("VELUM_RATE_LIMIT_MAX_REQUESTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        
        let period_secs = std::env::var("VELUM_RATE_LIMIT_PERIOD_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            max_requests,
            period_secs,
        }
    }
}

impl RateLimitConfig {
    /// Конфигурация для API endpoints
    pub fn for_api() -> Self {
        Self::default()
    }

    /// Конфигурация для auth endpoints (более строгая)
    pub fn for_auth() -> Self {
        let max_requests = std::env::var("VELUM_RATE_LIMIT_AUTH_MAX_REQUESTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        
        let period_secs = std::env::var("VELUM_RATE_LIMIT_AUTH_PERIOD_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            max_requests,
            period_secs,
        }
    }
}

#[derive(Debug, Clone)]
struct ClientInfo {
    requests: u64,
    period_start: Instant,
}

/// Результат проверки rate limiter
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u64,
    pub reset_at: Duration,
}

/// Rate Limiter
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(RateLimitConfig::default())
    }

    pub fn for_api() -> Self {
        Self::new(RateLimitConfig::for_api())
    }

    pub fn for_auth() -> Self {
        Self::new(RateLimitConfig::for_auth())
    }

    pub async fn check(&self, key: &str) -> RateLimitResult {
        let now = Instant::now();
        let period = Duration::from_secs(self.config.period_secs);

        let mut clients = self.clients.write().await;
        let client = clients.entry(key.to_string()).or_insert_with(|| ClientInfo {
            requests: 0,
            period_start: now,
        });

        // Проверка периода
        if now.duration_since(client.period_start) > period {
            client.requests = 0;
            client.period_start = now;
        }

        let remaining = if client.requests >= self.config.max_requests {
            0
        } else {
            self.config.max_requests - client.requests
        };

        let reset_at = period - now.duration_since(client.period_start);

        // Проверка лимита
        if client.requests >= self.config.max_requests {
            return RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at,
            };
        }

        client.requests += 1;
        RateLimitResult {
            allowed: true,
            remaining,
            reset_at,
        }
    }

    pub async fn is_allowed(&self, key: &str) -> bool {
        self.check(key).await.allowed
    }
}

/// State для middleware
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Arc<RateLimiter>,
}

fn extract_ip<B>(req: &Request<B>) -> String {
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    
    if let Some(addr) = req.extensions().get::<std::net::SocketAddr>() {
        return addr.ip().to_string();
    }
    
    "unknown".to_string()
}

/// Middleware функция для API rate limiting
pub async fn api_rate_limit(
    State(state): State<RateLimitState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_ip(&req);
    let result = state.limiter.check(&ip).await;

    // Добавляем заголовки rate limiting
    let response = next.run(req).await;
    let (mut parts, body) = response.into_parts();

    parts.headers.insert(
        "X-RateLimit-Limit",
        state.limiter.config.max_requests.to_string().parse().unwrap()
    );
    parts.headers.insert(
        "X-RateLimit-Remaining",
        result.remaining.to_string().parse().unwrap()
    );
    parts.headers.insert(
        "X-RateLimit-Reset",
        result.reset_at.as_secs().to_string().parse().unwrap()
    );

    Ok(Response::from_parts(parts, body))
}

/// Middleware функция для auth rate limiting с заголовками
pub async fn auth_rate_limit(
    State(state): State<RateLimitState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_ip(&req);
    let result = state.limiter.check(&ip).await;

    if !result.allowed {
        let mut response = Response::new(Body::from("Too Many Requests"));
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        response.headers_mut().insert(
            "X-RateLimit-Limit",
            state.limiter.config.max_requests.to_string().parse().unwrap()
        );
        response.headers_mut().insert(
            "X-RateLimit-Remaining",
            "0".parse().unwrap()
        );
        response.headers_mut().insert(
            "X-RateLimit-Reset",
            result.reset_at.as_secs().to_string().parse().unwrap()
        );
        response.headers_mut().insert(
            "Retry-After",
            result.reset_at.as_secs().to_string().parse().unwrap()
        );
        return Ok(response);
    }

    let response = next.run(req).await;
    let (mut parts, body) = response.into_parts();

    parts.headers.insert(
        "X-RateLimit-Limit",
        state.limiter.config.max_requests.to_string().parse().unwrap()
    );
    parts.headers.insert(
        "X-RateLimit-Remaining",
        result.remaining.to_string().parse().unwrap()
    );
    parts.headers.insert(
        "X-RateLimit-Reset",
        result.reset_at.as_secs().to_string().parse().unwrap()
    );

    Ok(Response::from_parts(parts, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 5,
            period_secs: 60,
        });
        
        for i in 0..5 {
            assert!(limiter.is_allowed("test").await, "Request {} should be allowed", i);
        }
        
        assert!(!limiter.is_allowed("test").await, "Request 6 should be blocked");
    }
}
