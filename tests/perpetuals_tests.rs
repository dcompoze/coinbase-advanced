//! Integration tests for the perpetuals API.

mod common;

use coinbase_advanced::models::SetMultiAssetCollateralRequest;
use common::mock_client;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_multi_asset_collateral_path() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v3/brokerage/intx/multi_asset_collateral"))
        .and(body_json(serde_json::json!({
            "portfolio_uuid": "portfolio-1",
            "multi_asset_collateral_enabled": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "multi_asset_collateral_enabled": true
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let response = client
        .perpetuals()
        .set_multi_asset_collateral("portfolio-1", SetMultiAssetCollateralRequest::new(true))
        .await
        .unwrap();

    assert!(response.multi_asset_collateral_enabled);
}
