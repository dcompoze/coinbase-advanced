//! Ergonomic order builder APIs.
//!
//! These builders provide a more convenient way to construct orders
//! compared to manually creating `CreateOrderRequest` objects.

use crate::client::RestClient;
use crate::error::{Error, Result};
use crate::models::{CreateOrderRequest, CreateOrderResponse, OrderConfiguration, OrderSide, StopDirection};

/// Builder for market orders.
pub struct MarketOrderBuilder<'a> {
    client: &'a RestClient,
    product_id: Option<String>,
    side: Option<OrderSide>,
    quote_size: Option<String>,
    base_size: Option<String>,
    client_order_id: Option<String>,
}

impl<'a> MarketOrderBuilder<'a> {
    /// Create a new market order builder.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self {
            client,
            product_id: None,
            side: None,
            quote_size: None,
            base_size: None,
            client_order_id: None,
        }
    }

    /// Set the product ID.
    pub fn product_id(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self
    }

    /// Set as a buy order.
    pub fn buy(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Buy);
        self
    }

    /// Set as a sell order.
    pub fn sell(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Sell);
        self
    }

    /// Set the quote size (amount in quote currency, e.g., USD).
    pub fn quote_size(mut self, quote_size: impl Into<String>) -> Self {
        self.quote_size = Some(quote_size.into());
        self
    }

    /// Set the base size (amount in base currency, e.g., BTC).
    pub fn base_size(mut self, base_size: impl Into<String>) -> Self {
        self.base_size = Some(base_size.into());
        self
    }

    /// Set a custom client order ID.
    pub fn client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    /// Build and send the order.
    pub async fn send(self) -> Result<CreateOrderResponse> {
        let product_id = self.product_id
            .ok_or_else(|| Error::request("product_id is required"))?;
        let side = self.side
            .ok_or_else(|| Error::request("side is required (use .buy() or .sell())"))?;

        let config = if self.quote_size.is_some() {
            OrderConfiguration::market_buy_quote(self.quote_size.unwrap())
        } else if self.base_size.is_some() {
            if side == OrderSide::Buy {
                OrderConfiguration::market_buy_base(self.base_size.unwrap())
            } else {
                OrderConfiguration::market_sell(self.base_size.unwrap())
            }
        } else {
            return Err(Error::request("either quote_size or base_size is required"));
        };

        let client_order_id = self.client_order_id
            .unwrap_or_else(uuid_v4);

        let request = CreateOrderRequest::new(client_order_id, product_id, side, config);
        self.client.orders().create(request).await
    }
}

/// Builder for limit GTC (good-til-cancelled) orders.
pub struct LimitOrderGtcBuilder<'a> {
    client: &'a RestClient,
    product_id: Option<String>,
    side: Option<OrderSide>,
    base_size: Option<String>,
    limit_price: Option<String>,
    post_only: bool,
    client_order_id: Option<String>,
}

impl<'a> LimitOrderGtcBuilder<'a> {
    /// Create a new limit order GTC builder.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self {
            client,
            product_id: None,
            side: None,
            base_size: None,
            limit_price: None,
            post_only: false,
            client_order_id: None,
        }
    }

    /// Set as a buy order.
    pub fn buy(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Buy);
        self
    }

    /// Set as a sell order.
    pub fn sell(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Sell);
        self
    }

    /// Set the base size.
    pub fn base_size(mut self, base_size: impl Into<String>) -> Self {
        self.base_size = Some(base_size.into());
        self
    }

    /// Set the limit price.
    pub fn limit_price(mut self, limit_price: impl Into<String>) -> Self {
        self.limit_price = Some(limit_price.into());
        self
    }

    /// Set post-only mode (only add liquidity).
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = post_only;
        self
    }

    /// Set a custom client order ID.
    pub fn client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    /// Build and send the order.
    pub async fn send(self) -> Result<CreateOrderResponse> {
        let product_id = self.product_id
            .ok_or_else(|| Error::request("product_id is required"))?;
        let side = self.side
            .ok_or_else(|| Error::request("side is required (use .buy() or .sell())"))?;
        let base_size = self.base_size
            .ok_or_else(|| Error::request("base_size is required"))?;
        let limit_price = self.limit_price
            .ok_or_else(|| Error::request("limit_price is required"))?;

        let config = OrderConfiguration::limit_gtc(base_size, limit_price, self.post_only);
        let client_order_id = self.client_order_id.unwrap_or_else(uuid_v4);

        let request = CreateOrderRequest::new(client_order_id, product_id, side, config);
        self.client.orders().create(request).await
    }
}

