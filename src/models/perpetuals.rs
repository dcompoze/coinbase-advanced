//! Perpetuals/INTX API types.

use serde::{Deserialize, Serialize};

/// Amount with value and currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntxAmount {
    /// The amount value.
    #[serde(default)]
    pub value: Option<String>,
    /// The currency.
    #[serde(default)]
    pub currency: Option<String>,
}

/// INTX (perpetuals) position.
#[derive(Debug, Clone, Deserialize)]
pub struct IntxPosition {
    /// Product ID.
    pub product_id: String,
    /// Product UUID.
    #[serde(default)]
    pub product_uuid: Option<String>,
    /// Portfolio UUID.
    #[serde(default)]
    pub portfolio_uuid: Option<String>,
    /// Symbol (e.g., BTC-PERP).
    #[serde(default)]
    pub symbol: Option<String>,
    /// Volume-weighted average price.
    #[serde(default)]
    pub vwap: Option<IntxAmount>,
    /// Position side (LONG, SHORT).
    #[serde(default)]
    pub position_side: Option<String>,
    /// Net size.
    #[serde(default)]
    pub net_size: Option<String>,
    /// Buy order size.
    #[serde(default)]
    pub buy_order_size: Option<String>,
    /// Sell order size.
    #[serde(default)]
    pub sell_order_size: Option<String>,
    /// Initial margin contribution.
    #[serde(default)]
    pub im_contribution: Option<String>,
    /// Unrealized PnL.
    #[serde(default)]
    pub unrealized_pnl: Option<IntxAmount>,
    /// Mark price.
    #[serde(default)]
    pub mark_price: Option<IntxAmount>,
    /// Liquidation price.
    #[serde(default)]
    pub liquidation_price: Option<IntxAmount>,
    /// Leverage.
    #[serde(default)]
    pub leverage: Option<String>,
    /// IM notional.
    #[serde(default)]
    pub im_notional: Option<IntxAmount>,
    /// MM notional.
    #[serde(default)]
    pub mm_notional: Option<IntxAmount>,
    /// Position notional.
    #[serde(default)]
    pub position_notional: Option<String>,
}

/// INTX position summary.
#[derive(Debug, Clone, Deserialize)]
pub struct IntxSummary {
    /// Aggregated PnL.
    #[serde(default)]
    pub aggregated_pnl: Option<IntxAmount>,
}

/// Response for listing perpetuals positions.
#[derive(Debug, Clone, Deserialize)]
pub struct ListPerpetualsPositionsResponse {
    /// Positions.
    #[serde(default)]
    pub positions: Vec<IntxPosition>,
    /// Summary.
    #[serde(default)]
    pub summary: Option<IntxSummary>,
}

/// Response for getting a single perpetuals position.
#[derive(Debug, Clone, Deserialize)]
pub struct GetPerpetualsPositionResponse {
    /// The position.
    pub position: IntxPosition,
}

/// INTX portfolio balance.
#[derive(Debug, Clone, Deserialize)]
pub struct IntxPortfolioBalance {
    /// Asset.
    #[serde(default)]
    pub asset: Option<String>,
    /// Quantity.
    #[serde(default)]
    pub quantity: Option<String>,
    /// Hold.
    #[serde(default)]
    pub hold: Option<String>,
    /// Transfer hold.
    #[serde(default)]
    pub transfer_hold: Option<String>,
    /// Collateral value.
    #[serde(default)]
    pub collateral_value: Option<String>,
    /// Max withdraw amount.
    #[serde(default)]
    pub max_withdraw_amount: Option<String>,
}

/// Response for getting portfolio balances.
#[derive(Debug, Clone, Deserialize)]
pub struct GetPortfolioBalancesResponse {
    /// Portfolio balances.
    #[serde(default)]
    pub portfolio_balances: Vec<IntxPortfolioBalance>,
}

/// INTX portfolio summary.
#[derive(Debug, Clone, Deserialize)]
pub struct IntxPortfolioSummary {
    /// Unrealized PnL.
    #[serde(default)]
    pub unrealized_pnl: Option<IntxAmount>,
    /// Buying power.
    #[serde(default)]
    pub buying_power: Option<IntxAmount>,
    /// Total balance.
    #[serde(default)]
    pub total_balance: Option<IntxAmount>,
    /// Max withdrawal amount.
    #[serde(default)]
    pub max_withdrawal_amount: Option<IntxAmount>,
}

/// Response for getting portfolio summary.
#[derive(Debug, Clone, Deserialize)]
pub struct GetPerpetualsPortfolioSummaryResponse {
    /// Summary.
    pub summary: IntxPortfolioSummary,
}

/// Request to allocate funds to a portfolio.
#[derive(Debug, Clone, Serialize)]
pub struct AllocatePortfolioRequest {
    /// Portfolio UUID.
    pub portfolio_uuid: String,
    /// Symbol.
    pub symbol: String,
    /// Amount to allocate.
    pub amount: String,
    /// Currency.
    pub currency: String,
}

impl AllocatePortfolioRequest {
    /// Create a new allocate portfolio request.
    pub fn new(
        portfolio_uuid: impl Into<String>,
        symbol: impl Into<String>,
        amount: impl Into<String>,
        currency: impl Into<String>,
    ) -> Self {
        Self {
            portfolio_uuid: portfolio_uuid.into(),
            symbol: symbol.into(),
            amount: amount.into(),
            currency: currency.into(),
        }
    }
}

/// Request to set multi-asset collateral.
#[derive(Debug, Clone, Serialize)]
pub struct SetMultiAssetCollateralRequest {
    /// Whether multi-asset collateral is enabled.
    pub multi_asset_collateral_enabled: bool,
}

impl SetMultiAssetCollateralRequest {
    /// Create a new set multi-asset collateral request.
    pub fn new(enabled: bool) -> Self {
        Self {
            multi_asset_collateral_enabled: enabled,
        }
    }
}
