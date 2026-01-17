//! Order-related types.

use serde::{Deserialize, Serialize};

/// Order side (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    /// Buy order.
    Buy,
    /// Sell order.
    Sell,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// Order is pending.
    Pending,
    /// Order is open.
    Open,
    /// Order has been filled.
    Filled,
    /// Order has been cancelled.
    Cancelled,
    /// Order has expired.
    Expired,
    /// Order failed.
    Failed,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

/// Stop direction for stop orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StopDirection {
    /// Stop triggers when price goes up.
    StopDirectionStopUp,
    /// Stop triggers when price goes down.
    StopDirectionStopDown,
}

/// Market IOC order configuration.
#[derive(Debug, Clone, Serialize)]
pub struct MarketIoc {
    /// Size in quote currency (e.g., USD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_size: Option<String>,
    /// Size in base currency (e.g., BTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_size: Option<String>,
}

/// Limit GTC order configuration.
#[derive(Debug, Clone, Serialize)]
pub struct LimitGtc {
    /// Size in base currency.
    pub base_size: String,
    /// Limit price.
    pub limit_price: String,
    /// Whether to only add liquidity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
}

/// Limit GTD order configuration.
#[derive(Debug, Clone, Serialize)]
pub struct LimitGtd {
    /// Size in base currency.
    pub base_size: String,
    /// Limit price.
    pub limit_price: String,
    /// Expiration time (ISO 8601).
    pub end_time: String,
    /// Whether to only add liquidity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
}

/// Limit FOK order configuration.
#[derive(Debug, Clone, Serialize)]
pub struct LimitFok {
    /// Size in base currency.
    pub base_size: String,
    /// Limit price.
    pub limit_price: String,
}

/// Stop-limit GTC order configuration.
#[derive(Debug, Clone, Serialize)]
pub struct StopLimitGtc {
    /// Size in base currency.
    pub base_size: String,
    /// Limit price.
    pub limit_price: String,
    /// Stop price.
    pub stop_price: String,
    /// Stop direction.
    pub stop_direction: StopDirection,
}

/// Stop-limit GTD order configuration.
#[derive(Debug, Clone, Serialize)]
pub struct StopLimitGtd {
    /// Size in base currency.
    pub base_size: String,
    /// Limit price.
    pub limit_price: String,
    /// Stop price.
    pub stop_price: String,
    /// Expiration time (ISO 8601).
    pub end_time: String,
    /// Stop direction.
    pub stop_direction: StopDirection,
}

/// Order configuration.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum OrderConfiguration {
    /// Market order (immediate-or-cancel).
    MarketIoc {
        /// Market IOC configuration.
        market_market_ioc: MarketIoc,
    },
    /// Limit order (good-til-cancelled).
    LimitGtc {
        /// Limit GTC configuration.
        limit_limit_gtc: LimitGtc,
    },
    /// Limit order (good-til-date).
    LimitGtd {
        /// Limit GTD configuration.
        limit_limit_gtd: LimitGtd,
    },
    /// Limit order (fill-or-kill).
    LimitFok {
        /// Limit FOK configuration.
        limit_limit_fok: LimitFok,
    },
    /// Stop-limit order (good-til-cancelled).
    StopLimitGtc {
        /// Stop-limit GTC configuration.
        stop_limit_stop_limit_gtc: StopLimitGtc,
    },
    /// Stop-limit order (good-til-date).
    StopLimitGtd {
        /// Stop-limit GTD configuration.
        stop_limit_stop_limit_gtd: StopLimitGtd,
    },
}

impl OrderConfiguration {
    /// Create a market buy order by quote size (e.g., $100 of BTC).
    pub fn market_buy_quote(quote_size: impl Into<String>) -> Self {
        Self::MarketIoc {
            market_market_ioc: MarketIoc {
                quote_size: Some(quote_size.into()),
                base_size: None,
            },
        }
    }

    /// Create a market buy order by base size (e.g., 0.001 BTC).
    pub fn market_buy_base(base_size: impl Into<String>) -> Self {
        Self::MarketIoc {
            market_market_ioc: MarketIoc {
                quote_size: None,
                base_size: Some(base_size.into()),
            },
        }
    }

    /// Create a market sell order by base size.
    pub fn market_sell(base_size: impl Into<String>) -> Self {
        Self::MarketIoc {
            market_market_ioc: MarketIoc {
                quote_size: None,
                base_size: Some(base_size.into()),
            },
        }
    }

    /// Create a limit GTC order.
    pub fn limit_gtc(
        base_size: impl Into<String>,
        limit_price: impl Into<String>,
        post_only: bool,
    ) -> Self {
        Self::LimitGtc {
            limit_limit_gtc: LimitGtc {
                base_size: base_size.into(),
                limit_price: limit_price.into(),
                post_only: Some(post_only),
            },
        }
    }

    /// Create a limit GTD order.
    pub fn limit_gtd(
        base_size: impl Into<String>,
        limit_price: impl Into<String>,
        end_time: impl Into<String>,
        post_only: bool,
    ) -> Self {
        Self::LimitGtd {
            limit_limit_gtd: LimitGtd {
                base_size: base_size.into(),
                limit_price: limit_price.into(),
                end_time: end_time.into(),
                post_only: Some(post_only),
            },
        }
    }

