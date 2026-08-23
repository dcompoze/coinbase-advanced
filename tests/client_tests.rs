//! Integration tests for the REST client transport layer.
//!
//! These tests use wiremock to mock API responses.

mod common;

use std::time::Duration;

use coinbase_advanced::rate_limit::RateLimitConfig;
use coinbase_advanced::{Credentials, Error, RestClient};
use common::{TEST_EC_KEY, mock_client};
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_credentials_validation() {
    let result = Credentials::new(
        "",
        "-----BEGIN EC PRIVATE KEY-----\ntest\n-----END EC PRIVATE KEY-----",
    );
    assert!(result.is_err());

    let result = Credentials::new("test-key", "");
    assert!(result.is_err());

    let result = Credentials::new("test-key", "not a key");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_builder() {
    let client = RestClient::builder().build().unwrap();
    assert!(!client.has_credentials());

    let client = RestClient::builder().sandbox(true).build().unwrap();
    assert!(client.base_url().contains("sandbox"));

    let client = RestClient::builder().rate_limiting(true).build().unwrap();
    assert!(!client.has_credentials());

    let client = RestClient::builder()
        .base_url("http://localhost:1234")
        .sandbox(true)
        .build()
        .unwrap();
    assert_eq!(client.base_url(), "http://localhost:1234");
}

#[tokio::test]
async fn test_authenticated_request_sends_bearer_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/account-1"))
        .and(header_exists("authorization"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"account": common::account_json("account-1")})),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let account = client.accounts().get("account-1").await.unwrap();
    assert_eq!(account.uuid, "account-1");
}

#[tokio::test]
async fn test_public_request_sends_no_auth_header() {
    let mock_server = MockServer::start().await;

    // The public time endpoint must not carry an Authorization header.
    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/time"))
        .and(|request: &wiremock::Request| !request.headers.contains_key("authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "iso": "2024-01-15T12:00:00Z",
            "epochSeconds": "1705320000",
            "epochMillis": "1705320000000"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let time = client.public().get_time().await.unwrap();
    assert_eq!(time.epoch_seconds, "1705320000");
}

#[tokio::test]
async fn test_api_error_parsing() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/missing"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "Invalid request"
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let error = client.accounts().get("missing").await.unwrap_err();

    match error {
        Error::Api {
            message, status, ..
        } => {
            assert_eq!(message, "Invalid request");
            assert_eq!(status, 400);
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_rate_limit_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/limited"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "60")
                .set_body_json(serde_json::json!({"message": "Rate limit exceeded"})),
        )
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let error = client.accounts().get("limited").await.unwrap_err();

    assert!(error.is_rate_limited());
    match error {
        Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(Duration::from_secs(60)));
        }
        other => panic!("Expected RateLimited error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_retry_config_retries_server_errors() {
    let mock_server = MockServer::start().await;

    // The first request fails with a 500, the retry succeeds.
    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/account-1"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/account-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"account": common::account_json("account-1")})),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = RestClient::builder()
        .credentials(Credentials::new("test-key", TEST_EC_KEY).unwrap())
        .base_url(mock_server.uri())
        .retry_config(
            RateLimitConfig::new()
                .with_max_retries(2)
                .with_initial_backoff(Duration::from_millis(10))
                .with_max_backoff(Duration::from_millis(50)),
        )
        .build()
        .unwrap();

    let account = client.accounts().get("account-1").await.unwrap();
    assert_eq!(account.uuid, "account-1");
}

#[tokio::test]
async fn test_parse_error_keeps_body() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/account-1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let error = client.accounts().get("account-1").await.unwrap_err();

    match error {
        Error::Parse { body, .. } => assert_eq!(body.as_deref(), Some("not json")),
        other => panic!("Expected Parse error, got: {:?}", other),
    }
}
