//! Orders API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    CancelOrdersRequest, CancelOrdersResponse, ClosePositionRequest, CreateOrderRequest,
    CreateOrderResponse, EditOrderRequest, EditOrderResponse, ListFillsParams,
    ListFillsResponse, ListOrdersParams, ListOrdersResponse, Order,
};

/// Response from getting a single order.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GetOrderResponse {
    /// The order.
    pub order: Order,
}

/// API for managing orders.
///
/// This API provides endpoints for creating, editing, cancelling,
/// and querying orders.
pub struct OrdersApi<'a> {
    client: &'a RestClient,
}

impl<'a> OrdersApi<'a> {
    /// Create a new Orders API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// Create a new order.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::{CreateOrderRequest, OrderSide, OrderConfiguration}};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// // Create a market buy order for $100 of BTC
    /// let request = CreateOrderRequest::new(
    ///     "unique-client-order-id", // Use a UUID or unique identifier
    ///     "BTC-USD",
    ///     OrderSide::Buy,
    ///     OrderConfiguration::market_buy_quote("100"),
    /// );
    ///
    /// let response = client.orders().create(request).await?;
    /// if response.success {
    ///     println!("Order created: {:?}", response.order_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: CreateOrderRequest) -> Result<CreateOrderResponse> {
        self.client.post("/orders", &request).await
    }

    /// Preview an order without executing it.
    ///
    /// Returns the expected fees and total for the order.
    pub async fn preview(&self, request: CreateOrderRequest) -> Result<serde_json::Value> {
        self.client.post("/orders/preview", &request).await
    }

    /// Edit an existing order.
    ///
    /// Only the price and/or size can be modified.
    pub async fn edit(&self, request: EditOrderRequest) -> Result<EditOrderResponse> {
        self.client.post("/orders/edit", &request).await
    }

    /// Preview an order edit.
    pub async fn preview_edit(&self, request: EditOrderRequest) -> Result<serde_json::Value> {
        self.client.post("/orders/edit_preview", &request).await
    }

    /// Cancel one or more orders.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::CancelOrdersRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// // Cancel a single order
    /// let response = client.orders()
    ///     .cancel(CancelOrdersRequest::single("order-id"))
    ///     .await?;
    ///
    /// // Cancel multiple orders
    /// let response = client.orders()
    ///     .cancel(CancelOrdersRequest::new(vec![
    ///         "order-1".to_string(),
    ///         "order-2".to_string(),
    ///     ]))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel(&self, request: CancelOrdersRequest) -> Result<CancelOrdersResponse> {
        self.client.post("/orders/batch_cancel", &request).await
    }

    /// List orders.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::ListOrdersParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// // List open orders for BTC-USD
    /// let orders = client.orders()
    ///     .list(ListOrdersParams::new()
    ///         .product_id("BTC-USD")
    ///         .status("OPEN")
    ///         .limit(10))
    ///     .await?;
    ///
    /// for order in orders.orders {
    ///     println!("{}: {} {}", order.order_id, order.side, order.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, params: ListOrdersParams) -> Result<ListOrdersResponse> {
        self.client
            .get_with_query("/orders/historical/batch", &params)
            .await
    }

    /// List all orders with default parameters.
    pub async fn list_all(&self) -> Result<ListOrdersResponse> {
        self.list(ListOrdersParams::default()).await
    }

    /// Get a single order by ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let order = client.orders().get("order-id").await?;
    /// println!("Order status: {}", order.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, order_id: &str) -> Result<Order> {
        let endpoint = format!("/orders/historical/{}", order_id);
        let response: GetOrderResponse = self.client.get(&endpoint).await?;
        Ok(response.order)
    }

    /// List order fills (executions).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::ListFillsParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// // Get fills for a specific order
    /// let fills = client.orders()
    ///     .list_fills(ListFillsParams::new().order_id("order-id"))
    ///     .await?;
    ///
    /// for fill in fills.fills {
    ///     println!("Filled {} @ {}", fill.size, fill.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_fills(&self, params: ListFillsParams) -> Result<ListFillsResponse> {
        self.client
            .get_with_query("/orders/historical/fills", &params)
            .await
    }

    /// Close a position.
    ///
    /// This creates a market order to close an existing position.
    pub async fn close_position(&self, request: ClosePositionRequest) -> Result<CreateOrderResponse> {
        self.client.post("/orders/close_position", &request).await
    }
}
