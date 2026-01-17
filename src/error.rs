use std::time::Duration;

/// Result type alias for coinbase-client operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the Coinbase client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing credentials, invalid format, etc.)
    #[error("Configuration error: {0}")]
    Config(String),

    /// JWT signing error
    #[error("JWT error: {0}")]
    Jwt(String),

    /// HTTP transport error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Request building error
    #[error("Request error: {0}")]
    Request(String),

    /// API error response from Coinbase
    #[error("API error: {message}")]
    Api {
        /// Error message from the API
        message: String,
        /// HTTP status code
        status: u16,
        /// Raw error response body
        body: Option<String>,
    },

    /// Rate limit exceeded
    #[error("Rate limited, retry after {retry_after:?}")]
    RateLimited {
        /// Duration to wait before retrying
        retry_after: Option<Duration>,
    },

    /// Response parsing error
    #[error("Parse error: {message}")]
    Parse {
        /// Description of the parse error
        message: String,
        /// Raw response body that failed to parse
        body: Option<String>,
    },

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// URL construction error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

impl Error {
    /// Create a new configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new JWT error.
    pub fn jwt(msg: impl Into<String>) -> Self {
        Self::Jwt(msg.into())
    }

    /// Create a new request error.
    pub fn request(msg: impl Into<String>) -> Self {
        Self::Request(msg.into())
    }

    /// Create a new API error.
    pub fn api(status: u16, message: impl Into<String>, body: Option<String>) -> Self {
        Self::Api {
            message: message.into(),
            status,
            body,
        }
    }

    /// Create a new parse error.
    pub fn parse(message: impl Into<String>, body: Option<String>) -> Self {
        Self::Parse {
            message: message.into(),
            body,
        }
    }

    /// Create a new authentication error.
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create a new WebSocket error.
    pub fn websocket(msg: impl Into<String>) -> Self {
        Self::WebSocket(msg.into())
    }

    /// Check if this error is a rate limit error.
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. } => true,
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            Self::Api { status, .. } => *status >= 500,
            _ => false,
        }
    }
}
