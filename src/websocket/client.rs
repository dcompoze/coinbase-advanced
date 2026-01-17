//! WebSocket client implementation.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use super::channels::{Channel, ChannelName, EndpointType};
use super::messages::Message;
use crate::credentials::Credentials;
use crate::error::{Error, Result};
use crate::jwt::generate_ws_jwt;

/// WebSocket endpoints.
const PUBLIC_ENDPOINT: &str = "wss://advanced-trade-ws.coinbase.com";
const USER_ENDPOINT: &str = "wss://advanced-trade-ws-user.coinbase.com";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<Socket, WsMessage>;
type WsStream = SplitStream<Socket>;

/// Subscription message sent to the WebSocket.
#[derive(Debug, serde::Serialize)]
struct SubscriptionMessage {
    r#type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    product_ids: Vec<String>,
    channel: ChannelName,
    #[serde(skip_serializing_if = "Option::is_none")]
    jwt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

/// Builder for creating a WebSocket client.
#[derive(Default)]
pub struct WebSocketClientBuilder {
    credentials: Option<Credentials>,
    auto_reconnect: bool,
    max_retries: u32,
}

impl WebSocketClientBuilder {
    /// Create a new WebSocket client builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set credentials for authenticated channels.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Enable auto-reconnect on connection loss.
    pub fn auto_reconnect(mut self, enable: bool) -> Self {
        self.auto_reconnect = enable;
        if enable && self.max_retries == 0 {
            self.max_retries = 10;
        }
        self
    }

    /// Set maximum number of reconnection attempts.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Build the WebSocket client.
    pub fn build(self) -> Result<WebSocketClient> {
        Ok(WebSocketClient {
            credentials: self.credentials,
            auto_reconnect: self.auto_reconnect,
            max_retries: self.max_retries,
            public_sink: Arc::new(Mutex::new(None)),
            user_sink: Arc::new(Mutex::new(None)),
            subscriptions: Arc::new(Mutex::new(Subscriptions::new())),
        })
    }
}

/// Tracks current subscriptions for reconnection.
#[derive(Debug, Default)]
struct Subscriptions {
    public: HashMap<ChannelName, Vec<String>>,
    user: HashMap<ChannelName, Vec<String>>,
}

impl Subscriptions {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, channel: &Channel) {
        let name = ChannelName::from(channel);
        let product_ids = channel.product_ids().to_vec();

        let map = match channel.endpoint_type() {
            EndpointType::Public => &mut self.public,
            EndpointType::User => &mut self.user,
        };

        map.entry(name)
            .or_default()
            .extend(product_ids);
    }

    fn remove(&mut self, channel: &Channel) {
        let name = ChannelName::from(channel);
        let product_ids = channel.product_ids();

        let map = match channel.endpoint_type() {
            EndpointType::Public => &mut self.public,
            EndpointType::User => &mut self.user,
        };

        if let Some(ids) = map.get_mut(&name) {
            ids.retain(|id| !product_ids.contains(id));
            if ids.is_empty() {
                map.remove(&name);
            }
        }
    }
}

/// WebSocket client for Coinbase Advanced Trade API.
pub struct WebSocketClient {
    credentials: Option<Credentials>,
    auto_reconnect: bool,
    max_retries: u32,
    public_sink: Arc<Mutex<Option<WsSink>>>,
    user_sink: Arc<Mutex<Option<WsSink>>>,
    subscriptions: Arc<Mutex<Subscriptions>>,
}

impl WebSocketClient {
    /// Create a new WebSocket client builder.
    pub fn builder() -> WebSocketClientBuilder {
        WebSocketClientBuilder::new()
    }

