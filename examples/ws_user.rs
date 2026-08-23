//! Authenticated WebSocket example.
//!
//! Run with: cargo run --example ws_user
//!
//! Streams order updates from the user channel with auto-reconnect
//! and sequence number validation enabled.
//!
//! Requires environment variables:
//! - COINBASE_API_KEY
//! - COINBASE_PRIVATE_KEY

use coinbase_advanced::Credentials;
use coinbase_advanced::ws::{Channel, Events, WebSocketClient};
use futures::StreamExt;

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    tracing_subscriber::fmt::init();

    let client = WebSocketClient::builder()
        .credentials(Credentials::from_env()?)
        .auto_reconnect(true)
        .validate_sequence(true)
        .build()?;

    println!("Connecting...");
    let mut stream = client.connect().await?;

    // The user channel requires authentication.
    // Heartbeats keep the connection alive during quiet periods.
    client
        .subscribe(&[Channel::User, Channel::Heartbeats])
        .await?;
    println!("Subscribed to user channel. Waiting for order updates...\n");

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(message) => match &message.events {
                Events::User(events) => {
                    for event in events {
                        for order in &event.orders {
                            println!("Order update: {:?}", order);
                        }
                    }
                }
                Events::Heartbeats(_) => {}
                other => println!("Other event: {:?}", other),
            },
            // A sequence gap or connection error surfaces here.
            // With auto_reconnect enabled the stream continues afterwards.
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }

    Ok(())
}
