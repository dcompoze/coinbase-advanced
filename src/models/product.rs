//! Product-related types.

use serde::{Deserialize, Serialize};

/// A tradeable product (trading pair).
#[derive(Debug, Clone, Deserialize)]
pub struct Product {
    /// Product identifier (e.g., "BTC-USD").
    pub product_id: String,
    /// Current price.
    pub price: String,
    /// 24-hour price change percentage.
    pub price_percentage_change_24h: String,
    /// 24-hour trading volume.
    pub volume_24h: String,
    /// 24-hour volume change percentage.
    pub volume_percentage_change_24h: String,
    /// Minimum increment for base currency.
    pub base_increment: String,
    /// Minimum increment for quote currency.
    pub quote_increment: String,
    /// Minimum order size in quote currency.
    pub quote_min_size: String,
    /// Maximum order size in quote currency.
    pub quote_max_size: String,
    /// Minimum order size in base currency.
    pub base_min_size: String,
    /// Maximum order size in base currency.
    pub base_max_size: String,
    /// Base currency name.
    pub base_name: String,
    /// Quote currency name.
    pub quote_name: String,
    /// Whether the product is on the user's watchlist.
    pub watched: bool,
    /// Whether trading is disabled.
    pub is_disabled: bool,
    /// Whether the product is new.
    pub new: bool,
    /// Product status.
    pub status: String,
    /// Whether only cancel orders are allowed.
    pub cancel_only: bool,
    /// Whether only limit orders are allowed.
    pub limit_only: bool,
    /// Whether only post-only orders are allowed.
    pub post_only: bool,
    /// Whether trading is disabled.
    pub trading_disabled: bool,
    /// Whether the product is in auction mode.
    pub auction_mode: bool,
    /// Type of product (SPOT, FUTURE).
    pub product_type: Option<String>,
    /// Quote currency ID.
    pub quote_currency_id: String,
    /// Base currency ID.
    pub base_currency_id: String,
    /// Display symbol for base currency.
    pub base_display_symbol: Option<String>,
    /// Display symbol for quote currency.
    pub quote_display_symbol: Option<String>,
}

/// Request parameters for listing products.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListProductsParams {
    /// Maximum number of products to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Offset for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Filter by product type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    /// Filter by specific product IDs (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<String>,
    /// Include all products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_all_products: Option<bool>,
}

impl ListProductsParams {
    /// Create new list products parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Filter by product type.
    pub fn product_type(mut self, product_type: impl Into<String>) -> Self {
        self.product_type = Some(product_type.into());
        self
    }

    /// Filter by specific product IDs.
    pub fn product_ids(mut self, ids: &[&str]) -> Self {
        self.product_ids = Some(ids.join(","));
        self
    }

    /// Include all products.
    pub fn all(mut self) -> Self {
        self.get_all_products = Some(true);
        self
    }
}

/// Response from listing products.
#[derive(Debug, Clone, Deserialize)]
pub struct ListProductsResponse {
    /// The list of products.
    pub products: Vec<Product>,
    /// Number of products returned.
    pub num_products: Option<u32>,
}

/// Request parameters for getting product details.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetProductParams {
    /// Whether to include tradability status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get_tradability_status: Option<bool>,
}

/// An entry in the order book (bid or ask).
#[derive(Debug, Clone, Deserialize)]
pub struct BookLevel {
    /// Price level.
    pub price: String,
    /// Size at this level.
    pub size: String,
}

/// Order book for a product.
#[derive(Debug, Clone, Deserialize)]
pub struct ProductBook {
    /// Product ID.
    pub product_id: String,
    /// Bid levels (buy orders).
    pub bids: Vec<BookLevel>,
    /// Ask levels (sell orders).
    pub asks: Vec<BookLevel>,
    /// Timestamp of the snapshot.
    pub time: Option<String>,
}

/// Response from getting product book.
#[derive(Debug, Clone, Deserialize)]
pub struct GetProductBookResponse {
    /// The order book.
    pub pricebook: ProductBook,
}

/// Request parameters for getting product book.
#[derive(Debug, Clone, Serialize)]
pub struct GetProductBookParams {
    /// Product ID.
    pub product_id: String,
    /// Number of levels to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Price aggregation increment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_price_increment: Option<String>,
}

impl GetProductBookParams {
    /// Create new product book parameters.
    pub fn new(product_id: impl Into<String>) -> Self {
        Self {
            product_id: product_id.into(),
            limit: None,
            aggregation_price_increment: None,
        }
    }