    /// Connect to the WebSocket endpoints.
    ///
    /// Returns a stream of messages from all connected endpoints.
    pub async fn connect(&self) -> Result<MessageStream> {
        let (public_socket, _) = connect_async(PUBLIC_ENDPOINT).await.map_err(|e| {
            Error::websocket(format!("Failed to connect to public WebSocket: {}", e))
        })?;

        let (public_sink, public_stream) = public_socket.split();
        {
            let mut sink = self.public_sink.lock().await;
            *sink = Some(public_sink);
        }

        // If we have credentials, also connect to the user endpoint
        let user_stream = if self.credentials.is_some() {
            let (user_socket, _) = connect_async(USER_ENDPOINT).await.map_err(|e| {
                Error::websocket(format!("Failed to connect to user WebSocket: {}", e))
            })?;

            let (user_sink, user_stream) = user_socket.split();
            {
                let mut sink = self.user_sink.lock().await;
                *sink = Some(user_sink);
            }
            Some(user_stream)
        } else {
            None
        };

        Ok(MessageStream {
            public_stream: Some(public_stream),
            user_stream,
            client: self.clone_internal(),
        })
    }

    /// Subscribe to one or more channels.
    pub async fn subscribe(&self, channels: &[Channel]) -> Result<()> {
        for channel in channels {
            self.subscribe_one(channel).await?;
        }
        Ok(())
    }

    /// Subscribe to a single channel.
    async fn subscribe_one(&self, channel: &Channel) -> Result<()> {
        let endpoint = channel.endpoint_type();

        // Check if we can subscribe to this channel
        if channel.requires_auth() && self.credentials.is_none() {
            return Err(Error::websocket(format!(
                "Channel {:?} requires authentication",
                channel.name()
            )));
        }

        let msg = self.build_subscription_message(channel, "subscribe")?;
        self.send_message(&endpoint, msg).await?;

        // Track subscription
        {
            let mut subs = self.subscriptions.lock().await;
            subs.add(channel);
        }

        Ok(())
    }

    /// Unsubscribe from one or more channels.
    pub async fn unsubscribe(&self, channels: &[Channel]) -> Result<()> {
        for channel in channels {
            self.unsubscribe_one(channel).await?;
        }
        Ok(())
    }

    /// Unsubscribe from a single channel.
    async fn unsubscribe_one(&self, channel: &Channel) -> Result<()> {
        let endpoint = channel.endpoint_type();
        let msg = self.build_subscription_message(channel, "unsubscribe")?;
        self.send_message(&endpoint, msg).await?;

        // Update subscription tracking
        {
            let mut subs = self.subscriptions.lock().await;
            subs.remove(channel);
        }

        Ok(())
    }

    /// Build a subscription/unsubscription message.
    fn build_subscription_message(&self, channel: &Channel, action: &str) -> Result<WsMessage> {
        let channel_name = ChannelName::from(channel);
        let product_ids = channel.product_ids().to_vec();

        let msg = if channel.requires_auth() {
            let jwt = self.generate_jwt()?;
            SubscriptionMessage {
                r#type: action.to_string(),
                product_ids,
                channel: channel_name,
                jwt: Some(jwt),
                timestamp: None,
            }
        } else {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Error::websocket(format!("Failed to get timestamp: {}", e)))?
                .as_secs()
                .to_string();

            SubscriptionMessage {
                r#type: action.to_string(),
                product_ids,
                channel: channel_name,
                jwt: None,
                timestamp: Some(timestamp),
            }
        };

        let json = serde_json::to_string(&msg)
            .map_err(|e| Error::websocket(format!("Failed to serialize message: {}", e)))?;

