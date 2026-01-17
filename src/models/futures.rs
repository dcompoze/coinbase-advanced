//! Futures/CFM API types.

use serde::{Deserialize, Serialize};

/// CFM futures position.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesPosition {
    /// Product ID.
    pub product_id: String,
    /// Expiration time.
    #[serde(default)]
    pub expiration_time: Option<String>,
    /// Position side (LONG, SHORT).
    #[serde(default)]
    pub side: Option<String>,
    /// Number of contracts.
    #[serde(default)]
    pub number_of_contracts: Option<String>,
    /// Current price.
    #[serde(default)]
    pub current_price: Option<String>,
    /// Average entry price.
    #[serde(default)]
    pub avg_entry_price: Option<String>,
    /// Unrealized PnL.
    #[serde(default)]
    pub unrealized_pnl: Option<String>,
    /// Daily realized PnL.
    #[serde(default)]
    pub daily_realized_pnl: Option<String>,
}

/// Response for listing futures positions.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFuturesPositionsResponse {
    /// Positions.
    #[serde(default)]
    pub positions: Vec<FuturesPosition>,
}

/// Response for getting a single futures position.
#[derive(Debug, Clone, Deserialize)]
pub struct GetFuturesPositionResponse {
    /// The position.
    pub position: FuturesPosition,
}

/// Futures balance summary.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesBalanceSummary {
    /// Futures buying power.
    #[serde(default)]
    pub futures_buying_power: Option<String>,
    /// Total USD balance.
    #[serde(default)]
    pub total_usd_balance: Option<String>,
    /// CFTC unrealized PnL.
    #[serde(default)]
    pub cbi_usd_balance: Option<String>,
    /// CFM USD balance.
    #[serde(default)]
    pub cfm_usd_balance: Option<String>,
    /// Total open orders hold amount.
    #[serde(default)]
    pub total_open_orders_hold_amount: Option<String>,
    /// Unrealized PnL.
    #[serde(default)]
    pub unrealized_pnl: Option<String>,
    /// Daily realized PnL.
    #[serde(default)]
    pub daily_realized_pnl: Option<String>,
    /// Initial margin.
    #[serde(default)]
    pub initial_margin: Option<String>,
    /// Available margin.
    #[serde(default)]
    pub available_margin: Option<String>,
    /// Liquidation threshold.
    #[serde(default)]
    pub liquidation_threshold: Option<String>,
    /// Liquidation buffer amount.
    #[serde(default)]
    pub liquidation_buffer_amount: Option<String>,
    /// Liquidation buffer percentage.
    #[serde(default)]
    pub liquidation_buffer_percentage: Option<String>,
}

/// Response for getting balance summary.
#[derive(Debug, Clone, Deserialize)]
pub struct GetFuturesBalanceSummaryResponse {
    /// Balance summary.
    pub balance_summary: FuturesBalanceSummary,
}

/// Intraday margin setting.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntradayMarginSetting {
    /// The margin setting value.
    #[serde(default)]
    pub setting: Option<String>,
}

/// Response for getting intraday margin setting.
#[derive(Debug, Clone, Deserialize)]
pub struct GetIntradayMarginSettingResponse {
    /// The setting.
    #[serde(default)]
    pub setting: Option<String>,
}

/// Current margin window.
#[derive(Debug, Clone, Deserialize)]
pub struct MarginWindow {
    /// Margin window type.
    #[serde(default)]
    pub margin_window_type: Option<String>,
    /// End time.
    #[serde(default)]
    pub end_time: Option<String>,
    /// Is intraday margin killswitch enabled.
    #[serde(default)]
    pub is_intraday_margin_killswitch_enabled: Option<bool>,
    /// Is intraday margin enabled.
    #[serde(default)]
    pub is_intraday_margin_enrollment_killswitch_enabled: Option<bool>,
}

/// Response for getting current margin window.
#[derive(Debug, Clone, Deserialize)]
pub struct GetCurrentMarginWindowResponse {
    /// The margin window.
    pub margin_window: MarginWindow,
}

/// Parameters for getting current margin window.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetCurrentMarginWindowParams {
    /// Margin profile type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_profile_type: Option<String>,
}

impl GetCurrentMarginWindowParams {
    /// Create new parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the margin profile type.
    pub fn margin_profile_type(mut self, margin_profile_type: impl Into<String>) -> Self {
        self.margin_profile_type = Some(margin_profile_type.into());
        self
    }
}

/// A futures sweep.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesSweep {
    /// Sweep ID.
    #[serde(default)]
    pub id: Option<String>,
    /// Requested amount.
    #[serde(default)]
    pub requested_amount: Option<String>,
    /// Should sweep all.
    #[serde(default)]
    pub should_sweep_all: Option<bool>,
    /// Status.
    #[serde(default)]
    pub status: Option<String>,
    /// Scheduled time.
    #[serde(default)]
    pub scheduled_time: Option<String>,
}

/// Response for listing futures sweeps.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFuturesSweepsResponse {
    /// Sweeps.
    #[serde(default)]
    pub sweeps: Vec<FuturesSweep>,
}

/// Request to schedule a futures sweep.
#[derive(Debug, Clone, Serialize)]
pub struct ScheduleFuturesSweepRequest {
    /// USD amount to sweep.
    pub usd_amount: String,
}

impl ScheduleFuturesSweepRequest {
    /// Create a new schedule futures sweep request.
    pub fn new(usd_amount: impl Into<String>) -> Self {
        Self {
            usd_amount: usd_amount.into(),
        }
    }
}

/// Response from scheduling a futures sweep.
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleFuturesSweepResponse {
    /// Success status.
    #[serde(default)]
    pub success: bool,
}

/// Request to set intraday margin setting.
#[derive(Debug, Clone, Serialize)]
pub struct SetIntradayMarginSettingRequest {
    /// The setting value.
    pub setting: String,
}

impl SetIntradayMarginSettingRequest {
    /// Create a new set intraday margin setting request.
    pub fn new(setting: impl Into<String>) -> Self {
        Self {
            setting: setting.into(),
        }
    }
}