    /// Create a limit FOK order.
    pub fn limit_fok(base_size: impl Into<String>, limit_price: impl Into<String>) -> Self {
        Self::LimitFok {
            limit_limit_fok: LimitFok {
                base_size: base_size.into(),
                limit_price: limit_price.into(),
            },
        }
    }

    /// Create a stop-limit GTC order.
    pub fn stop_limit_gtc(
        base_size: impl Into<String>,
        limit_price: impl Into<String>,
        stop_price: impl Into<String>,
        stop_direction: StopDirection,
    ) -> Self {
        Self::StopLimitGtc {
            stop_limit_stop_limit_gtc: StopLimitGtc {
                base_size: base_size.into(),
                limit_price: limit_price.into(),
                stop_price: stop_price.into(),
                stop_direction,
            },
        }
    }

    /// Create a stop-limit GTD order.
    pub fn stop_limit_gtd(
        base_size: impl Into<String>,
        limit_price: impl Into<String>,
        stop_price: impl Into<String>,
        end_time: impl Into<String>,
        stop_direction: StopDirection,
    ) -> Self {
        Self::StopLimitGtd {
            stop_limit_stop_limit_gtd: StopLimitGtd {
                base_size: base_size.into(),
                limit_price: limit_price.into(),
                stop_price: stop_price.into(),
                end_time: end_time.into(),
                stop_direction,
            },
        }
    }
}

/// Request to create an order.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    /// Client-generated order ID (UUID recommended).
    pub client_order_id: String,
    /// Product ID (e.g., "BTC-USD").
    pub product_id: String,
    /// Order side (buy or sell).
    pub side: OrderSide,
    /// Order configuration.
    pub order_configuration: OrderConfiguration,
    /// Self-trade prevention ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_trade_prevention_id: Option<String>,
    /// Leverage (for margin trading).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leverage: Option<String>,
    /// Margin type (for margin trading).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_type: Option<String>,
    /// Retail portfolio ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retail_portfolio_id: Option<String>,
}

impl CreateOrderRequest {
    /// Create a new order request.
    pub fn new(
        client_order_id: impl Into<String>,
        product_id: impl Into<String>,
        side: OrderSide,
        order_configuration: OrderConfiguration,
    ) -> Self {
        Self {
            client_order_id: client_order_id.into(),
            product_id: product_id.into(),
            side,
            order_configuration,
            self_trade_prevention_id: None,
            leverage: None,
            margin_type: None,
            retail_portfolio_id: None,
        }
    }
}

/// Success response when creating an order.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderSuccessResponse {
    /// The order ID.
    pub order_id: String,
    /// Product ID.
    pub product_id: Option<String>,
    /// Order side.
    pub side: Option<String>,
    /// Client order ID.
    pub client_order_id: Option<String>,
}

/// Response from creating an order.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    /// Whether the order was successful.
    pub success: bool,
    /// Failure reason (if failed).
    pub failure_reason: Option<String>,
    /// Order ID (if successful).
    pub order_id: Option<String>,
    /// Success response details.
    pub success_response: Option<OrderSuccessResponse>,
    /// Error response (if failed).
    pub error_response: Option<serde_json::Value>,
}

/// Request to cancel orders.
#[derive(Debug, Clone, Serialize)]
pub struct CancelOrdersRequest {
    /// Order IDs to cancel.
    pub order_ids: Vec<String>,
}

impl CancelOrdersRequest {
    /// Create a new cancel orders request.
    pub fn new(order_ids: Vec<String>) -> Self {
        Self { order_ids }
    }

    /// Create a cancel request for a single order.
    pub fn single(order_id: impl Into<String>) -> Self {
        Self {
            order_ids: vec![order_id.into()],
        }
    }
}

/// Result of cancelling a single order.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResult {
    /// Whether the cancellation was successful.
    pub success: bool,
    /// Failure reason (if failed).
    pub failure_reason: Option<String>,
    /// The order ID.
    pub order_id: String,
}

/// Response from cancelling orders.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrdersResponse {
    /// Results for each order.
    pub results: Vec<CancelOrderResult>,
}

/// Request to edit an order.
#[derive(Debug, Clone, Serialize)]
pub struct EditOrderRequest {
    /// The order ID to edit.
    pub order_id: String,
    /// New price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// New size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

impl EditOrderRequest {
    /// Create a new edit order request.
    pub fn new(order_id: impl Into<String>) -> Self {
        Self {
            order_id: order_id.into(),
            price: None,
            size: None,
        }
    }

    /// Set the new price.
    pub fn price(mut self, price: impl Into<String>) -> Self {
        self.price = Some(price.into());
        self
    }

    /// Set the new size.
    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }
}

/// Response from editing an order.
#[derive(Debug, Clone, Deserialize)]
pub struct EditOrderResponse {
    /// Whether the edit was successful.
    pub success: bool,
    /// Errors (if any).
    pub errors: Option<Vec<serde_json::Value>>,
}