        Ok(WsMessage::Text(json.into()))
    }

    /// Generate a JWT for WebSocket authentication.
    fn generate_jwt(&self) -> Result<String> {
        let credentials = self.credentials.as_ref().ok_or_else(|| {
            Error::websocket("Credentials required for authenticated channels")
        })?;
        generate_ws_jwt(credentials)
    }

    /// Send a message to the appropriate endpoint.
    async fn send_message(&self, endpoint: &EndpointType, msg: WsMessage) -> Result<()> {
        let sink = match endpoint {
            EndpointType::Public => &self.public_sink,
            EndpointType::User => &self.user_sink,
        };

        let mut guard = sink.lock().await;
        let sink = guard.as_mut().ok_or_else(|| {
            Error::websocket(format!(
                "{:?} WebSocket not connected. Call connect() first.",
                endpoint
            ))
        })?;

        sink.send(msg).await.map_err(|e| {
            Error::websocket(format!("Failed to send message: {}", e))
        })
    }

    /// Attempt to reconnect after a connection loss.
    #[allow(dead_code)]
    async fn reconnect(&self) -> Result<(Option<WsStream>, Option<WsStream>)> {
        if !self.auto_reconnect {
            return Err(Error::websocket("Auto-reconnect is disabled"));
        }

        let mut retry_count = 0;
        let mut delay = Duration::from_secs(1);

        while retry_count < self.max_retries {
            tokio::time::sleep(delay).await;

            match self.attempt_reconnect().await {
                Ok(streams) => {
                    // Resubscribe to previous channels
                    self.resubscribe().await?;
                    return Ok(streams);
                }
                Err(e) => {
                    tracing::warn!("Reconnect attempt {} failed: {}", retry_count + 1, e);
                    retry_count += 1;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(60));
                }
            }
        }

        Err(Error::websocket(format!(
            "Failed to reconnect after {} attempts",
            self.max_retries
        )))
    }

    /// Attempt a single reconnection.
    #[allow(dead_code)]
    async fn attempt_reconnect(&self) -> Result<(Option<WsStream>, Option<WsStream>)> {
        // Reconnect to public endpoint
        let (public_socket, _) = connect_async(PUBLIC_ENDPOINT).await.map_err(|e| {
            Error::websocket(format!("Failed to reconnect to public WebSocket: {}", e))
        })?;

        let (public_sink, public_stream) = public_socket.split();
        {
            let mut sink = self.public_sink.lock().await;
            *sink = Some(public_sink);
        }

        // Reconnect to user endpoint if we have credentials
        let user_stream = if self.credentials.is_some() {
            let (user_socket, _) = connect_async(USER_ENDPOINT).await.map_err(|e| {
                Error::websocket(format!("Failed to reconnect to user WebSocket: {}", e))
            })?;

            let (user_sink, user_stream) = user_socket.split();
            {
                let mut sink = self.user_sink.lock().await;
                *sink = Some(user_sink);
            }
            Some(user_stream)
        } else {
            None
        };

        Ok((Some(public_stream), user_stream))
    }

    /// Resubscribe to all previously subscribed channels.
    #[allow(dead_code)]
    async fn resubscribe(&self) -> Result<()> {
        // Collect channels to resubscribe to
        let channels_to_resubscribe: Vec<Channel> = {
            let subs = self.subscriptions.lock().await;
            let mut channels = Vec::new();

            // Collect public channels
            for (channel_name, product_ids) in &subs.public {
                if let Some(ch) = self.channel_from_name(channel_name.clone(), product_ids.clone()) {
                    channels.push(ch);
                }
            }

            // Collect user channels
            for (channel_name, product_ids) in &subs.user {
                if let Some(ch) = self.channel_from_name(channel_name.clone(), product_ids.clone()) {
                    channels.push(ch);
                }
            }

            channels
        };

        // Now resubscribe without holding the lock
        for channel in channels_to_resubscribe {
            self.subscribe_one(&channel).await?;
        }

        Ok(())
    }

    /// Convert a channel name and product IDs back to a Channel enum.
    #[allow(dead_code)]
    fn channel_from_name(&self, name: ChannelName, product_ids: Vec<String>) -> Option<Channel> {
        match name {
            ChannelName::Heartbeats => Some(Channel::Heartbeats),
            ChannelName::Status => Some(Channel::Status),
            ChannelName::Ticker => Some(Channel::Ticker { product_ids }),
            ChannelName::TickerBatch => Some(Channel::TickerBatch { product_ids }),
            ChannelName::Level2 => Some(Channel::Level2 { product_ids }),
            ChannelName::Candles => Some(Channel::Candles { product_ids }),
            ChannelName::MarketTrades => Some(Channel::MarketTrades { product_ids }),
            ChannelName::User => Some(Channel::User),
            ChannelName::FuturesBalanceSummary => Some(Channel::FuturesBalanceSummary),
            ChannelName::Subscriptions => None,
        }
    }

    /// Clone internal state for the message stream.
    fn clone_internal(&self) -> WebSocketClientInternal {
        WebSocketClientInternal {
            credentials: self.credentials.clone(),
            auto_reconnect: self.auto_reconnect,
            max_retries: self.max_retries,
            public_sink: self.public_sink.clone(),
            user_sink: self.user_sink.clone(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}

/// Internal client state that can be cloned for the message stream.
#[derive(Clone)]
#[allow(dead_code)]
struct WebSocketClientInternal {
    credentials: Option<Credentials>,
    auto_reconnect: bool,
    max_retries: u32,
    public_sink: Arc<Mutex<Option<WsSink>>>,
    user_sink: Arc<Mutex<Option<WsSink>>>,
    subscriptions: Arc<Mutex<Subscriptions>>,
}

/// A stream of WebSocket messages.
pub struct MessageStream {
    public_stream: Option<WsStream>,
    user_stream: Option<WsStream>,
    #[allow(dead_code)]
    client: WebSocketClientInternal,
}

impl Stream for MessageStream {
    type Item = Result<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Try to get a message from the public stream
        if let Some(ref mut stream) = self.public_stream {
            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(Ok(ws_msg))) => {
                    if let Some(msg) = process_ws_message(ws_msg) {
                        return Poll::Ready(Some(msg));
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(Error::websocket(format!(
                        "WebSocket error: {}",
                        e
                    )))));
                }
                Poll::Ready(None) => {
                    // Stream ended
                    self.public_stream = None;
                }
                Poll::Pending => {}
            }
        }

        // Try to get a message from the user stream
        if let Some(ref mut stream) = self.user_stream {
            match Pin::new(stream).poll_next(cx) {
                Poll::Ready(Some(Ok(ws_msg))) => {
                    if let Some(msg) = process_ws_message(ws_msg) {
                        return Poll::Ready(Some(msg));
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(Error::websocket(format!(
                        "WebSocket error: {}",
                        e
                    )))));
                }
                Poll::Ready(None) => {
                    self.user_stream = None;
                }
                Poll::Pending => {}
            }
        }

        // If both streams are gone, we're done
        if self.public_stream.is_none() && self.user_stream.is_none() {
            return Poll::Ready(None);
        }

        Poll::Pending
    }
}

