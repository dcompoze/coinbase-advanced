//! Convert API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    CommitConvertTradeRequest, ConvertTrade, ConvertTradeResponse, CreateConvertQuoteRequest,
    GetConvertTradeParams,
};

/// API for currency conversion.
///
/// This API provides endpoints for creating conversion quotes, committing trades,
/// and retrieving trade details.
pub struct ConvertApi<'a> {
    client: &'a RestClient,
}

impl<'a> ConvertApi<'a> {
    /// Create a new Convert API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// Create a convert quote.
    ///
    /// This creates a quote for converting between two currencies. The quote
    /// can then be committed to execute the conversion.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::CreateConvertQuoteRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = CreateConvertQuoteRequest::new(
    ///     "USD-account-id",
    ///     "USDC-account-id",
    ///     "100.00",
    /// );
    ///
    /// let quote = client.convert().create_quote(request).await?;
    /// println!("Quote ID: {}", quote.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_quote(&self, request: CreateConvertQuoteRequest) -> Result<ConvertTrade> {
        let response: ConvertTradeResponse = self.client.post("/convert/quote", &request).await?;
        Ok(response.trade)
    }

    /// Commit a convert trade.
    ///
    /// This executes a previously created conversion quote.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::CommitConvertTradeRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = CommitConvertTradeRequest::new(
    ///     "USD-account-id",
    ///     "USDC-account-id",
    /// );
    ///
    /// let trade = client.convert().commit_trade("trade-id", request).await?;
    /// println!("Trade status: {:?}", trade.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn commit_trade(
        &self,
        trade_id: &str,
        request: CommitConvertTradeRequest,
    ) -> Result<ConvertTrade> {
        let endpoint = format!("/convert/trade/{}", trade_id);
        let response: ConvertTradeResponse = self.client.post(&endpoint, &request).await?;
        Ok(response.trade)
    }

    /// Get a convert trade.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::GetConvertTradeParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let params = GetConvertTradeParams::new(
    ///     "USD-account-id",
    ///     "USDC-account-id",
    /// );
    ///
    /// let trade = client.convert().get_trade("trade-id", params).await?;
    /// println!("Trade status: {:?}", trade.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_trade(
        &self,
        trade_id: &str,
        params: GetConvertTradeParams,
    ) -> Result<ConvertTrade> {
        let endpoint = format!("/convert/trade/{}", trade_id);
        let response: ConvertTradeResponse = self
            .client
            .get_with_query(&endpoint, &params)
            .await?;
        Ok(response.trade)
    }
}
