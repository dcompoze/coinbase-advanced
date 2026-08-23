//! WebSocket client implementation.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::channels::{Channel, ChannelName, EndpointType};
use super::messages::Message;
use crate::constants::{WS_SANDBOX_URL, WS_URL, WS_USER_URL};
use crate::credentials::Credentials;
use crate::error::{Error, Result};
use crate::jwt::generate_ws_jwt;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<Socket, WsMessage>;
type WsStream = SplitStream<Socket>;
type ReconnectFuture = Pin<Box<dyn Future<Output = Result<(WsStream, Option<WsStream>)>> + Send>>;

/// Subscription message sent to the WebSocket.
#[derive(Debug, serde::Serialize)]
struct SubscriptionMessage {
    r#type: &'static str,
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
    max_retries: Option<u32>,
    sandbox: bool,
    validate_sequence: bool,
    public_url: Option<String>,
    user_url: Option<String>,
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
    ///
    /// When enabled, the message stream reconnects with exponential backoff
    /// and resubscribes to all previously subscribed channels.
    pub fn auto_reconnect(mut self, enable: bool) -> Self {
        self.auto_reconnect = enable;
        self
    }

    /// Set maximum number of reconnection attempts (default 10).
    ///
    /// An explicit `0` disables reconnection attempts.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Enable sandbox mode.
    ///
    /// When enabled, the public WebSocket connects to the sandbox endpoint.
    pub fn sandbox(mut self, enabled: bool) -> Self {
        self.sandbox = enabled;
        self
    }

    /// Enable sequence number validation.
    ///
    /// When enabled, a gap in `sequence_num` yields an error item on the stream.
    /// The message that revealed the gap is delivered on the next poll.
    pub fn validate_sequence(mut self, enabled: bool) -> Self {
        self.validate_sequence = enabled;
        self
    }

    /// Override the public WebSocket URL.
    ///
    /// Takes precedence over `sandbox`. Useful for testing against a mock server.
    pub fn public_url(mut self, url: impl Into<String>) -> Self {
        self.public_url = Some(url.into());
        self
    }

    /// Override the user WebSocket URL.
    ///
    /// Useful for testing against a mock server.
    pub fn user_url(mut self, url: impl Into<String>) -> Self {
        self.user_url = Some(url.into());
        self
    }

