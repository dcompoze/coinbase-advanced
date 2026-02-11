//! WebSocket channel definitions.

use serde::{Deserialize, Serialize};

/// Endpoint types for WebSocket connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EndpointType {
    /// Public endpoint for unauthenticated market data.
    Public,
    /// User endpoint for authenticated user data.
    User,
}

/// WebSocket channels that can be subscribed to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Channel {
    /// Heartbeat messages to keep connections alive.
    Heartbeats,

    /// Product status updates.
    Status,

    /// Real-time ticker updates for products.
    Ticker {
        /// Product IDs to subscribe to (e.g., "BTC-USD").
        product_ids: Vec<String>,
    },

    /// Batched ticker updates (less frequent than ticker).
    TickerBatch {
        /// Product IDs to subscribe to.
        product_ids: Vec<String>,
    },

    /// Level 2 order book updates.
    Level2 {
        /// Product IDs to subscribe to.
        product_ids: Vec<String>,
    },

    /// Candle (OHLCV) updates.
    Candles {
        /// Product IDs to subscribe to.
        product_ids: Vec<String>,
    },

    /// Market trade updates.
    MarketTrades {
        /// Product IDs to subscribe to.
        product_ids: Vec<String>,
    },

    /// User channel for authenticated updates (orders, fills).
    /// Requires authentication.
    User,

    /// Futures balance summary updates.
    /// Requires authentication.
    FuturesBalanceSummary,
}

impl Channel {
    /// Get the channel name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Channel::Heartbeats => "heartbeats",
            Channel::Status => "status",
            Channel::Ticker { .. } => "ticker",
            Channel::TickerBatch { .. } => "ticker_batch",
            Channel::Level2 { .. } => "level2",
            Channel::Candles { .. } => "candles",
            Channel::MarketTrades { .. } => "market_trades",
            Channel::User => "user",
            Channel::FuturesBalanceSummary => "futures_balance_summary",
        }
    }

    /// Get the product IDs for this channel, if applicable.
    pub fn product_ids(&self) -> &[String] {
        match self {
            Channel::Ticker { product_ids }
            | Channel::TickerBatch { product_ids }
            | Channel::Level2 { product_ids }
            | Channel::Candles { product_ids }
            | Channel::MarketTrades { product_ids } => product_ids,
            _ => &[],
        }
    }

    /// Get the endpoint type for this channel.
    pub fn endpoint_type(&self) -> EndpointType {
        match self {
            Channel::User | Channel::FuturesBalanceSummary => EndpointType::User,
            _ => EndpointType::Public,
        }
    }

    /// Check if this channel requires authentication.
    pub fn requires_auth(&self) -> bool {
        matches!(self, Channel::User | Channel::FuturesBalanceSummary)
    }
}

/// Channel name for serialization/deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ChannelName {
    Heartbeats,
    Status,
    Ticker,
    TickerBatch,
    #[serde(alias = "l2_data")]
    Level2,
    Candles,
    MarketTrades,
    User,
    FuturesBalanceSummary,
    Subscriptions,
}

impl From<&Channel> for ChannelName {
    fn from(channel: &Channel) -> Self {
        match channel {
            Channel::Heartbeats => ChannelName::Heartbeats,
            Channel::Status => ChannelName::Status,
            Channel::Ticker { .. } => ChannelName::Ticker,
            Channel::TickerBatch { .. } => ChannelName::TickerBatch,
            Channel::Level2 { .. } => ChannelName::Level2,
            Channel::Candles { .. } => ChannelName::Candles,
            Channel::MarketTrades { .. } => ChannelName::MarketTrades,
            Channel::User => ChannelName::User,
            Channel::FuturesBalanceSummary => ChannelName::FuturesBalanceSummary,
        }
    }
}
