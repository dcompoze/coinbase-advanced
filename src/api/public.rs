//! Public (unauthenticated) API endpoints.

use serde::Deserialize;

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    Candle, GetCandlesParams, GetCandlesResponse, GetMarketTradesParams, GetMarketTradesResponse,
    GetProductBookParams, GetProductBookResponse, ListProductsParams, ListProductsResponse,
    Product, ProductBook,
};

/// Server time response.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerTime {
    /// ISO 8601 formatted time.
    pub iso: String,
    /// Unix epoch seconds.
    #[serde(rename = "epochSeconds")]
    pub epoch_seconds: String,
    /// Unix epoch milliseconds.
    #[serde(rename = "epochMillis")]
    pub epoch_millis: String,
}

/// API for accessing public (unauthenticated) endpoints.
///
/// These endpoints do not require API credentials and can be used
/// to access market data without authentication.
pub struct PublicApi<'a> {
    client: &'a RestClient,
}

impl<'a> PublicApi<'a> {
    /// Create a new Public API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// Get the current server time.
    ///
    /// This is useful for synchronizing your local clock with
    /// the Coinbase server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::RestClient;
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder().build()?;
    ///
    /// let time = client.public().get_time().await?;
    /// println!("Server time: {}", time.iso);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_time(&self) -> Result<ServerTime> {
        self.client.public_get("/time").await
    }

    /// List all public products.
    ///
    /// Similar to the authenticated products endpoint but does not
    /// require credentials.
    pub async fn list_products(&self, params: ListProductsParams) -> Result<ListProductsResponse> {
        self.client
            .public_get_with_query("/market/products", &params)
            .await
    }

    /// List all products with default parameters.
    pub async fn list_products_all(&self) -> Result<ListProductsResponse> {
        self.list_products(ListProductsParams::default()).await
    }

    /// Get a single product by ID.
    pub async fn get_product(&self, product_id: &str) -> Result<Product> {
        let endpoint = format!("/market/products/{}", product_id);
        self.client.public_get(&endpoint).await
    }

    /// Get the order book for a product.
    pub async fn get_product_book(&self, params: GetProductBookParams) -> Result<ProductBook> {
        let response: GetProductBookResponse = self
            .client
            .public_get_with_query("/market/product_book", &params)
            .await?;
        Ok(response.pricebook)
    }

    /// Get candlestick (OHLCV) data for a product.
    pub async fn get_candles(&self, params: GetCandlesParams) -> Result<Vec<Candle>> {
        let endpoint = format!("/market/products/{}/candles", params.product_id);
        let response: GetCandlesResponse = self
            .client
            .public_get_with_query(&endpoint, &params)
            .await?;
        Ok(response.candles)
    }

    /// Get recent trades for a product.
    pub async fn get_market_trades(
        &self,
        params: GetMarketTradesParams,
    ) -> Result<GetMarketTradesResponse> {
        let endpoint = format!("/market/products/{}/ticker", params.product_id);
        self.client.public_get_with_query(&endpoint, &params).await
    }
}
