//! WebSocket streaming example.
//!
//! Run with: cargo run --example websocket
//!
//! This example connects to the public WebSocket and streams ticker updates.

use coinbase_client::websocket::{Channel, WebSocketClient};
use futures::StreamExt;

#[tokio::main]
async fn main() -> coinbase_client::Result<()> {
    tracing_subscriber::fmt::init();

    println!("Connecting to Coinbase WebSocket...");

    // Build a WebSocket client (no auth needed for public channels)
    let client = WebSocketClient::builder()
        .auto_reconnect(true)
        .build()?;

    // Connect to WebSocket
    let mut stream = client.connect().await?;
    println!("Connected!");

    // Subscribe to ticker updates for BTC-USD and ETH-USD
    println!("Subscribing to ticker updates...");
    client
        .subscribe(&[
            Channel::Ticker {
                product_ids: vec!["BTC-USD".to_string(), "ETH-USD".to_string()],
            },
            Channel::Heartbeats,
        ])
        .await?;
    println!("Subscribed!");

    // Process messages (limit to 10 for demo)
    println!("\nReceiving messages (press Ctrl+C to stop)...\n");

    let mut count = 0;
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(message) => {
                println!("Message #{}: {:?}", count + 1, message.channel);
                match &message.events {
                    coinbase_client::websocket::Events::Ticker(tickers) => {
                        for event in tickers {
                            for ticker in &event.tickers {
                                println!(
                                    "  {} - Price: ${}, 24h Change: {}%",
                                    ticker.product_id, ticker.price, ticker.price_percent_chg_24_h
                                );
                            }
                        }
                    }
                    coinbase_client::websocket::Events::Heartbeats(hbs) => {
                        for hb in hbs {
                            println!("  Heartbeat #{}", hb.heartbeat_counter);
                        }
                    }
                    _ => {
                        println!("  Other event type");
                    }
                }
                count += 1;

                // Stop after 20 messages for demo
                if count >= 20 {
                    println!("\nReceived 20 messages, stopping...");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    println!("Done!");
    Ok(())
}