/// Builder for limit GTD (good-til-date) orders.
pub struct LimitOrderGtdBuilder<'a> {
    client: &'a RestClient,
    product_id: Option<String>,
    side: Option<OrderSide>,
    base_size: Option<String>,
    limit_price: Option<String>,
    end_time: Option<String>,
    post_only: bool,
    client_order_id: Option<String>,
}

impl<'a> LimitOrderGtdBuilder<'a> {
    /// Create a new limit order GTD builder.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self {
            client,
            product_id: None,
            side: None,
            base_size: None,
            limit_price: None,
            end_time: None,
            post_only: false,
            client_order_id: None,
        }
    }

    /// Set as a buy order.
    pub fn buy(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Buy);
        self
    }

    /// Set as a sell order.
    pub fn sell(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Sell);
        self
    }

    /// Set the base size.
    pub fn base_size(mut self, base_size: impl Into<String>) -> Self {
        self.base_size = Some(base_size.into());
        self
    }

    /// Set the limit price.
    pub fn limit_price(mut self, limit_price: impl Into<String>) -> Self {
        self.limit_price = Some(limit_price.into());
        self
    }

    /// Set the end time (ISO 8601 format).
    pub fn end_time(mut self, end_time: impl Into<String>) -> Self {
        self.end_time = Some(end_time.into());
        self
    }

    /// Set post-only mode (only add liquidity).
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = post_only;
        self
    }

    /// Set a custom client order ID.
    pub fn client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    /// Build and send the order.
    pub async fn send(self) -> Result<CreateOrderResponse> {
        let product_id = self.product_id
            .ok_or_else(|| Error::request("product_id is required"))?;
        let side = self.side
            .ok_or_else(|| Error::request("side is required (use .buy() or .sell())"))?;
        let base_size = self.base_size
            .ok_or_else(|| Error::request("base_size is required"))?;
        let limit_price = self.limit_price
            .ok_or_else(|| Error::request("limit_price is required"))?;
        let end_time = self.end_time
            .ok_or_else(|| Error::request("end_time is required"))?;

        let config = OrderConfiguration::limit_gtd(base_size, limit_price, end_time, self.post_only);
        let client_order_id = self.client_order_id.unwrap_or_else(uuid_v4);

        let request = CreateOrderRequest::new(client_order_id, product_id, side, config);
        self.client.orders().create(request).await
    }
}

/// Builder for stop-limit GTC orders.
pub struct StopLimitOrderGtcBuilder<'a> {
    client: &'a RestClient,
    product_id: Option<String>,
    side: Option<OrderSide>,
    base_size: Option<String>,
    limit_price: Option<String>,
    stop_price: Option<String>,
    stop_direction: Option<StopDirection>,
    client_order_id: Option<String>,
}

