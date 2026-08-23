//! WebSocket message types.

use serde::{Deserialize, Serialize};

use super::channels::ChannelName;

/// A message received from the WebSocket.
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    /// The channel the message is from.
    pub channel: ChannelName,
    /// The client ID for the message.
    pub client_id: String,
    /// The timestamp for the message.
    pub timestamp: String,
    /// The sequence number for the message.
    pub sequence_num: u64,
    /// The events in the message.
    pub events: Events,
}

impl<'de> Deserialize<'de> for Message {
    /// Deserialize a message and dispatch the events on the `channel` field.
    ///
    /// Dispatching by channel avoids the mis-detection an untagged enum has
    /// for structurally similar or empty event payloads.
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMessage {
            channel: ChannelName,
            #[serde(default)]
            client_id: String,
            #[serde(default)]
            timestamp: String,
            #[serde(default)]
            sequence_num: u64,
            #[serde(default = "empty_events")]
            events: serde_json::Value,
        }

        fn empty_events() -> serde_json::Value {
            serde_json::Value::Array(Vec::new())
        }

        fn parse<T, E>(value: serde_json::Value) -> std::result::Result<Vec<T>, E>
        where
            T: serde::de::DeserializeOwned,
            E: serde::de::Error,
        {
            serde_json::from_value(value).map_err(E::custom)
        }

        let raw = RawMessage::deserialize(deserializer)?;
        let events = match raw.channel {
            ChannelName::Status => Events::Status(parse(raw.events)?),
            ChannelName::Candles => Events::Candles(parse(raw.events)?),
            // Ticker and ticker batch share the same event payload.
            ChannelName::Ticker | ChannelName::TickerBatch => Events::Ticker(parse(raw.events)?),
            ChannelName::Level2 => Events::Level2(parse(raw.events)?),
            ChannelName::User => Events::User(parse(raw.events)?),
            ChannelName::MarketTrades => Events::MarketTrades(parse(raw.events)?),
            ChannelName::Heartbeats => Events::Heartbeats(parse(raw.events)?),
            ChannelName::Subscriptions => Events::Subscriptions(parse(raw.events)?),
            ChannelName::FuturesBalanceSummary => Events::FuturesBalanceSummary(parse(raw.events)?),
        };

        Ok(Message {
            channel: raw.channel,
            client_id: raw.client_id,
            timestamp: raw.timestamp,
            sequence_num: raw.sequence_num,
            events,
        })
    }
}

/// Events that can be received in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Events {
    /// Status events.
    Status(Vec<StatusEvent>),
    /// Candle events.
    Candles(Vec<CandlesEvent>),
    /// Ticker events.
    Ticker(Vec<TickerEvent>),
    /// Level 2 order book events.
    Level2(Vec<Level2Event>),
    /// User events.
    User(Vec<UserEvent>),
    /// Market trade events.
    MarketTrades(Vec<MarketTradesEvent>),
    /// Heartbeat events.
    Heartbeats(Vec<HeartbeatsEvent>),
    /// Subscription confirmation events.
    Subscriptions(Vec<SubscriptionsEvent>),
    /// Futures balance summary events.
    FuturesBalanceSummary(Vec<FuturesBalanceSummaryEvent>),
}

/// Event type (snapshot or update).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Initial snapshot of data.
    Snapshot,
    /// Incremental update.
    Update,
}

/// Status event containing product status updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEvent {
    /// Event type.
    pub r#type: EventType,
    /// Product updates.
    pub products: Vec<ProductStatus>,
}

/// Product status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductStatus {
    /// Product type.
    #[serde(default)]
    pub product_type: String,
    /// Product ID.
    pub id: String,
    /// Base currency symbol.
    pub base_currency: String,
    /// Quote currency symbol.
    pub quote_currency: String,
    /// Minimum base increment.
    pub base_increment: String,
    /// Minimum quote increment.
    pub quote_increment: String,
    /// Display name.
    #[serde(default)]
    pub display_name: String,
    /// Product status.
    pub status: String,
    /// Additional status message.
    #[serde(default)]
    pub status_message: String,
    /// Minimum market funds.
    #[serde(default)]
    pub min_market_funds: String,
}

/// Candles event containing candle updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlesEvent {
    /// Event type.
    pub r#type: EventType,
    /// Candle updates.
    pub candles: Vec<CandleUpdate>,
}

/// A candle (OHLCV) update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleUpdate {
    /// Product ID.
    pub product_id: String,
    /// Start time.
    pub start: String,
    /// Open price.
    pub open: String,
    /// High price.
    pub high: String,
    /// Low price.
    pub low: String,
    /// Close price.
    pub close: String,
    /// Volume.
    pub volume: String,
}

/// Ticker event containing ticker updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerEvent {
    /// Event type.
    pub r#type: EventType,
    /// Ticker updates.
    pub tickers: Vec<TickerUpdate>,
}