    /// Build the WebSocket client.
    pub fn build(self) -> Result<WebSocketClient> {
        Ok(WebSocketClient {
            inner: Arc::new(ClientInner {
                credentials: self.credentials,
                auto_reconnect: self.auto_reconnect,
                max_retries: self.max_retries.unwrap_or(10),
                sandbox: self.sandbox,
                validate_sequence: self.validate_sequence,
                public_url: self.public_url,
                user_url: self.user_url,
                public_sink: Mutex::new(None),
                user_sink: Mutex::new(None),
                subscriptions: Mutex::new(Subscriptions::default()),
            }),
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
    fn add(&mut self, channel: &Channel) {
        let name = ChannelName::from(channel);
        let product_ids = channel.product_ids().to_vec();

        let map = match channel.endpoint_type() {
            EndpointType::Public => &mut self.public,
            EndpointType::User => &mut self.user,
        };

        let entry = map.entry(name).or_default();
        for id in product_ids {
            if !entry.contains(&id) {
                entry.push(id);
            }
        }
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

/// Shared client state used by both the client handle and the message stream.
struct ClientInner {
    credentials: Option<Credentials>,
    auto_reconnect: bool,
    max_retries: u32,
    sandbox: bool,
    validate_sequence: bool,
    public_url: Option<String>,
    user_url: Option<String>,
    public_sink: Mutex<Option<WsSink>>,
    user_sink: Mutex<Option<WsSink>>,
    subscriptions: Mutex<Subscriptions>,
}

impl ClientInner {
    fn public_url(&self) -> &str {
        match self.public_url {
            Some(ref url) => url,
            None if self.sandbox => WS_SANDBOX_URL,
            None => WS_URL,
        }
    }

    fn user_url(&self) -> &str {
        self.user_url.as_deref().unwrap_or(WS_USER_URL)
    }

    /// Connect both endpoints, install the sinks, and return the read halves.
    ///
    /// The sinks are installed only after every required connection succeeds,
    /// so a failed connect cannot leave a half-installed pair behind.
    async fn connect_streams(&self) -> Result<(WsStream, Option<WsStream>)> {
        let (public_socket, _) = connect_async(self.public_url()).await.map_err(|e| {
            Error::websocket(format!("Failed to connect to public WebSocket: {}", e))
        })?;

        // If we have credentials, also connect to the user endpoint.
        let user_socket = if self.credentials.is_some() {
            let (user_socket, _) = connect_async(self.user_url()).await.map_err(|e| {
                Error::websocket(format!("Failed to connect to user WebSocket: {}", e))
            })?;
            Some(user_socket)
        } else {
            None
        };

        let (public_sink, public_stream) = public_socket.split();
        *self.public_sink.lock().await = Some(public_sink);

        let user_stream = match user_socket {
            Some(socket) => {
                let (user_sink, user_stream) = socket.split();
                *self.user_sink.lock().await = Some(user_sink);
                Some(user_stream)
            }
            None => None,
        };

        Ok((public_stream, user_stream))
    }

    /// Subscribe to a single channel.
    async fn subscribe_one(&self, channel: &Channel) -> Result<()> {
        let endpoint = channel.endpoint_type();

        // Check if we can subscribe to this channel.
        if channel.requires_auth() && self.credentials.is_none() {
            return Err(Error::websocket(format!(
                "Channel {:?} requires authentication",
                channel.name()
            )));
        }

        let msg = self.build_subscription_message(channel, "subscribe")?;
        self.send_message(endpoint, msg).await?;

        // Track subscription.
        self.subscriptions.lock().await.add(channel);

        Ok(())
    }

    /// Unsubscribe from a single channel.
    async fn unsubscribe_one(&self, channel: &Channel) -> Result<()> {
        let endpoint = channel.endpoint_type();
        let msg = self.build_subscription_message(channel, "unsubscribe")?;
        self.send_message(endpoint, msg).await?;

        // Update subscription tracking.
        self.subscriptions.lock().await.remove(channel);

        Ok(())
    }

    /// Build a subscription/unsubscription message.
    ///
    /// Authenticated channels get a freshly minted JWT on every call,
    /// so resubscribes after a reconnect never reuse an expired token.
    fn build_subscription_message(
        &self,
        channel: &Channel,
        action: &'static str,
    ) -> Result<WsMessage> {
        // Authenticated channels carry a JWT, public channels a timestamp.
        let (jwt, timestamp) = if channel.requires_auth() {
            (Some(self.generate_jwt()?), None)
        } else {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| Error::websocket(format!("Failed to get timestamp: {}", e)))?
                .as_secs()
                .to_string();
            (None, Some(timestamp))
        };

        let msg = SubscriptionMessage {
            r#type: action,
            product_ids: channel.product_ids().to_vec(),
            channel: ChannelName::from(channel),
            jwt,
            timestamp,
        };

        let json = serde_json::to_string(&msg)
            .map_err(|e| Error::websocket(format!("Failed to serialize message: {}", e)))?;

        Ok(WsMessage::Text(json.into()))
    }

    /// Generate a JWT for WebSocket authentication.
    fn generate_jwt(&self) -> Result<String> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or_else(|| Error::websocket("Credentials required for authenticated channels"))?;
        generate_ws_jwt(credentials)
    }

    /// Send a message to the appropriate endpoint.
    async fn send_message(&self, endpoint: EndpointType, msg: WsMessage) -> Result<()> {
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

        sink.send(msg)
            .await
            .map_err(|e| Error::websocket(format!("Failed to send message: {}", e)))
    }

    /// Reconnect with exponential backoff and resubscribe to tracked channels.
    ///
    /// The first attempt is made immediately, the backoff only applies
    /// between failed attempts.
    async fn reconnect(&self) -> Result<(WsStream, Option<WsStream>)> {
        let mut delay = Duration::from_secs(1);

        for attempt in 1..=self.max_retries {
            match self.connect_streams().await {
                Ok((public_stream, user_stream)) => match self.resubscribe().await {
                    Ok(()) => return Ok((public_stream, user_stream)),
                    Err(e) => {
                        tracing::warn!("Resubscribe after reconnect failed: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Reconnect attempt {} failed: {}", attempt, e);
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
            }
        }

        Err(Error::websocket(format!(
            "Failed to reconnect after {} attempts",
            self.max_retries
        )))
    }

    /// Resubscribe to all previously subscribed channels.
    async fn resubscribe(&self) -> Result<()> {
        // Collect channels to resubscribe to.
        let channels_to_resubscribe: Vec<Channel> = {
            let subs = self.subscriptions.lock().await;
            subs.public
                .iter()
                .chain(subs.user.iter())
                .filter_map(|(name, ids)| channel_from_name(name.clone(), ids.clone()))
                .collect()
        };

        // Now resubscribe without holding the lock.
        for channel in channels_to_resubscribe {
            self.subscribe_one(&channel).await?;
        }

        Ok(())
    }
}

/// Convert a channel name and product IDs back to a Channel enum.
fn channel_from_name(name: ChannelName, product_ids: Vec<String>) -> Option<Channel> {
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

/// WebSocket client for Coinbase Advanced Trade API.
pub struct WebSocketClient {
    inner: Arc<ClientInner>,
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
        let (public_stream, user_stream) = self.inner.connect_streams().await?;

        Ok(MessageStream {
            inner: self.inner.clone(),
            public_stream: Some(public_stream),
            user_stream,
            reconnect: None,
            last_public_seq: None,
            last_user_seq: None,
            pending: None,
        })
    }

    /// Subscribe to one or more channels.
    pub async fn subscribe(&self, channels: &[Channel]) -> Result<()> {
        for channel in channels {
            self.inner.subscribe_one(channel).await?;
        }
        Ok(())
    }

    /// Unsubscribe from one or more channels.
    pub async fn unsubscribe(&self, channels: &[Channel]) -> Result<()> {
        for channel in channels {
            self.inner.unsubscribe_one(channel).await?;
        }
        Ok(())
    }
}

/// A stream of WebSocket messages.
///
/// When the client is built with `auto_reconnect(true)`, a lost connection is
/// re-established transparently and all tracked subscriptions are restored.
pub struct MessageStream {
    inner: Arc<ClientInner>,
    public_stream: Option<WsStream>,
    user_stream: Option<WsStream>,
    reconnect: Option<ReconnectFuture>,
    last_public_seq: Option<u64>,
    last_user_seq: Option<u64>,
    pending: Option<Message>,
}

/// Outcome of polling one connection.
enum SideEvent {
    /// An item to yield to the consumer.
    Item(Result<Message>),
    /// The connection is gone.
    Lost(Option<Error>),
    /// A frame was consumed without producing an item.
    Progress,
    /// Nothing available.
    Pending,
}

impl MessageStream {
    fn poll_side(
        stream: &mut Option<WsStream>,
        last_seq: &mut Option<u64>,
        validate_sequence: bool,
        pending: &mut Option<Message>,
        cx: &mut Context<'_>,
    ) -> SideEvent {
        let Some(inner_stream) = stream.as_mut() else {
            return SideEvent::Pending;
        };

        match Pin::new(inner_stream).poll_next(cx) {
            Poll::Ready(Some(Ok(ws_msg))) => match process_ws_message(ws_msg) {
                Processed::Message(Ok(msg)) => {
                    if let Some(gap_error) = check_sequence(validate_sequence, last_seq, &msg) {
                        // Deliver the gap error now and the message on the next poll.
                        *pending = Some(msg);
                        return SideEvent::Item(Err(gap_error));
                    }
                    SideEvent::Item(Ok(msg))
                }
                Processed::Message(Err(e)) => SideEvent::Item(Err(e)),
                Processed::Closed(reason) => {
                    *stream = None;
                    SideEvent::Lost(Some(Error::websocket(reason)))
                }
                Processed::Ignored => SideEvent::Progress,
            },
            Poll::Ready(Some(Err(e))) => {
                *stream = None;
                SideEvent::Lost(Some(Error::websocket(format!("WebSocket error: {}", e))))
            }
            Poll::Ready(None) => {
                *stream = None;
                SideEvent::Lost(None)
            }
            Poll::Pending => SideEvent::Pending,
        }
    }

    fn start_reconnect(&mut self) {
        self.public_stream = None;
        self.user_stream = None;
        let inner = self.inner.clone();
        self.reconnect = Some(Box::pin(async move { inner.reconnect().await }));
    }
}

impl Stream for MessageStream {
    type Item = Result<Message>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // A message held back behind a sequence gap error.
        if let Some(msg) = this.pending.take() {
            return Poll::Ready(Some(Ok(msg)));
        }

        loop {
            // Drive an in-flight reconnect to completion first.
            if let Some(fut) = this.reconnect.as_mut() {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(Ok((public_stream, user_stream))) => {
                        this.reconnect = None;
                        this.public_stream = Some(public_stream);
                        this.user_stream = user_stream;
                        this.last_public_seq = None;
                        this.last_user_seq = None;
                    }
                    Poll::Ready(Err(e)) => {
                        this.reconnect = None;
                        return Poll::Ready(Some(Err(e)));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            let validate = this.inner.validate_sequence;
            let mut progressed = false;
            let mut connection_lost = false;
            let mut loss_error: Option<Error> = None;

            // Always poll both sides so the surviving side registers its
            // waker even when the other side is lost.
            match Self::poll_side(
                &mut this.public_stream,
                &mut this.last_public_seq,
                validate,
                &mut this.pending,
                cx,
            ) {
                SideEvent::Item(item) => return Poll::Ready(Some(item)),
                SideEvent::Lost(err) => {
                    connection_lost = true;
                    loss_error = err;
                }
                SideEvent::Progress => progressed = true,
                SideEvent::Pending => {}
            }

            match Self::poll_side(
                &mut this.user_stream,
                &mut this.last_user_seq,
                validate,
                &mut this.pending,
                cx,
            ) {
                SideEvent::Item(item) => return Poll::Ready(Some(item)),
                SideEvent::Lost(err) => {
                    connection_lost = true;
                    if loss_error.is_none() {
                        loss_error = err;
                    }
                }
                SideEvent::Progress => progressed = true,
                SideEvent::Pending => {}
            }

            if connection_lost {
                if this.inner.auto_reconnect {
                    this.start_reconnect();
                    continue;
                }
                if let Some(e) = loss_error {
                    return Poll::Ready(Some(Err(e)));
                }
                // Connection closed cleanly, fall through to the end check.
            }

            if this.public_stream.is_none() && this.user_stream.is_none() {
                return Poll::Ready(None);
            }

            if !progressed {
                return Poll::Pending;
            }
            // A frame was consumed without an item (e.g. ping), poll again so
            // we never return Pending without a registered waker.
        }
    }
}

/// Result of processing a raw WebSocket frame.
enum Processed {
    /// A parsed message or a parse error.
    Message(Result<Message>),
    /// The server closed the connection.
    Closed(String),
    /// A frame that carries no data (ping/pong/binary).
    Ignored,
}

/// Process a raw WebSocket message into a typed Message.
fn process_ws_message(msg: WsMessage) -> Processed {
    match msg {
        WsMessage::Text(text) => {
            Processed::Message(serde_json::from_str::<Message>(&text).map_err(|e| {
                Error::websocket(format!("Failed to parse message: {}. Raw: {}", e, text))
            }))
        }
        WsMessage::Close(frame) => Processed::Closed(format!("WebSocket closed: {:?}", frame)),
        _ => Processed::Ignored,
    }
}

/// Track the per-connection sequence number and report gaps.
fn check_sequence(validate: bool, last: &mut Option<u64>, msg: &Message) -> Option<Error> {
    let seq = msg.sequence_num;
    let expected = last.and_then(|l| l.checked_add(1));
    *last = Some(seq);

    if !validate {
        return None;
    }

    match expected {
        Some(exp) if seq != exp => Some(Error::websocket(format!(
            "Sequence gap detected: expected {}, got {}",
            exp, seq
        ))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let client = WebSocketClient::builder().build().unwrap();
        assert!(client.inner.credentials.is_none());
        assert!(!client.inner.auto_reconnect);
        assert_eq!(client.inner.max_retries, 10);
        assert!(!client.inner.sandbox);
        assert!(!client.inner.validate_sequence);
    }

    #[test]
    fn test_builder_with_auto_reconnect() {
        let client = WebSocketClient::builder()
            .auto_reconnect(true)
            .build()
            .unwrap();
        assert!(client.inner.auto_reconnect);
        assert_eq!(client.inner.max_retries, 10);
    }

    #[test]
    fn test_builder_sandbox_url() {
        let client = WebSocketClient::builder().sandbox(true).build().unwrap();
        assert_eq!(client.inner.public_url(), WS_SANDBOX_URL);
        let client = WebSocketClient::builder().build().unwrap();
        assert_eq!(client.inner.public_url(), WS_URL);
    }

    #[test]
    fn test_subscription_message_serialize() {
        let msg = SubscriptionMessage {
            r#type: "subscribe",
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

    #[test]
    fn test_check_sequence_gap() {
        let mut last = None;
        let msg = |seq| Message {
            channel: ChannelName::Heartbeats,
            client_id: String::new(),
            timestamp: String::new(),
            sequence_num: seq,
            events: super::super::messages::Events::Heartbeats(Vec::new()),
        };

        assert!(check_sequence(true, &mut last, &msg(0)).is_none());
        assert!(check_sequence(true, &mut last, &msg(1)).is_none());
        // Gap: 2 is skipped.
        assert!(check_sequence(true, &mut last, &msg(3)).is_some());
        // Counter resyncs after a gap.
        assert!(check_sequence(true, &mut last, &msg(4)).is_none());
        // Disabled validation never reports.
        let mut last = Some(0);
        assert!(check_sequence(false, &mut last, &msg(9)).is_none());
    }

    #[test]
    fn test_subscriptions_dedupe() {
        let mut subs = Subscriptions::default();
        let channel = Channel::Ticker {
            product_ids: vec!["BTC-USD".to_string()],
        };
        subs.add(&channel);
        subs.add(&channel);
        assert_eq!(subs.public[&ChannelName::Ticker], vec!["BTC-USD"]);
    }
}