/// An order.
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    /// Order ID.
    pub order_id: String,
    /// Product ID.
    pub product_id: String,
    /// User ID.
    pub user_id: Option<String>,
    /// Order configuration.
    pub order_configuration: Option<serde_json::Value>,
    /// Order side.
    pub side: String,
    /// Client order ID.
    pub client_order_id: String,
    /// Order status.
    pub status: String,
    /// Time in force.
    pub time_in_force: Option<String>,
    /// Created time.
    pub created_time: Option<String>,
    /// Completion percentage.
    pub completion_percentage: Option<String>,
    /// Filled size.
    pub filled_size: Option<String>,
    /// Average filled price.
    pub average_filled_price: Option<String>,
    /// Fee amount.
    pub fee: Option<String>,
    /// Number of fills.
    pub number_of_fills: Option<String>,
    /// Filled value.
    pub filled_value: Option<String>,
    /// Whether the order is pending cancel.
    pub pending_cancel: Option<bool>,
    /// Whether the order size includes fees.
    pub size_in_quote: Option<bool>,
    /// Total fees.
    pub total_fees: Option<String>,
    /// Whether size includes fees.
    pub size_inclusive_of_fees: Option<bool>,
    /// Total value after fees.
    pub total_value_after_fees: Option<String>,
    /// Trigger status.
    pub trigger_status: Option<String>,
    /// Order type.
    pub order_type: Option<String>,
    /// Reject reason.
    pub reject_reason: Option<String>,
    /// Settled.
    pub settled: Option<bool>,
    /// Product type.
    pub product_type: Option<String>,
    /// Reject message.
    pub reject_message: Option<String>,
    /// Cancel message.
    pub cancel_message: Option<String>,
    /// Order placement source.
    pub order_placement_source: Option<String>,
    /// Outstanding hold amount.
    pub outstanding_hold_amount: Option<String>,
}

/// Parameters for listing orders.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListOrdersParams {
    /// Filter by product IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<String>,
    /// Filter by order status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    /// Maximum number of orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Start date (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// End date (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    /// Order side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_side: Option<String>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Product type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    /// Order type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    /// Retail portfolio ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retail_portfolio_id: Option<String>,
}

impl ListOrdersParams {
    /// Create new list orders parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by product ID.
    pub fn product_id(mut self, product_id: impl Into<String>) -> Self {
        self.product_ids = Some(product_id.into());
        self
    }

    /// Filter by order status.
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.order_status = Some(status.into());
        self
    }

    /// Set the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

/// Response from listing orders.
#[derive(Debug, Clone, Deserialize)]
pub struct ListOrdersResponse {
    /// The orders.
    pub orders: Vec<Order>,
    /// Sequence number.
    pub sequence: Option<String>,
    /// Whether there are more orders.
    pub has_next: bool,
    /// Cursor for the next page.
    pub cursor: Option<String>,
}

/// An order fill (execution).
#[derive(Debug, Clone, Deserialize)]
pub struct Fill {
    /// Entry ID.
    pub entry_id: String,
    /// Trade ID.
    pub trade_id: String,
    /// Order ID.
    pub order_id: String,
    /// Trade time.
    pub trade_time: String,
    /// Trade type.
    pub trade_type: String,
    /// Execution price.
    pub price: String,
    /// Execution size.
    pub size: String,
    /// Commission.
    pub commission: String,
    /// Product ID.
    pub product_id: String,
    /// Sequence timestamp.
    pub sequence_timestamp: Option<String>,
    /// Liquidity indicator.
    pub liquidity_indicator: Option<String>,
    /// Size in quote currency.
    pub size_in_quote: Option<bool>,
    /// User ID.
    pub user_id: Option<String>,
    /// Order side.
    pub side: Option<String>,
}

/// Parameters for listing fills.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListFillsParams {
    /// Filter by order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Filter by product ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// Start sequence timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_sequence_timestamp: Option<String>,
    /// End sequence timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_sequence_timestamp: Option<String>,
    /// Maximum number of fills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl ListFillsParams {
    /// Create new list fills parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by order ID.
    pub fn order_id(mut self, order_id: impl Into<String>) -> Self {
        self.order_id = Some(order_id.into());
        self
    }

    /// Filter by product ID.
    pub fn product_id(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self
    }

    /// Set the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

/// Response from listing fills.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFillsResponse {
    /// The fills.
    pub fills: Vec<Fill>,
    /// Cursor for the next page.
    pub cursor: Option<String>,
}

/// Request to close a position.
#[derive(Debug, Clone, Serialize)]
pub struct ClosePositionRequest {
    /// Client order ID.
    pub client_order_id: String,
    /// Product ID.
    pub product_id: String,
    /// Size to close (optional, closes entire position if not specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

impl ClosePositionRequest {
    /// Create a new close position request.
    pub fn new(client_order_id: impl Into<String>, product_id: impl Into<String>) -> Self {
        Self {
            client_order_id: client_order_id.into(),
            product_id: product_id.into(),
            size: None,
        }
    }

    /// Set the size to close.
    pub fn size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }
}