impl<'a> StopLimitOrderGtcBuilder<'a> {
    /// Create a new stop-limit order GTC builder.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self {
            client,
            product_id: None,
            side: None,
            base_size: None,
            limit_price: None,
            stop_price: None,
            stop_direction: None,
            client_order_id: None,
        }
    }

    /// Set as a buy order.
    pub fn buy(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Buy);
        self
    }

    /// Set as a sell order.
    pub fn sell(mut self, product_id: impl Into<String>) -> Self {
        self.product_id = Some(product_id.into());
        self.side = Some(OrderSide::Sell);
        self
    }

    /// Set the base size.
    pub fn base_size(mut self, base_size: impl Into<String>) -> Self {
        self.base_size = Some(base_size.into());
        self
    }

    /// Set the limit price.
    pub fn limit_price(mut self, limit_price: impl Into<String>) -> Self {
        self.limit_price = Some(limit_price.into());
        self
    }

    /// Set the stop price.
    pub fn stop_price(mut self, stop_price: impl Into<String>) -> Self {
        self.stop_price = Some(stop_price.into());
        self
    }

    /// Set the stop direction.
    pub fn stop_direction(mut self, stop_direction: StopDirection) -> Self {
        self.stop_direction = Some(stop_direction);
        self
    }

    /// Set a custom client order ID.
    pub fn client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    /// Build and send the order.
    pub async fn send(self) -> Result<CreateOrderResponse> {
        let product_id = self.product_id
            .ok_or_else(|| Error::request("product_id is required"))?;
        let side = self.side
            .ok_or_else(|| Error::request("side is required (use .buy() or .sell())"))?;
        let base_size = self.base_size
            .ok_or_else(|| Error::request("base_size is required"))?;
        let limit_price = self.limit_price
            .ok_or_else(|| Error::request("limit_price is required"))?;
        let stop_price = self.stop_price
            .ok_or_else(|| Error::request("stop_price is required"))?;
        let stop_direction = self.stop_direction
            .ok_or_else(|| Error::request("stop_direction is required"))?;

        let config = OrderConfiguration::stop_limit_gtc(base_size, limit_price, stop_price, stop_direction);
        let client_order_id = self.client_order_id.unwrap_or_else(uuid_v4);

        let request = CreateOrderRequest::new(client_order_id, product_id, side, config);
        self.client.orders().create(request).await
    }
}

/// Generate a simple UUID v4 string.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    // Simple UUID-like format using timestamp and random bits
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (now >> 96) as u32,
        (now >> 80) as u16,
        (now >> 68) as u16 & 0x0fff,
        ((now >> 52) as u16 & 0x3fff) | 0x8000,
        now as u64 & 0xffffffffffff
    )
}

// Add builder methods to RestClient
impl RestClient {
    /// Create a market order builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// // Buy $100 of BTC
    /// let response = client.market_order()
    ///     .buy("BTC-USD")
    ///     .quote_size("100.00")
    ///     .send()
    ///     .await?;
    ///
    /// // Sell 0.001 BTC
    /// let response = client.market_order()
    ///     .sell("BTC-USD")
    ///     .base_size("0.001")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn market_order(&self) -> MarketOrderBuilder<'_> {
        MarketOrderBuilder::new(self)
    }

    /// Create a limit order (GTC) builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let response = client.limit_order_gtc()
    ///     .buy("BTC-USD")
    ///     .base_size("0.001")
    ///     .limit_price("50000.00")
    ///     .post_only(true)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn limit_order_gtc(&self) -> LimitOrderGtcBuilder<'_> {
        LimitOrderGtcBuilder::new(self)
    }

    /// Create a limit order (GTD) builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let response = client.limit_order_gtd()
    ///     .buy("BTC-USD")
    ///     .base_size("0.001")
    ///     .limit_price("50000.00")
    ///     .end_time("2024-12-31T23:59:59Z")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn limit_order_gtd(&self) -> LimitOrderGtdBuilder<'_> {
        LimitOrderGtdBuilder::new(self)
    }

    /// Create a stop-limit order (GTC) builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials, models::StopDirection};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let response = client.stop_limit_order_gtc()
    ///     .sell("BTC-USD")
    ///     .base_size("0.001")
    ///     .limit_price("49000.00")
    ///     .stop_price("50000.00")
    ///     .stop_direction(StopDirection::StopDirectionStopDown)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn stop_limit_order_gtc(&self) -> StopLimitOrderGtcBuilder<'_> {
        StopLimitOrderGtcBuilder::new(self)
    }
}
