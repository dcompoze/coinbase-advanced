//! Perpetuals/INTX API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    AllocatePortfolioRequest, GetPerpetualsPortfolioSummaryResponse, GetPerpetualsPositionResponse,
    GetPortfolioBalancesResponse, IntxPortfolioSummary, IntxPosition,
    ListPerpetualsPositionsResponse, SetMultiAssetCollateralRequest,
};

/// API for perpetuals (INTX) trading.
///
/// This API provides endpoints for managing perpetual futures positions and portfolios.
pub struct PerpetualsApi<'a> {
    client: &'a RestClient,
}

impl<'a> PerpetualsApi<'a> {
    /// Create a new Perpetuals API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// List all perpetuals positions for a portfolio.
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
    /// let response = client.perpetuals().list_positions("portfolio-uuid").await?;
    /// for position in response.positions {
    ///     println!("{}: {:?}", position.product_id, position.net_size);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_positions(
        &self,
        portfolio_uuid: &str,
    ) -> Result<ListPerpetualsPositionsResponse> {
        let endpoint = format!("/intx/positions/{}", portfolio_uuid);
        self.client.get(&endpoint).await
    }

    /// Get a specific perpetuals position.
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
    /// let position = client.perpetuals()
    ///     .get_position("portfolio-uuid", "BTC-PERP")
    ///     .await?;
    /// println!("Position: {:?}", position.net_size);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_position(&self, portfolio_uuid: &str, symbol: &str) -> Result<IntxPosition> {
        let endpoint = format!("/intx/positions/{}/{}", portfolio_uuid, symbol);
        let response: GetPerpetualsPositionResponse = self.client.get(&endpoint).await?;
        Ok(response.position)
    }

    /// Get portfolio balances for perpetuals.
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
    /// let response = client.perpetuals()
    ///     .get_portfolio_balances("portfolio-uuid")
    ///     .await?;
    /// for balance in response.portfolio_balances {
    ///     println!("{:?}: {:?}", balance.asset, balance.quantity);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_portfolio_balances(
        &self,
        portfolio_uuid: &str,
    ) -> Result<GetPortfolioBalancesResponse> {
        let endpoint = format!("/intx/balances/{}", portfolio_uuid);
        self.client.get(&endpoint).await
    }

    /// Get portfolio summary for perpetuals.
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
    /// let summary = client.perpetuals()
    ///     .get_portfolio_summary("portfolio-uuid")
    ///     .await?;
    /// println!("Total balance: {:?}", summary.total_balance);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_portfolio_summary(
        &self,
        portfolio_uuid: &str,
    ) -> Result<IntxPortfolioSummary> {
        let endpoint = format!("/intx/portfolio/{}", portfolio_uuid);
        let response: GetPerpetualsPortfolioSummaryResponse = self.client.get(&endpoint).await?;
        Ok(response.summary)
    }

    /// Allocate funds to a perpetuals portfolio.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::AllocatePortfolioRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = AllocatePortfolioRequest::new(
    ///     "portfolio-uuid",
    ///     "BTC-PERP",
    ///     "1000",
    ///     "USD",
    /// );
    ///
    /// client.perpetuals().allocate(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn allocate(&self, request: AllocatePortfolioRequest) -> Result<()> {
        let _response: serde_json::Value = self.client.post("/intx/allocate", &request).await?;
        Ok(())
    }

    /// Set multi-asset collateral for a portfolio.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::SetMultiAssetCollateralRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = SetMultiAssetCollateralRequest::new(true);
    /// client.perpetuals()
    ///     .set_multi_asset_collateral("portfolio-uuid", request)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_multi_asset_collateral(
        &self,
        portfolio_uuid: &str,
        request: SetMultiAssetCollateralRequest,
    ) -> Result<()> {
        let endpoint = format!("/intx/balances/{}", portfolio_uuid);
        let _response: serde_json::Value = self.client.post(&endpoint, &request).await?;
        Ok(())
    }
}
