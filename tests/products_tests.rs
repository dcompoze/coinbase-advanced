//! Integration tests for the products API.

mod common;

use coinbase_advanced::models::{GetCandlesParams, Granularity};
use common::{candle_json, mock_client};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_candles() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/products/BTC-USD/candles"))
        .and(query_param("start", "0"))
        .and(query_param("end", "600"))
        .and(query_param("granularity", "ONE_MINUTE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candles": [candle_json(0), candle_json(60)]
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let candles = client
        .products()
        .get_candles(GetCandlesParams::new(
            "BTC-USD",
            "0",
            "600",
            Granularity::OneMinute,
        ))
        .await
        .unwrap();

    assert_eq!(candles.len(), 2);
}

#[tokio::test]
async fn test_get_candles_ext_chunks_long_ranges() {
    let mock_server = MockServer::start().await;

    // 700 one-minute candles require two windows of at most 350 candles.
    // Window bounds are inclusive, so a full window spans 349 minutes.
    // Coinbase returns candles newest first within a window.
    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/products/BTC-USD/candles"))
        .and(query_param("start", "0"))
        .and(query_param("end", "20940"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candles": [candle_json(60), candle_json(0)]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/products/BTC-USD/candles"))
        .and(query_param("start", "21000"))
        .and(query_param("end", "41940"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candles": [candle_json(21060), candle_json(21000)]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let candles = client
        .products()
        .get_candles_ext(GetCandlesParams::new(
            "BTC-USD",
            "0",
            "41940",
            Granularity::OneMinute,
        ))
        .await
        .unwrap();

    // Combined, deduplicated, and sorted ascending by start time.
    let starts: Vec<&str> = candles.iter().map(|c| c.start.as_str()).collect();
    assert_eq!(starts, vec!["0", "60", "21000", "21060"]);
}
