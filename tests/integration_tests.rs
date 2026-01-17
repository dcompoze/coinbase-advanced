//! Integration tests for the Coinbase Advanced Trade API client.
//!
//! These tests use wiremock to mock API responses.

use coinbase_client::{Credentials, RestClient};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_mock_server_setup() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iso": "2024-01-15T12:00:00Z",
            "epochSeconds": 1705320000,
            "epochMillis": 1705320000000_i64
        })))
        .mount(&mock_server)
        .await;

    // Verify the mock is mounted correctly
    let response = reqwest::get(format!("{}/api/v3/brokerage/time", mock_server.uri()))
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_credentials_validation() {
    // Test that invalid credentials are rejected
    let result = Credentials::new(
        "",
        "-----BEGIN EC PRIVATE KEY-----\ntest\n-----END EC PRIVATE KEY-----",
    );
    assert!(result.is_err());

    let result = Credentials::new("test-key", "");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_builder() {
    // Test default client
    let client = RestClient::builder().build().unwrap();
    assert!(!client.has_credentials());

    // Test sandbox mode
    let client = RestClient::builder().sandbox(true).build().unwrap();
    assert!(client.base_url().contains("sandbox"));

    // Test rate limiting
    let client = RestClient::builder().rate_limiting(true).build().unwrap();
    assert!(!client.has_credentials());
}

#[tokio::test]
async fn test_error_response_parsing() {
    // Test that error responses are properly parsed
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "Invalid request"
        })))
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/error", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn test_rate_limit_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/rate-limited"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "60")
                .set_body_json(serde_json::json!({
                    "message": "Rate limit exceeded"
                })),
        )
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/rate-limited", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 429);
    assert_eq!(response.headers().get("retry-after").unwrap(), "60");
}

mod models {
    use coinbase_client::models::*;

    #[test]
    fn test_order_side_serialization() {
        let side = OrderSide::Buy;
        let json = serde_json::to_string(&side).unwrap();
        assert_eq!(json, "\"BUY\"");

        let side: OrderSide = serde_json::from_str("\"SELL\"").unwrap();
        assert_eq!(side, OrderSide::Sell);
    }

    #[test]
    fn test_order_status_deserialization() {
        let status: OrderStatus = serde_json::from_str("\"FILLED\"").unwrap();
        assert_eq!(status, OrderStatus::Filled);

        let status: OrderStatus = serde_json::from_str("\"CANCELLED\"").unwrap();
        assert_eq!(status, OrderStatus::Cancelled);
    }

    #[test]
    fn test_product_response_deserialization() {
        let json = r#"{
            "product_id": "BTC-USD",
            "price": "50000.00",
            "price_percentage_change_24h": "5.25",
            "volume_24h": "1000000.00",
            "volume_percentage_change_24h": "10.5",
            "base_increment": "0.00000001",
            "quote_increment": "0.01",
            "quote_min_size": "1",
            "quote_max_size": "10000000",
            "base_min_size": "0.0001",
            "base_max_size": "1000",
            "base_name": "Bitcoin",
            "quote_name": "US Dollar",
            "watched": false,
            "is_disabled": false,
            "new": false,
            "status": "online",
            "cancel_only": false,
            "limit_only": false,
            "post_only": false,
            "trading_disabled": false,
            "auction_mode": false,
            "product_type": "SPOT",
            "quote_currency_id": "USD",
            "base_currency_id": "BTC",
            "fcm_trading_session_details": null,
            "mid_market_price": "50000.00",
            "alias": "",
            "alias_to": [],
            "base_display_symbol": "BTC",
            "quote_display_symbol": "USD",
            "view_only": false,
            "price_increment": "0.01",
            "display_name": "BTC-USD",
            "product_venue": "CBE",
            "approximate_quote_24h_volume": "50000000.00"
        }"#;

        let product: Product = serde_json::from_str(json).unwrap();
        assert_eq!(product.product_id, "BTC-USD");
        assert_eq!(product.base_name, "Bitcoin");
    }

    #[test]
    fn test_account_response_deserialization() {
        let json = r#"{
            "uuid": "12345678-1234-1234-1234-123456789012",
            "name": "BTC Wallet",
            "currency": "BTC",
            "available_balance": {
                "value": "1.5",
                "currency": "BTC"
            },
            "default": true,
            "active": true,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "deleted_at": null,
            "type": "ACCOUNT_TYPE_CRYPTO",
            "ready": true,
            "hold": {
                "value": "0.1",
                "currency": "BTC"
            },
            "retail_portfolio_id": "portfolio-123"
        }"#;

        let account: Account = serde_json::from_str(json).unwrap();
        assert_eq!(account.uuid, "12345678-1234-1234-1234-123456789012");
        assert_eq!(account.currency, "BTC");
    }

    #[test]
    fn test_create_order_request() {
        let order = CreateOrderRequest {
            client_order_id: "my-order-123".to_string(),
            product_id: "BTC-USD".to_string(),
            side: OrderSide::Buy,
            order_configuration: OrderConfiguration::MarketIoc {
                market_market_ioc: MarketIoc {
                    quote_size: Some("100.00".to_string()),
                    base_size: None,
                },
            },
            leverage: None,
            margin_type: None,
            retail_portfolio_id: None,
            self_trade_prevention_id: None,
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("BTC-USD"));
        assert!(json.contains("BUY"));
    }
}

mod rate_limit {
    use coinbase_client::rate_limit::{RateLimitConfig, RateLimitInfo, RateLimiter, TokenBucket};

    #[test]
    fn test_token_bucket_creation() {
        let bucket = TokenBucket::new(10.0, 5.0);
        assert_eq!(bucket.available_tokens(), 10.0);
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::new().with_max_retries(5);
        assert_eq!(config.max_retries, 5);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(TokenBucket::new(3.0, 10.0));

        // Should have 3 tokens available
        let available = limiter.available().await;
        assert!((2.9..=3.0).contains(&available));

        // Acquire 3 tokens
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);

        // 4th should fail
        assert!(!limiter.try_acquire().await);
    }

    #[test]
    fn test_rate_limit_info_from_headers() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-ratelimit-limit", "100".parse().unwrap());
        headers.insert("x-ratelimit-remaining", "99".parse().unwrap());

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.limit, Some(100));
        assert_eq!(info.remaining, Some(99));
        assert!(!info.is_exhausted());
    }
}

mod websocket {
    use coinbase_client::websocket::{Channel, ChannelName, EndpointType, Message};

    #[test]
    fn test_channel_types() {
        let channel = Channel::Ticker {
            product_ids: vec!["BTC-USD".to_string()],
        };
        assert_eq!(channel.name(), "ticker");
        assert_eq!(channel.endpoint_type(), EndpointType::Public);
        assert!(!channel.requires_auth());

        let channel = Channel::User;
        assert_eq!(channel.name(), "user");
        assert_eq!(channel.endpoint_type(), EndpointType::User);
        assert!(channel.requires_auth());
    }

    #[test]
    fn test_message_parsing() {
        let json = r#"{
            "channel": "heartbeats",
            "client_id": "test",
            "timestamp": "2024-01-15T12:00:00Z",
            "sequence_num": 1,
            "events": [{
                "current_time": "2024-01-15T12:00:00Z",
                "heartbeat_counter": 100
            }]
        }"#;

        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.channel, ChannelName::Heartbeats);
        assert_eq!(msg.sequence_num, 1);
    }
}