/// A ticker update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerUpdate {
    /// Ticker type.
    pub r#type: String,
    /// Product ID.
    pub product_id: String,
    /// Current price.
    pub price: String,
    /// 24-hour volume.
    pub volume_24_h: String,
    /// 24-hour low.
    pub low_24_h: String,
    /// 24-hour high.
    pub high_24_h: String,
    /// 52-week low.
    pub low_52_w: String,
    /// 52-week high.
    pub high_52_w: String,
    /// 24-hour price percentage change.
    pub price_percent_chg_24_h: String,
}

/// Level 2 order book event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level2Event {
    /// Event type.
    pub r#type: EventType,
    /// Product ID.
    pub product_id: String,
    /// Order book updates.
    pub updates: Vec<Level2Update>,
}

/// A Level 2 order book update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level2Update {
    /// Side (bid or ask).
    pub side: Level2Side,
    /// Event time.
    #[serde(default)]
    pub event_time: String,
    /// Price level.
    pub price_level: String,
    /// New quantity at this level.
    pub new_quantity: String,
}

/// Side of a Level 2 order book entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level2Side {
    /// Bid (buy) side.
    Bid,
    /// Ask (sell) side.
    #[serde(alias = "offer")]
    Ask,
}

/// User event containing order updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    /// Event type.
    pub r#type: EventType,
    /// Order updates.
    pub orders: Vec<OrderUpdate>,
}

/// An order update for the user channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    /// Average fill price.
    #[serde(default)]
    pub avg_price: String,
    /// Cancel reason if cancelled.
    #[serde(default)]
    pub cancel_reason: String,
    /// Client-provided order ID.
    #[serde(default)]
    pub client_order_id: String,
    /// Completion percentage.
    #[serde(default)]
    pub completion_percentage: String,
    /// Contract expiry type.
    #[serde(default)]
    pub contract_expiry_type: String,
    /// Cumulative filled quantity.
    #[serde(default)]
    pub cumulative_quantity: String,
    /// Total filled value.
    #[serde(default)]
    pub filled_value: String,
    /// Remaining quantity.
    #[serde(default)]
    pub leaves_quantity: String,
    /// Limit price.
    #[serde(default)]
    pub limit_price: String,
    /// Number of fills.
    #[serde(default)]
    pub number_of_fills: String,
    /// Order ID.
    pub order_id: String,
    /// Order side (BUY or SELL).
    pub order_side: String,
    /// Order type.
    pub order_type: String,
    /// Outstanding hold amount.
    #[serde(default)]
    pub outstanding_hold_amount: String,
    /// Post-only flag.
    /// The API sends this as a string in user channel messages.
    #[serde(default, deserialize_with = "bool_from_bool_or_string")]
    pub post_only: bool,
    /// Product ID.
    pub product_id: String,
    /// Product type.
    #[serde(default)]
    pub product_type: String,
    /// Reject reason if rejected.
    #[serde(default)]
    pub reject_reason: Option<String>,
    /// Retail portfolio ID.
    #[serde(default)]
    pub retail_portfolio_id: String,
    /// Risk managed by.
    #[serde(default)]
    pub risk_managed_by: String,
    /// Order status.
    pub status: String,
    /// Stop price.
    #[serde(default)]
    pub stop_price: Option<String>,
    /// Time in force.
    #[serde(default)]
    pub time_in_force: String,
    /// Total fees.
    #[serde(default)]
    pub total_fees: String,
    /// Total value after fees.
    #[serde(default)]
    pub total_value_after_fees: String,
    /// Trigger status.
    #[serde(default)]
    pub trigger_status: String,
    /// Creation time.
    #[serde(default)]
    pub creation_time: String,
    /// End time.
    #[serde(default)]
    pub end_time: String,
    /// Start time.
    #[serde(default)]
    pub start_time: String,
}

/// Deserialize a bool that the API may send as either a bool or a string.
fn bool_from_bool_or_string<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        Str(String),
    }

    match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(value) => Ok(value),
        BoolOrString::Str(value) => match value.to_ascii_lowercase().as_str() {
            "true" => Ok(true),
            "false" | "" => Ok(false),
            other => Err(serde::de::Error::custom(format!(
                "invalid boolean string: {}",
                other
            ))),
        },
    }
}

/// Market trades event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTradesEvent {
    /// Event type.
    pub r#type: EventType,
    /// Trade updates.
    pub trades: Vec<TradeUpdate>,
}

/// A trade update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeUpdate {
    /// Trade ID.
    pub trade_id: String,
    /// Product ID.
    pub product_id: String,
    /// Trade price.
    pub price: String,
    /// Trade size.
    pub size: String,
    /// Trade side (BUY or SELL).
    pub side: String,
    /// Trade time.
    #[serde(default)]
    pub time: String,
}

/// Heartbeat event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatsEvent {
    /// Current server time.
    pub current_time: String,
    /// Heartbeat counter.
    pub heartbeat_counter: u64,
}

/// Subscriptions confirmation event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionsEvent {
    /// Current subscriptions.
    pub subscriptions: SubscriptionStatus,
}

