//! Fee-related types.

use serde::{Deserialize, Serialize};

/// Fee tier for the user, determined by notional (USD) volume.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeTier {
    /// Current fee tier for the user.
    pub pricing_tier: String,
    /// Lower bound (inclusive) of pricing tier in notional volume.
    pub usd_from: String,
    /// Upper bound (exclusive) of pricing tier in notional volume.
    pub usd_to: String,
    /// Taker fee rate, applied if the order takes liquidity.
    pub taker_fee_rate: String,
    /// Maker fee rate, applied if the order creates liquidity.
    pub maker_fee_rate: String,
    /// AOP (Advanced Order Placement) lower bound.
    #[serde(default)]
    pub aop_from: Option<String>,
    /// AOP upper bound.
    #[serde(default)]
    pub aop_to: Option<String>,
}

/// Margin rate information.
#[derive(Debug, Clone, Deserialize)]
pub struct MarginRate {
    /// The margin rate value.
    pub value: String,
}

/// Goods and Services Tax information.
#[derive(Debug, Clone, Deserialize)]
pub struct GoodsAndServicesTax {
    /// The GST rate.
    pub rate: String,
    /// The GST type (e.g., "INCLUSIVE", "EXCLUSIVE").
    #[serde(rename = "type")]
    pub gst_type: String,
}

/// Transaction summary containing fee information.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionSummary {
    /// Total volume across assets, denoted in USD.
    pub total_volume: f64,
    /// Total fees across assets, denoted in USD.
    pub total_fees: f64,
    /// Fee tier information.
    pub fee_tier: FeeTier,
    /// Margin rate (if applicable).
    pub margin_rate: Option<MarginRate>,
    /// Goods and Services Tax (if applicable).
    pub goods_and_services_tax: Option<GoodsAndServicesTax>,
    /// Advanced Trade volume (non-inclusive of Pro) across assets, denoted in USD.
    pub advanced_trade_only_volume: f64,
    /// Advanced Trade fees (non-inclusive of Pro) across assets, denoted in USD.
    pub advanced_trade_only_fees: f64,
    /// Coinbase Pro volume across assets, denoted in USD.
    pub coinbase_pro_volume: f64,
    /// Coinbase Pro fees across assets, denoted in USD.
    pub coinbase_pro_fees: f64,
    /// Total balance (optional).
    #[serde(default)]
    pub total_balance: Option<String>,
}

/// Parameters for getting transaction summary.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TransactionSummaryParams {
    /// Product type filter (e.g., "SPOT", "FUTURE").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    /// Contract expiry type filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_expiry_type: Option<String>,
    /// Product venue filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_venue: Option<String>,
}

impl TransactionSummaryParams {
    /// Create new transaction summary parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the product type filter.
    pub fn product_type(mut self, product_type: impl Into<String>) -> Self {
        self.product_type = Some(product_type.into());
        self
    }

    /// Set the contract expiry type filter.
    pub fn contract_expiry_type(mut self, contract_expiry_type: impl Into<String>) -> Self {
        self.contract_expiry_type = Some(contract_expiry_type.into());
        self
    }

    /// Set the product venue filter.
    pub fn product_venue(mut self, product_venue: impl Into<String>) -> Self {
        self.product_venue = Some(product_venue.into());
        self
    }
}