    /// Set the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the aggregation increment.
    pub fn aggregation(mut self, increment: impl Into<String>) -> Self {
        self.aggregation_price_increment = Some(increment.into());
        self
    }
}

/// Best bid and ask for a product.
#[derive(Debug, Clone, Deserialize)]
pub struct BestBidAsk {
    /// Product ID.
    pub product_id: String,
    /// Best bids.
    pub bids: Vec<BookLevel>,
    /// Best asks.
    pub asks: Vec<BookLevel>,
    /// Timestamp.
    pub time: Option<String>,
}

/// Response from getting best bid/ask.
#[derive(Debug, Clone, Deserialize)]
pub struct GetBestBidAskResponse {
    /// Best bid/ask for each product.
    pub pricebooks: Vec<BestBidAsk>,
}

/// Request parameters for getting best bid/ask.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetBestBidAskParams {
    /// Product IDs (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<String>,
}

impl GetBestBidAskParams {
    /// Create new best bid/ask parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the product IDs.
    pub fn product_ids(mut self, ids: &[&str]) -> Self {
        self.product_ids = Some(ids.join(","));
        self
    }
}

/// Candle data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Granularity {
    /// 1 minute candles.
    OneMinute,
    /// 5 minute candles.
    FiveMinute,
    /// 15 minute candles.
    FifteenMinute,
    /// 30 minute candles.
    ThirtyMinute,
    /// 1 hour candles.
    OneHour,
    /// 2 hour candles.
    TwoHour,
    /// 6 hour candles.
    SixHour,
    /// 1 day candles.
    OneDay,
}

/// A candlestick (OHLCV) data point.
#[derive(Debug, Clone, Deserialize)]
pub struct Candle {
    /// Start time (Unix timestamp).
    pub start: String,
    /// Lowest price.
    pub low: String,
    /// Highest price.
    pub high: String,
    /// Opening price.
    pub open: String,
    /// Closing price.
    pub close: String,
    /// Trading volume.
    pub volume: String,
}

/// Request parameters for getting candles.
#[derive(Debug, Clone, Serialize)]
pub struct GetCandlesParams {
    /// Product ID.
    #[serde(skip_serializing)]
    pub product_id: String,
    /// Start time (Unix timestamp).
    pub start: String,
    /// End time (Unix timestamp).
    pub end: String,
    /// Candle granularity.
    pub granularity: Granularity,
    /// Maximum number of candles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl GetCandlesParams {
    /// Create new candles parameters.
    pub fn new(
        product_id: impl Into<String>,
        start: impl Into<String>,
        end: impl Into<String>,
        granularity: Granularity,
    ) -> Self {
        Self {
            product_id: product_id.into(),
            start: start.into(),
            end: end.into(),
            granularity,
            limit: None,
        }
    }

    /// Set the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response from getting candles.
#[derive(Debug, Clone, Deserialize)]
pub struct GetCandlesResponse {
    /// The candle data.
    pub candles: Vec<Candle>,
}

/// A market trade.
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    /// Trade ID.
    pub trade_id: String,
    /// Product ID.
    pub product_id: String,
    /// Trade price.
    pub price: String,
    /// Trade size.
    pub size: String,
    /// Trade time.
    pub time: String,
    /// Trade side.
    pub side: String,
}

/// Request parameters for getting market trades.
#[derive(Debug, Clone, Serialize)]
pub struct GetMarketTradesParams {
    /// Product ID.
    #[serde(skip_serializing)]
    pub product_id: String,
    /// Maximum number of trades.
    pub limit: u32,
    /// Start time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

impl GetMarketTradesParams {
    /// Create new market trades parameters.
    pub fn new(product_id: impl Into<String>, limit: u32) -> Self {
        Self {
            product_id: product_id.into(),
            limit,
            start: None,
            end: None,
        }
    }

    /// Set the start time.
    pub fn start(mut self, start: impl Into<String>) -> Self {
        self.start = Some(start.into());
        self
    }

    /// Set the end time.
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.end = Some(end.into());
        self
    }
}

/// Response from getting market trades.
#[derive(Debug, Clone, Deserialize)]
pub struct GetMarketTradesResponse {
    /// The trades.
    pub trades: Vec<Trade>,
    /// Best bid.
    pub best_bid: Option<String>,
    /// Best ask.
    pub best_ask: Option<String>,
}
