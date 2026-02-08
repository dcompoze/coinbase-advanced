//! WebSocket client for real-time market data and user updates.
//!
//! # Overview
//!
//! This module provides a WebSocket client for connecting to the Coinbase Advanced Trade
//! WebSocket API. It supports both public channels (market data) and authenticated user
//! channels (order updates, fills, etc.).
//!
//! # Example
//!
//! ```no_run
//! use coinbase_advanced::websocket::{WebSocketClient, Channel};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> coinbase_advanced::Result<()> {
//!     // For public data only
//!     let client = WebSocketClient::builder()
//!         .build()?;
//!
//!     // Connect and subscribe
//!     let mut stream = client.connect().await?;
//!     client.subscribe(&[Channel::Ticker {
//!         product_ids: vec!["BTC-USD".to_string()],
//!     }]).await?;
//!
//!     // Listen for messages
//!     while let Some(msg) = stream.next().await {
//!         println!("Received: {:?}", msg);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod channels;
mod client;
mod messages;

pub use channels::{Channel, ChannelName, EndpointType};
pub use client::{WebSocketClient, WebSocketClientBuilder};
pub use messages::*;
