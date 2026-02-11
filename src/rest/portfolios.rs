//! Portfolios API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    CreatePortfolioRequest, EditPortfolioRequest, GetPortfolioBreakdownResponse,
    ListPortfoliosParams, ListPortfoliosResponse, MoveFundsRequest, MoveFundsResponse, Portfolio,
    PortfolioBreakdown, PortfolioResponse,
};

/// API for managing portfolios.
///
/// This API provides endpoints for creating, editing, deleting, and querying portfolios.
pub struct PortfoliosApi<'a> {
    client: &'a RestClient,
}

impl<'a> PortfoliosApi<'a> {
    /// Create a new Portfolios API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// List all portfolios.
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
    /// let portfolios = client.portfolios().list().await?;
    /// for portfolio in portfolios {
    ///     println!("{}: {}", portfolio.uuid, portfolio.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<Portfolio>> {
        self.list_with_params(ListPortfoliosParams::default()).await
    }

    /// List portfolios with custom parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::ListPortfoliosParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let params = ListPortfoliosParams::new().portfolio_type("CONSUMER");
    /// let portfolios = client.portfolios().list_with_params(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_with_params(&self, params: ListPortfoliosParams) -> Result<Vec<Portfolio>> {
        let response: ListPortfoliosResponse = self
            .client
            .get_with_query("/portfolios", &params)
            .await?;
        Ok(response.portfolios)
    }

    /// Get a portfolio breakdown by UUID.
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
    /// let breakdown = client.portfolios().get_breakdown("portfolio-uuid").await?;
    /// println!("Portfolio: {}", breakdown.portfolio.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_breakdown(&self, portfolio_uuid: &str) -> Result<PortfolioBreakdown> {
        let endpoint = format!("/portfolios/{}", portfolio_uuid);
        let response: GetPortfolioBreakdownResponse = self.client.get(&endpoint).await?;
        Ok(response.breakdown)
    }

    /// Create a new portfolio.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::CreatePortfolioRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let portfolio = client.portfolios()
    ///     .create(CreatePortfolioRequest::new("My New Portfolio"))
    ///     .await?;
    /// println!("Created portfolio: {}", portfolio.uuid);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: CreatePortfolioRequest) -> Result<Portfolio> {
        let response: PortfolioResponse = self.client.post("/portfolios", &request).await?;
        Ok(response.portfolio)
    }

    /// Edit an existing portfolio.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::EditPortfolioRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let portfolio = client.portfolios()
    ///     .edit("portfolio-uuid", EditPortfolioRequest::new("New Name"))
    ///     .await?;
    /// println!("Updated portfolio: {}", portfolio.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn edit(
        &self,
        portfolio_uuid: &str,
        request: EditPortfolioRequest,
    ) -> Result<Portfolio> {
        let endpoint = format!("/portfolios/{}", portfolio_uuid);
        let response: PortfolioResponse = self.client.put(&endpoint, &request).await?;
        Ok(response.portfolio)
    }

    /// Delete a portfolio.
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
    /// client.portfolios().delete("portfolio-uuid").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, portfolio_uuid: &str) -> Result<()> {
        let endpoint = format!("/portfolios/{}", portfolio_uuid);
        let _response: serde_json::Value = self.client.delete(&endpoint).await?;
        Ok(())
    }

    /// Move funds between portfolios.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::{MoveFundsRequest, MoveFunds}};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = MoveFundsRequest::new(
    ///     MoveFunds::new("100.00", "USD"),
    ///     "source-portfolio-uuid",
    ///     "target-portfolio-uuid",
    /// );
    ///
    /// let response = client.portfolios().move_funds(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn move_funds(&self, request: MoveFundsRequest) -> Result<MoveFundsResponse> {
        self.client.post("/portfolios/move_funds", &request).await
    }
}
