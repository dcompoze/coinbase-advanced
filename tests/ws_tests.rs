//! Integration tests for the WebSocket client.
//!
//! These tests run a local WebSocket server and point the client at it
//! via the `public_url` builder override.

mod common;

use coinbase_advanced::ws::{Channel, ChannelName, EndpointType, Message, WebSocketClient};
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;

fn heartbeat_json(sequence_num: u64) -> String {
    serde_json::json!({
        "channel": "heartbeats",
        "client_id": "",
        "timestamp": "2026-01-01T00:00:00Z",
        "sequence_num": sequence_num,
        "events": [{
            "current_time": "2026-01-01T00:00:00Z",
            "heartbeat_counter": sequence_num
        }]
    })
    .to_string()
}

async fn accept_ws(listener: &TcpListener) -> tokio_tungstenite::WebSocketStream<TcpStream> {
    let (stream, _) = listener.accept().await.unwrap();
    accept_async(stream).await.unwrap()
}

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
    let msg: Message = serde_json::from_str(&heartbeat_json(1)).unwrap();
    assert_eq!(msg.channel, ChannelName::Heartbeats);
    assert_eq!(msg.sequence_num, 1);
}

#[tokio::test]
async fn test_subscribe_and_receive() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let mut ws = accept_ws(&listener).await;

        // Expect a subscribe message, then send one heartbeat.
        let subscribe = ws.next().await.unwrap().unwrap();
        let subscribe: serde_json::Value =
            serde_json::from_str(subscribe.to_text().unwrap()).unwrap();
        assert_eq!(subscribe["type"], "subscribe");
        assert_eq!(subscribe["channel"], "heartbeats");

        ws.send(WsMessage::text(heartbeat_json(1))).await.unwrap();

        // Keep the connection open until the client disconnects.
        while ws.next().await.is_some() {}
    });

    let client = WebSocketClient::builder()
        .public_url(format!("ws://{}", addr))
        .build()
        .unwrap();

    let mut stream = client.connect().await.unwrap();
    client.subscribe(&[Channel::Heartbeats]).await.unwrap();

    let message = stream.next().await.unwrap().unwrap();
    assert_eq!(message.channel, ChannelName::Heartbeats);
    assert_eq!(message.sequence_num, 1);

    drop(stream);
    drop(client);
    server.await.unwrap();
}

#[tokio::test]
async fn test_auto_reconnect_resubscribes() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        // First connection: subscribe, one message, then close.
        let mut ws = accept_ws(&listener).await;
        let _subscribe = ws.next().await.unwrap().unwrap();
        ws.send(WsMessage::text(heartbeat_json(1))).await.unwrap();
        ws.close(None).await.unwrap();

        // Second connection: the client must resubscribe on its own.
        let mut ws = accept_ws(&listener).await;
        let resubscribe = ws.next().await.unwrap().unwrap();
        let resubscribe: serde_json::Value =
            serde_json::from_str(resubscribe.to_text().unwrap()).unwrap();
        assert_eq!(resubscribe["type"], "subscribe");
        assert_eq!(resubscribe["channel"], "heartbeats");

        ws.send(WsMessage::text(heartbeat_json(1))).await.unwrap();
        while ws.next().await.is_some() {}
    });

    let client = WebSocketClient::builder()
        .public_url(format!("ws://{}", addr))
        .auto_reconnect(true)
        .max_retries(3)
        .build()
        .unwrap();

    let mut stream = client.connect().await.unwrap();
    client.subscribe(&[Channel::Heartbeats]).await.unwrap();

    // First message from the first connection.
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first.sequence_num, 1);

    // Second message arrives after the transparent reconnect.
    let second = stream.next().await.unwrap().unwrap();
    assert_eq!(second.sequence_num, 1);

    drop(stream);
    drop(client);
    server.await.unwrap();
}

#[tokio::test]
async fn test_sequence_gap_detection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server = tokio::spawn(async move {
        let mut ws = accept_ws(&listener).await;
        let _subscribe = ws.next().await.unwrap().unwrap();

        // Send sequence 1, then skip to sequence 5.
        ws.send(WsMessage::text(heartbeat_json(1))).await.unwrap();
        ws.send(WsMessage::text(heartbeat_json(5))).await.unwrap();
        while ws.next().await.is_some() {}
    });

    let client = WebSocketClient::builder()
        .public_url(format!("ws://{}", addr))
        .validate_sequence(true)
        .build()
        .unwrap();

    let mut stream = client.connect().await.unwrap();
    client.subscribe(&[Channel::Heartbeats]).await.unwrap();

    // Sequence 1 is delivered normally.
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first.sequence_num, 1);

    // The gap yields an error, then the gapped message is delivered.
    let gap = stream.next().await.unwrap();
    assert!(gap.is_err());
    let second = stream.next().await.unwrap().unwrap();
    assert_eq!(second.sequence_num, 5);

    drop(stream);
    drop(client);
    server.await.unwrap();
}
