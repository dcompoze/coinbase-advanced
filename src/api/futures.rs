//! Futures/CFM API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{
    FuturesBalanceSummary, FuturesPosition, FuturesSweep, GetCurrentMarginWindowParams,
    GetCurrentMarginWindowResponse, GetFuturesBalanceSummaryResponse, GetFuturesPositionResponse,
    GetIntradayMarginSettingResponse, ListFuturesPositionsResponse, ListFuturesSweepsResponse,
    MarginWindow, ScheduleFuturesSweepRequest, ScheduleFuturesSweepResponse,
    SetIntradayMarginSettingRequest,
};

/// API for futures (CFM) trading.
///
/// This API provides endpoints for managing futures positions, balances, margins, and sweeps.
pub struct FuturesApi<'a> {
    client: &'a RestClient,
}

impl<'a> FuturesApi<'a> {
    /// Create a new Futures API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// List all futures positions.
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
    /// let positions = client.futures().list_positions().await?;
    /// for position in positions {
    ///     println!("{}: {:?}", position.product_id, position.number_of_contracts);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_positions(&self) -> Result<Vec<FuturesPosition>> {
        let response: ListFuturesPositionsResponse = self.client.get("/cfm/positions").await?;
        Ok(response.positions)
    }

    /// Get a specific futures position.
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
    /// let position = client.futures().get_position("BIT-28JUN24-CDE").await?;
    /// println!("Position: {:?}", position.number_of_contracts);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_position(&self, product_id: &str) -> Result<FuturesPosition> {
        let endpoint = format!("/cfm/positions/{}", product_id);
        let response: GetFuturesPositionResponse = self.client.get(&endpoint).await?;
        Ok(response.position)
    }

    /// Get the futures balance summary.
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
    /// let summary = client.futures().get_balance_summary().await?;
    /// println!("Buying power: {:?}", summary.futures_buying_power);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_balance_summary(&self) -> Result<FuturesBalanceSummary> {
        let response: GetFuturesBalanceSummaryResponse =
            self.client.get("/cfm/balance_summary").await?;
        Ok(response.balance_summary)
    }

    /// Get the intraday margin setting.
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
    /// let response = client.futures().get_intraday_margin_setting().await?;
    /// println!("Setting: {:?}", response.setting);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_intraday_margin_setting(&self) -> Result<GetIntradayMarginSettingResponse> {
        self.client.get("/cfm/intraday/margin_setting").await
    }

    /// Set the intraday margin setting.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials, models::SetIntradayMarginSettingRequest};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = SetIntradayMarginSettingRequest::new("STANDARD");
    /// client.futures().set_intraday_margin_setting(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_intraday_margin_setting(
        &self,
        request: SetIntradayMarginSettingRequest,
    ) -> Result<()> {
        let _response: serde_json::Value = self
            .client
            .post("/cfm/intraday/margin_setting", &request)
            .await?;
        Ok(())
    }

    /// Get the current margin window.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials, models::GetCurrentMarginWindowParams};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let window = client.futures()
    ///     .get_current_margin_window(GetCurrentMarginWindowParams::new())
    ///     .await?;
    /// println!("Window type: {:?}", window.margin_window_type);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_current_margin_window(
        &self,
        params: GetCurrentMarginWindowParams,
    ) -> Result<MarginWindow> {
        let response: GetCurrentMarginWindowResponse = self
            .client
            .get_with_query("/cfm/intraday/current_margin_window", &params)
            .await?;
        Ok(response.margin_window)
    }

    /// List all scheduled futures sweeps.
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
    /// let sweeps = client.futures().list_sweeps().await?;
    /// for sweep in sweeps {
    ///     println!("Sweep: {:?} - {:?}", sweep.id, sweep.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_sweeps(&self) -> Result<Vec<FuturesSweep>> {
        let response: ListFuturesSweepsResponse = self.client.get("/cfm/sweeps").await?;
        Ok(response.sweeps)
    }

    /// Schedule a futures sweep.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials, models::ScheduleFuturesSweepRequest};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = ScheduleFuturesSweepRequest::new("1000.00");
    /// let response = client.futures().schedule_sweep(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn schedule_sweep(
        &self,
        request: ScheduleFuturesSweepRequest,
    ) -> Result<ScheduleFuturesSweepResponse> {
        self.client.post("/cfm/sweeps/schedule", &request).await
    }

    /// Cancel a pending futures sweep.
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
    /// client.futures().cancel_pending_sweep().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_pending_sweep(&self) -> Result<()> {
        let _response: serde_json::Value = self.client.delete("/cfm/sweeps").await?;
        Ok(())
    }
}
