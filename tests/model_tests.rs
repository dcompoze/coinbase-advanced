//! Serialization tests for request and response models.

use coinbase_advanced::models::*;

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

#[test]
fn test_order_configuration_serialization() {
    let config = OrderConfiguration::limit_ioc("0.001", "50000");
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["sor_limit_ioc"]["base_size"], "0.001");
    assert_eq!(json["sor_limit_ioc"]["limit_price"], "50000");

    let config = OrderConfiguration::trigger_bracket_gtc("0.001", "50000", "45000");
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["trigger_bracket_gtc"]["stop_trigger_price"], "45000");

    let config = OrderConfiguration::trigger_bracket_gtd("0.001", "50000", "45000", "2026-01-01");
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["trigger_bracket_gtd"]["end_time"], "2026-01-01");
}

#[test]
fn test_granularity_seconds() {
    assert_eq!(Granularity::OneMinute.seconds(), 60);
    assert_eq!(Granularity::OneHour.seconds(), 3600);
    assert_eq!(Granularity::OneDay.seconds(), 86400);
}

mod rate_limit {
    use coinbase_advanced::rate_limit::{RateLimitConfig, RateLimitInfo, RateLimiter, TokenBucket};

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
