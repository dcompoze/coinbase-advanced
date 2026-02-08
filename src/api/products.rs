//! Products API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    Candle, GetBestBidAskParams, GetBestBidAskResponse, GetCandlesParams, GetCandlesResponse,
    GetMarketTradesParams, GetMarketTradesResponse, GetProductBookParams, GetProductBookResponse,
    ListProductsParams, ListProductsResponse, Product, ProductBook,
};

/// API for accessing product and market data.
///
/// Products represent trading pairs (e.g., BTC-USD).
/// This API provides access to product information, order books,
/// candles, and recent trades.
pub struct ProductsApi<'a> {
    client: &'a RestClient,
}

impl<'a> ProductsApi<'a> {
    /// Create a new Products API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// List all products.
    ///
    /// Returns a list of available trading pairs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::ListProductsParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let products = client.products().list(ListProductsParams::new().limit(10)).await?;
    /// for product in products.products {
    ///     println!("{}: {} @ {}", product.product_id, product.base_name, product.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, params: ListProductsParams) -> Result<ListProductsResponse> {
        self.client.get_with_query("/products", &params).await
    }

    /// List all products with default parameters.
    pub async fn list_all(&self) -> Result<ListProductsResponse> {
        self.list(ListProductsParams::default()).await
    }

    /// Get a single product by ID.
    ///
    /// # Arguments
    ///
    /// * `product_id` - The product identifier (e.g., "BTC-USD").
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
    /// let product = client.products().get("BTC-USD").await?;
    /// println!("BTC price: ${}", product.price);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, product_id: &str) -> Result<Product> {
        let endpoint = format!("/products/{}", product_id);
        self.client.get(&endpoint).await
    }

    /// Get the order book for a product.
    ///
    /// Returns the current bids and asks for the specified product.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::GetProductBookParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let book = client.products()
    ///     .get_book(GetProductBookParams::new("BTC-USD").limit(10))
    ///     .await?;
    ///
    /// println!("Best bid: {}", book.bids.first().map(|b| &b.price).unwrap_or(&"N/A".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_book(&self, params: GetProductBookParams) -> Result<ProductBook> {
        let response: GetProductBookResponse =
            self.client.get_with_query("/product_book", &params).await?;
        Ok(response.pricebook)
    }

    /// Get the best bid/ask for one or more products.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::GetBestBidAskParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let response = client.products()
    ///     .get_best_bid_ask(GetBestBidAskParams::new().product_ids(&["BTC-USD", "ETH-USD"]))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_best_bid_ask(
        &self,
        params: GetBestBidAskParams,
    ) -> Result<GetBestBidAskResponse> {
        self.client.get_with_query("/best_bid_ask", &params).await
    }

    /// Get candlestick (OHLCV) data for a product.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::{GetCandlesParams, Granularity}};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let candles = client.products()
    ///     .get_candles(GetCandlesParams::new(
    ///         "BTC-USD",
    ///         "1704067200",  // Start timestamp
    ///         "1704153600",  // End timestamp
    ///         Granularity::OneHour
    ///     ))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_candles(&self, params: GetCandlesParams) -> Result<Vec<Candle>> {
        let endpoint = format!("/products/{}/candles", params.product_id);
        let response: GetCandlesResponse =
            self.client.get_with_query(&endpoint, &params).await?;
        Ok(response.candles)
    }

    /// Get recent trades for a product.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::GetMarketTradesParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let response = client.products()
    ///     .get_market_trades(GetMarketTradesParams::new("BTC-USD", 10))
    ///     .await?;
    ///
    /// for trade in response.trades {
    ///     println!("{} {} @ {}", trade.side, trade.size, trade.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_market_trades(
        &self,
        params: GetMarketTradesParams,
    ) -> Result<GetMarketTradesResponse> {
        let endpoint = format!("/products/{}/ticker", params.product_id);
        self.client.get_with_query(&endpoint, &params).await
    }
}
