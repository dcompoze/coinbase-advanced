//! Fees API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{TransactionSummary, TransactionSummaryParams};

/// API for retrieving fee information.
///
/// This API provides endpoints for querying fee tiers and transaction summaries.
pub struct FeesApi<'a> {
    client: &'a RestClient,
}

impl<'a> FeesApi<'a> {
    /// Create a new Fees API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// Get the transaction summary including fee tier information.
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
    /// let summary = client.fees().get_transaction_summary().await?;
    /// println!("Fee tier: {}", summary.fee_tier.pricing_tier);
    /// println!("Maker fee rate: {}", summary.fee_tier.maker_fee_rate);
    /// println!("Taker fee rate: {}", summary.fee_tier.taker_fee_rate);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_transaction_summary(&self) -> Result<TransactionSummary> {
        self.get_transaction_summary_with_params(TransactionSummaryParams::default())
            .await
    }

    /// Get the transaction summary with custom parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials, models::TransactionSummaryParams};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let params = TransactionSummaryParams::new()
    ///     .product_type("SPOT");
    ///
    /// let summary = client.fees()
    ///     .get_transaction_summary_with_params(params)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_transaction_summary_with_params(
        &self,
        params: TransactionSummaryParams,
    ) -> Result<TransactionSummary> {
        self.client
            .get_with_query("/transaction_summary", &params)
            .await
    }
}
