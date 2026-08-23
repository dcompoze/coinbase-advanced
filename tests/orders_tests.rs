//! Integration tests for the orders API.

mod common;

use coinbase_advanced::models::{
    CreateOrderRequest, ListFillsParams, ListOrdersParams, OrderConfiguration, OrderSide,
};
use common::{fill_json, mock_client, order_json};
use wiremock::matchers::{body_partial_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_order() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v3/brokerage/orders"))
        .and(body_partial_json(serde_json::json!({
            "client_order_id": "client-1",
            "product_id": "BTC-USD",
            "side": "BUY",
            "order_configuration": {
                "market_market_ioc": {"quote_size": "100.00"}
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "order_id": "order-1"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let request = CreateOrderRequest::new(
        "client-1",
        "BTC-USD",
        OrderSide::Buy,
        OrderConfiguration::market_buy_quote("100.00"),
    );
    let response = client.orders().create(request).await.unwrap();

    assert!(response.success);
    assert_eq!(response.order_id.as_deref(), Some("order-1"));
}

#[tokio::test]
async fn test_preview_order_typed_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v3/brokerage/orders/preview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "order_total": "100.50",
            "commission_total": "0.50",
            "errs": [],
            "warning": [],
            "quote_size": "100.00",
            "base_size": "0.002",
            "best_bid": "49999",
            "best_ask": "50001",
            "is_max": false
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let request = CreateOrderRequest::new(
        "client-1",
        "BTC-USD",
        OrderSide::Buy,
        OrderConfiguration::market_buy_quote("100.00"),
    );
    let preview = client.orders().preview(request).await.unwrap();

    assert_eq!(preview.order_total, "100.50");
    assert_eq!(preview.commission_total, "0.50");
    assert!(preview.errs.is_empty());
}

#[tokio::test]
async fn test_list_all_orders_follows_cursors() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/orders/historical/batch"))
        .and(query_param("cursor", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orders": [order_json("order-2")],
            "has_next": false,
            "cursor": ""
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/orders/historical/batch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orders": [order_json("order-1")],
            "has_next": true,
            "cursor": "page2"
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let orders = client
        .orders()
        .list_all(ListOrdersParams::new())
        .await
        .unwrap();

    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].order_id, "order-1");
    assert_eq!(orders[1].order_id, "order-2");
}

#[tokio::test]
async fn test_list_fills_all_follows_cursors() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/orders/historical/fills"))
        .and(query_param("cursor", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "fills": [fill_json("fill-2")],
            "cursor": ""
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/orders/historical/fills"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "fills": [fill_json("fill-1")],
            "cursor": "page2"
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let fills = client
        .orders()
        .list_fills_all(ListFillsParams::new())
        .await
        .unwrap();

    assert_eq!(fills.len(), 2);
    assert_eq!(fills[0].entry_id, "fill-1");
    assert_eq!(fills[1].entry_id, "fill-2");
}

#[tokio::test]
async fn test_cancel_all_cancels_open_orders() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/orders/historical/batch"))
        .and(query_param("product_ids", "BTC-USD"))
        .and(query_param("order_status", "OPEN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orders": [order_json("order-1"), order_json("order-2")],
            "has_next": false,
            "cursor": ""
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v3/brokerage/orders/batch_cancel"))
        .and(body_partial_json(serde_json::json!({
            "order_ids": ["order-1", "order-2"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                {"success": true, "failure_reason": null, "order_id": "order-1"},
                {"success": true, "failure_reason": null, "order_id": "order-2"}
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let response = client.orders().cancel_all("BTC-USD").await.unwrap();

    let response = response.expect("Expected a cancel response");
    assert_eq!(response.results.len(), 2);
    assert!(response.results.iter().all(|r| r.success));
}

#[tokio::test]
async fn test_cancel_all_with_no_open_orders() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/orders/historical/batch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orders": [],
            "has_next": false,
            "cursor": ""
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let response = client.orders().cancel_all("BTC-USD").await.unwrap();

    assert!(response.is_none());
}
