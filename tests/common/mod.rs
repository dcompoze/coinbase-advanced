//! Shared helpers for integration tests.

#![allow(dead_code)]

use coinbase_advanced::{Credentials, RestClient};

/// Throwaway EC key generated for tests only.
pub const TEST_EC_KEY: &str = "-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIFRQqrwlq7sCUJ56eM3bLnEQxtWNkOr9lA6oaQ/0sKfLoAoGCCqGSM49
AwEHoUQDQgAEat2hFxJwUbhH4oZp9z5rj7J6nU7FYt6pfE6Ei3gvMWAZIqJ8TdME
S5IRIotaS4KLpQhofOyNZ7i7rcCAipIZrw==
-----END EC PRIVATE KEY-----";

/// Build an authenticated client pointed at a mock server.
pub fn mock_client(uri: &str) -> RestClient {
    RestClient::builder()
        .credentials(Credentials::new("test-key", TEST_EC_KEY).unwrap())
        .base_url(uri)
        .build()
        .unwrap()
}

/// A valid account response body.
pub fn account_json(uuid: &str) -> serde_json::Value {
    serde_json::json!({
        "uuid": uuid,
        "name": "BTC Wallet",
        "currency": "BTC",
        "available_balance": {"value": "1.0", "currency": "BTC"},
        "default": true,
        "active": true,
        "created_at": "2024-01-15T12:00:00Z",
        "updated_at": "2024-01-15T12:00:00Z",
        "deleted_at": null,
        "type": "ACCOUNT_TYPE_CRYPTO",
        "ready": true,
        "hold": {"value": "0", "currency": "BTC"},
        "retail_portfolio_id": null
    })
}

/// A valid order response body.
pub fn order_json(order_id: &str) -> serde_json::Value {
    serde_json::json!({
        "order_id": order_id,
        "product_id": "BTC-USD",
        "side": "BUY",
        "client_order_id": "client-1",
        "status": "OPEN"
    })
}

/// A valid fill response body.
pub fn fill_json(entry_id: &str) -> serde_json::Value {
    serde_json::json!({
        "entry_id": entry_id,
        "trade_id": "trade-1",
        "order_id": "order-1",
        "trade_time": "2024-01-15T12:00:00Z",
        "trade_type": "FILL",
        "price": "50000",
        "size": "0.001",
        "commission": "0.05",
        "product_id": "BTC-USD"
    })
}

/// A valid candle response body for the given start time.
pub fn candle_json(start: u64) -> serde_json::Value {
    serde_json::json!({
        "start": start.to_string(),
        "low": "1",
        "high": "2",
        "open": "1",
        "close": "2",
        "volume": "10"
    })
}
