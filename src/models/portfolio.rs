//! Portfolio-related types.

use serde::{Deserialize, Serialize};

/// Portfolio type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PortfolioType {
    /// User's default portfolio.
    Default,
    /// Portfolios created by the user.
    Consumer,
    /// International Exchange portfolios.
    Intx,
    /// Unknown/undefined portfolio type.
    #[serde(other)]
    Undefined,
}

/// A user's portfolio.
#[derive(Debug, Clone, Deserialize)]
pub struct Portfolio {
    /// Name of the portfolio.
    pub name: String,
    /// UUID of the portfolio.
    pub uuid: String,
    /// Type of the portfolio.
    #[serde(rename = "type")]
    pub portfolio_type: PortfolioType,
    /// Whether the portfolio is deleted.
    #[serde(default)]
    pub deleted: bool,
}

/// Balance information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioBalance {
    /// The balance value.
    pub value: String,
    /// The currency.
    pub currency: String,
}

/// Portfolio balances breakdown.
#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioBalances {
    /// Total balance.
    pub total_balance: PortfolioBalance,
    /// Total futures balance.
    #[serde(default)]
    pub total_futures_balance: Option<PortfolioBalance>,
    /// Total cash equivalent balance.
    #[serde(default)]
    pub total_cash_equivalent_balance: Option<PortfolioBalance>,
    /// Total crypto balance.
    #[serde(default)]
    pub total_crypto_balance: Option<PortfolioBalance>,
    /// Futures unrealized PnL.
    #[serde(default)]
    pub futures_unrealized_pnl: Option<PortfolioBalance>,
    /// Perpetuals unrealized PnL.
    #[serde(default)]
    pub perp_unrealized_pnl: Option<PortfolioBalance>,
}

/// Spot position in a portfolio.
#[derive(Debug, Clone, Deserialize)]
pub struct SpotPosition {
    /// The asset symbol (e.g., BTC, ETH).
    pub asset: String,
    /// The account UUID.
    pub account_uuid: String,
    /// Total balance in fiat.
    #[serde(default)]
    pub total_balance_fiat: f64,
    /// Total balance in crypto.
    #[serde(default)]
    pub total_balance_crypto: f64,
    /// Available to trade in fiat.
    #[serde(default)]
    pub available_to_trade_fiat: f64,
    /// Portfolio allocation percentage.
    #[serde(default)]
    pub allocation: f64,
    /// Cost basis.
    #[serde(default)]
    pub cost_basis: Option<PortfolioBalance>,
    /// Asset image URL.
    #[serde(default)]
    pub asset_img_url: Option<String>,
    /// Whether this is a cash position.
    #[serde(default)]
    pub is_cash: bool,
}

/// Portfolio breakdown with positions.
#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioBreakdown {
    /// The portfolio.
    pub portfolio: Portfolio,
    /// Portfolio balances.
    #[serde(default)]
    pub portfolio_balances: Option<PortfolioBalances>,
    /// Spot positions.
    #[serde(default)]
    pub spot_positions: Vec<SpotPosition>,
    /// Perpetual positions (raw JSON for flexibility).
    #[serde(default)]
    pub perp_positions: Vec<serde_json::Value>,
    /// Futures positions (raw JSON for flexibility).
    #[serde(default)]
    pub futures_positions: Vec<serde_json::Value>,
}

/// Response containing a list of portfolios.
#[derive(Debug, Clone, Deserialize)]
pub struct ListPortfoliosResponse {
    /// The portfolios.
    pub portfolios: Vec<Portfolio>,
}

/// Response containing a portfolio breakdown.
#[derive(Debug, Clone, Deserialize)]
pub struct GetPortfolioBreakdownResponse {
    /// The breakdown.
    pub breakdown: PortfolioBreakdown,
}

/// Response containing a single portfolio.
#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioResponse {
    /// The portfolio.
    pub portfolio: Portfolio,
}

/// Parameters for listing portfolios.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListPortfoliosParams {
    /// Filter by portfolio type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portfolio_type: Option<String>,
}

impl ListPortfoliosParams {
    /// Create new list portfolios parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by portfolio type.
    pub fn portfolio_type(mut self, portfolio_type: impl Into<String>) -> Self {
        self.portfolio_type = Some(portfolio_type.into());
        self
    }
}

/// Request to create a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePortfolioRequest {
    /// The portfolio name.
    pub name: String,
}

impl CreatePortfolioRequest {
    /// Create a new create portfolio request.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Request to edit a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct EditPortfolioRequest {
    /// The new portfolio name.
    pub name: String,
}

impl EditPortfolioRequest {
    /// Create a new edit portfolio request.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Funds to move between portfolios.
#[derive(Debug, Clone, Serialize)]
pub struct MoveFunds {
    /// The amount value.
    pub value: String,
    /// The currency.
    pub currency: String,
}

impl MoveFunds {
    /// Create new funds.
    pub fn new(value: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            currency: currency.into(),
        }
    }
}

/// Request to move funds between portfolios.
#[derive(Debug, Clone, Serialize)]
pub struct MoveFundsRequest {
    /// The funds to move.
    pub funds: MoveFunds,
    /// Source portfolio UUID.
    pub source_portfolio_uuid: String,
    /// Target portfolio UUID.
    pub target_portfolio_uuid: String,
}

impl MoveFundsRequest {
    /// Create a new move funds request.
    pub fn new(
        funds: MoveFunds,
        source_portfolio_uuid: impl Into<String>,
        target_portfolio_uuid: impl Into<String>,
    ) -> Self {
        Self {
            funds,
            source_portfolio_uuid: source_portfolio_uuid.into(),
            target_portfolio_uuid: target_portfolio_uuid.into(),
        }
    }
}

/// Response from moving funds.
#[derive(Debug, Clone, Deserialize)]
pub struct MoveFundsResponse {
    /// Source portfolio UUID.
    pub source_portfolio_uuid: String,
    /// Target portfolio UUID.
    pub target_portfolio_uuid: String,
}
