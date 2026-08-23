//! Integration tests for the accounts API.

mod common;

use common::{account_json, mock_client};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_account() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts/account-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"account": account_json("account-1")})),
        )
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let account = client.accounts().get("account-1").await.unwrap();

    assert_eq!(account.uuid, "account-1");
    assert_eq!(account.currency, "BTC");
}

#[tokio::test]
async fn test_list_all_accounts_follows_cursors() {
    let mock_server = MockServer::start().await;

    // First page has a cursor to the second page.
    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts"))
        .and(query_param("cursor", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accounts": [account_json("account-2")],
            "has_next": false,
            "cursor": "",
            "size": 1
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v3/brokerage/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accounts": [account_json("account-1")],
            "has_next": true,
            "cursor": "page2",
            "size": 1
        })))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server.uri());
    let accounts = client.accounts().list_all().await.unwrap();

    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].uuid, "account-1");
    assert_eq!(accounts[1].uuid, "account-2");
}