/// Current subscription status.
/// Channels the connection is not subscribed to are empty.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscriptionStatus {
    /// Status channel subscriptions.
    #[serde(default)]
    pub status: Vec<String>,
    /// Ticker channel subscriptions.
    #[serde(default)]
    pub ticker: Vec<String>,
    /// Ticker batch channel subscriptions.
    #[serde(default)]
    pub ticker_batch: Vec<String>,
    /// Level 2 channel subscriptions.
    #[serde(default)]
    pub level2: Vec<String>,
    /// Candles channel subscriptions.
    #[serde(default)]
    pub candles: Vec<String>,
    /// User channel subscriptions.
    #[serde(default)]
    pub user: Vec<String>,
    /// Market trades channel subscriptions.
    #[serde(default)]
    pub market_trades: Vec<String>,
    /// Heartbeats channel subscriptions.
    #[serde(default)]
    pub heartbeats: Vec<String>,
    /// Futures balance summary channel subscriptions.
    #[serde(default)]
    pub futures_balance_summary: Vec<String>,
}

/// Futures balance summary event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesBalanceSummaryEvent {
    /// Event type.
    pub r#type: EventType,
    /// Balance summary data.
    pub fcm_balance_summary: FuturesBalanceSummary,
}

/// Futures balance summary data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesBalanceSummary {
    /// Futures buying power.
    #[serde(default)]
    pub futures_buying_power: String,
    /// Total USD balance.
    #[serde(default)]
    pub total_usd_balance: String,
    /// CBI USD balance.
    #[serde(default)]
    pub cbi_usd_balance: String,
    /// CFM USD balance.
    #[serde(default)]
    pub cfm_usd_balance: String,
    /// Total open orders hold amount.
    #[serde(default)]
    pub total_open_orders_hold_amount: String,
    /// Unrealized PnL.
    #[serde(default)]
    pub unrealized_pnl: String,
    /// Daily realized PnL.
    #[serde(default)]
    pub daily_realized_pnl: String,
    /// Initial margin.
    #[serde(default)]
    pub initial_margin: String,
    /// Available margin.
    #[serde(default)]
    pub available_margin: String,
    /// Liquidation threshold.
    #[serde(default)]
    pub liquidation_threshold: String,
    /// Liquidation buffer amount.
    #[serde(default)]
    pub liquidation_buffer_amount: String,
    /// Liquidation buffer percentage.
    #[serde(default)]
    pub liquidation_buffer_percentage: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_deserialize() {
        let data = r#"
            {
                "channel":"heartbeats",
                "client_id":"",
                "timestamp":"2025-01-14T22:11:18.791273556Z",
                "sequence_num":17,
                "events":
                [
                    {
                        "current_time":"2025-01-14 22:11:18.787177997 +0000 UTC m=+25541.571430466",
                        "heartbeat_counter":25539
                    }
                ]
            }
        "#;

        let msg: Result<Message, _> = serde_json::from_str(data);
        assert!(msg.is_ok());
    }

    #[test]
    fn test_events_dispatch_on_channel() {
        // Empty events must dispatch to the channel's variant,
        // not fall through to the first untagged variant.
        let data =
            r#"{"channel":"ticker","client_id":"","timestamp":"t","sequence_num":1,"events":[]}"#;
        let msg: Message = serde_json::from_str(data).unwrap();
        assert!(matches!(msg.events, Events::Ticker(ref e) if e.is_empty()));

        // Ticker batch shares the ticker payload.
        let data = r#"{"channel":"ticker_batch","client_id":"","timestamp":"t","sequence_num":1,"events":[]}"#;
        let msg: Message = serde_json::from_str(data).unwrap();
        assert!(matches!(msg.events, Events::Ticker(_)));

        // Missing envelope fields get defaults.
        let data = r#"{"channel":"heartbeats"}"#;
        let msg: Message = serde_json::from_str(data).unwrap();
        assert_eq!(msg.sequence_num, 0);
        assert!(matches!(msg.events, Events::Heartbeats(ref e) if e.is_empty()));
    }

    #[test]
    fn test_order_update_post_only_string() {
        let data = r#"{
            "order_id": "o1",
            "order_side": "BUY",
            "order_type": "Limit",
            "post_only": "true",
            "product_id": "BTC-USD",
            "status": "OPEN"
        }"#;
        let update: OrderUpdate = serde_json::from_str(data).unwrap();
        assert!(update.post_only);

        let data = r#"{
            "order_id": "o1",
            "order_side": "BUY",
            "order_type": "Limit",
            "post_only": false,
            "product_id": "BTC-USD",
            "status": "OPEN"
        }"#;
        let update: OrderUpdate = serde_json::from_str(data).unwrap();
        assert!(!update.post_only);
    }

    #[test]
    fn test_level2_side_deserialize() {
        // Test normal cases
        assert_eq!(
            serde_json::from_str::<Level2Side>(r#""bid""#).unwrap(),
            Level2Side::Bid
        );
        assert_eq!(
            serde_json::from_str::<Level2Side>(r#""ask""#).unwrap(),
            Level2Side::Ask
        );
        // Test "offer" alias
        assert_eq!(
            serde_json::from_str::<Level2Side>(r#""offer""#).unwrap(),
            Level2Side::Ask
        );
    }
}
