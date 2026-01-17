/// Production API base URL.
pub const API_BASE_URL: &str = "https://api.coinbase.com";

/// Sandbox API base URL.
pub const API_SANDBOX_BASE_URL: &str = "https://api-sandbox.coinbase.com";

/// API version path prefix.
pub const API_PATH_PREFIX: &str = "/api/v3/brokerage";

/// WebSocket production URL.
pub const WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

/// WebSocket sandbox URL.
pub const WS_SANDBOX_URL: &str = "wss://advanced-trade-ws-sandbox.coinbase.com";

/// User agent string for HTTP requests.
pub const USER_AGENT: &str = concat!("coinbase-client-rust/", env!("CARGO_PKG_VERSION"));

/// JWT algorithm.
pub const JWT_ALGORITHM: &str = "ES256";

/// JWT issuer claim.
pub const JWT_ISSUER: &str = "cdp";

/// JWT expiration time in seconds.
pub const JWT_EXPIRY_SECONDS: u64 = 120;

/// Default request timeout in seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
