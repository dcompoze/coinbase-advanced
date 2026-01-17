//! Rate limiting implementation using a token bucket algorithm.
//!
//! This module provides client-side rate limiting to avoid hitting Coinbase API limits
//! and to gracefully handle rate limit responses.
//!
//! # Rate Limits
//!
//! According to Coinbase documentation:
//! - Public REST endpoints: ~10 requests/second
//! - Private REST endpoints: ~30 requests/second
//! - WebSocket: ~750 messages/second (authenticated)
//!
//! # Usage
//!
//! Rate limiting is automatically applied by the RestClient when enabled.
//! ```no_run
//! use coinbase_client::{RestClient, Credentials};
//!
//! let client = RestClient::builder()
//!     .credentials(Credentials::from_env().unwrap())
//!     .rate_limiting(true)
//!     .build()
//!     .unwrap();
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Default rate limits based on Coinbase API documentation.
pub mod limits {
    /// Public REST API refresh rate (requests per second).
    pub const PUBLIC_REST_RATE: f64 = 10.0;
    /// Private (authenticated) REST API refresh rate (requests per second).
    pub const PRIVATE_REST_RATE: f64 = 30.0;
    /// Public WebSocket refresh rate (messages per second).
    pub const PUBLIC_WS_RATE: f64 = 8.0;
    /// Private WebSocket refresh rate (messages per second).
    pub const PRIVATE_WS_RATE: f64 = 750.0;
}

/// A token bucket rate limiter.
///
/// Implements the token bucket algorithm for rate limiting. Tokens are added to the
/// bucket at a fixed rate, up to a maximum capacity. Each request consumes one token.
/// If no tokens are available, the caller can wait until a token becomes available.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum number of tokens in the bucket.
    max_tokens: f64,
    /// Number of tokens added per second.
    refill_rate: f64,
    /// Current number of tokens.
    tokens: f64,
    /// Time of last token consumption/refill.
    last_update: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the specified maximum tokens and refill rate.
    ///
    /// # Arguments
    ///
    /// * `max_tokens` - Maximum number of tokens the bucket can hold.
    /// * `refill_rate` - Number of tokens to add per second.
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            max_tokens,
            refill_rate,
            tokens: max_tokens,
            last_update: Instant::now(),
        }
    }

    /// Create a token bucket configured for public REST API requests.
    pub fn for_public_rest() -> Self {
        Self::new(limits::PUBLIC_REST_RATE, limits::PUBLIC_REST_RATE)
    }

    /// Create a token bucket configured for private REST API requests.
    pub fn for_private_rest() -> Self {
        Self::new(limits::PRIVATE_REST_RATE, limits::PRIVATE_REST_RATE)
    }

    /// Create a token bucket configured for public WebSocket messages.
    pub fn for_public_ws() -> Self {
        Self::new(limits::PUBLIC_WS_RATE, limits::PUBLIC_WS_RATE)
    }

    /// Create a token bucket configured for private WebSocket messages.
    pub fn for_private_ws() -> Self {
        Self::new(limits::PRIVATE_WS_RATE, limits::PRIVATE_WS_RATE)
    }

    /// Refill tokens based on elapsed time since last update.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
        self.last_update = now;
    }

    /// Try to consume a token. Returns true if successful, false if no tokens available.
    pub fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get the time until the next token is available.
    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64((1.0 - self.tokens) / self.refill_rate)
        }
    }

    /// Wait until a token is available and consume it.
    pub async fn wait_and_consume(&mut self) {
        while !self.try_consume() {
            let wait_time = self.time_until_available();
            tokio::time::sleep(wait_time).await;
        }
    }

    /// Get the current number of available tokens.
    pub fn available_tokens(&self) -> f64 {
        self.tokens
    }
}

/// A thread-safe rate limiter that can be shared across async tasks.
#[derive(Clone)]
pub struct RateLimiter {
    bucket: Arc<Mutex<TokenBucket>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given token bucket configuration.
    pub fn new(bucket: TokenBucket) -> Self {
        Self {
            bucket: Arc::new(Mutex::new(bucket)),
        }
    }

    /// Create a rate limiter configured for public REST API requests.
    pub fn for_public_rest() -> Self {
        Self::new(TokenBucket::for_public_rest())
    }

    /// Create a rate limiter configured for private REST API requests.
    pub fn for_private_rest() -> Self {
        Self::new(TokenBucket::for_private_rest())
    }

    /// Try to acquire a token without waiting.
    pub async fn try_acquire(&self) -> bool {
        let mut bucket = self.bucket.lock().await;
        bucket.try_consume()
    }

    /// Wait until a token is available and acquire it.
    pub async fn acquire(&self) {
        let mut bucket = self.bucket.lock().await;
        bucket.wait_and_consume().await;
    }

    /// Get the current number of available tokens.
    pub async fn available(&self) -> f64 {
        let mut bucket = self.bucket.lock().await;
        bucket.refill();
        bucket.available_tokens()
    }
}

/// Configuration for rate limiting behavior.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled.
    pub enabled: bool,
    /// Maximum number of retry attempts for rate-limited requests.
    pub max_retries: u32,
    /// Initial backoff duration for retries.
    pub initial_backoff: Duration,
    /// Maximum backoff duration for retries.
    pub max_backoff: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable rate limiting.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Set the maximum number of retries.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the initial backoff duration.
    pub fn with_initial_backoff(mut self, duration: Duration) -> Self {
        self.initial_backoff = duration;
        self
    }

    /// Set the maximum backoff duration.
    pub fn with_max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }
}

/// Parsed rate limit information from response headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Maximum number of requests allowed.
    pub limit: Option<u32>,
    /// Number of requests remaining.
    pub remaining: Option<u32>,
    /// Time when the rate limit resets (Unix timestamp).
    pub reset: Option<u64>,
}

impl RateLimitInfo {
    /// Parse rate limit headers from a response.
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        Self {
            limit: headers
                .get("x-ratelimit-limit")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            remaining: headers
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            reset: headers
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
        }
    }

    /// Check if rate limit is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.remaining == Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_new() {
        let bucket = TokenBucket::new(10.0, 5.0);
        assert_eq!(bucket.max_tokens, 10.0);
        assert_eq!(bucket.refill_rate, 5.0);
        assert_eq!(bucket.tokens, 10.0);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(5.0, 1.0);

        // Should be able to consume 5 tokens
        for _ in 0..5 {
            assert!(bucket.try_consume());
        }

        // 6th token should fail
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_rate_limit_config_disabled() {
        let config = RateLimitConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_rate_limit_info_parse() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-ratelimit-limit", "100".parse().unwrap());
        headers.insert("x-ratelimit-remaining", "50".parse().unwrap());
        headers.insert("x-ratelimit-reset", "1234567890".parse().unwrap());

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.limit, Some(100));
        assert_eq!(info.remaining, Some(50));
        assert_eq!(info.reset, Some(1234567890));
    }

    #[test]
    fn test_rate_limit_info_exhausted() {
        let info = RateLimitInfo {
            limit: Some(100),
            remaining: Some(0),
            reset: Some(1234567890),
        };
        assert!(info.is_exhausted());

        let info2 = RateLimitInfo {
            limit: Some(100),
            remaining: Some(50),
            reset: Some(1234567890),
        };
        assert!(!info2.is_exhausted());
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(TokenBucket::new(2.0, 10.0));

        // Should be able to acquire 2 tokens
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);

        // Third should fail immediately
        assert!(!limiter.try_acquire().await);
    }
}