/// Process a raw WebSocket message into a typed Message.
fn process_ws_message(msg: WsMessage) -> Option<Result<Message>> {
    match msg {
        WsMessage::Text(text) => {
            let result = serde_json::from_str::<Message>(&text).map_err(|e| {
                Error::websocket(format!("Failed to parse message: {}. Raw: {}", e, text))
            });
            Some(result)
        }
        WsMessage::Close(frame) => {
            Some(Err(Error::websocket(format!(
                "WebSocket closed: {:?}",
                frame
            ))))
        }
        // Ignore ping/pong/binary frames
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let client = WebSocketClient::builder().build().unwrap();
        assert!(client.credentials.is_none());
        assert!(!client.auto_reconnect);
        assert_eq!(client.max_retries, 0);
    }

    #[test]
    fn test_builder_with_auto_reconnect() {
        let client = WebSocketClient::builder()
            .auto_reconnect(true)
            .build()
            .unwrap();
        assert!(client.auto_reconnect);
        assert_eq!(client.max_retries, 10);
    }

    #[test]
    fn test_subscription_message_serialize() {
        let msg = SubscriptionMessage {
            r#type: "subscribe".to_string(),
            product_ids: vec!["BTC-USD".to_string()],
            channel: ChannelName::Ticker,
            jwt: None,
            timestamp: Some("1234567890".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("BTC-USD"));
        assert!(json.contains("ticker"));
    }
}
