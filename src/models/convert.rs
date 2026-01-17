//! Convert API types.

use serde::{Deserialize, Serialize};

/// Trade status for a conversion.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ConvertTradeStatus {
    /// Unspecified status.
    #[serde(rename = "TRADE_STATUS_UNSPECIFIED")]
    Unspecified,
    /// Trade has been created.
    #[serde(rename = "TRADE_STATUS_CREATED")]
    Created,
    /// Trade has started.
    #[serde(rename = "TRADE_STATUS_STARTED")]
    Started,
    /// Trade has been completed.
    #[serde(rename = "TRADE_STATUS_COMPLETED")]
    Completed,
    /// Trade has been canceled.
    #[serde(rename = "TRADE_STATUS_CANCELED")]
    Canceled,
    /// Unknown status.
    #[serde(other)]
    Unknown,
}

/// Amount with currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertAmount {
    /// The amount value.
    pub value: String,
    /// The currency.
    pub currency: String,
}

/// Fee information for a conversion.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertFee {
    /// Fee title.
    #[serde(default)]
    pub title: Option<String>,
    /// Fee description.
    #[serde(default)]
    pub description: Option<String>,
    /// Fee amount.
    pub amount: ConvertAmount,
    /// Fee label.
    #[serde(default)]
    pub label: Option<String>,
}

/// Account details for conversion source/target.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertAccountDetail {
    /// Account type.
    #[serde(rename = "type", default)]
    pub account_type: Option<String>,
    /// Network.
    #[serde(default)]
    pub network: Option<String>,
    /// Ledger account details.
    #[serde(default)]
    pub ledger_account: Option<serde_json::Value>,
}

/// A conversion trade.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertTrade {
    /// The trade ID.
    pub id: String,
    /// Trade status.
    pub status: ConvertTradeStatus,
    /// User entered amount.
    #[serde(default)]
    pub user_entered_amount: Option<ConvertAmount>,
    /// Converted amount.
    #[serde(default)]
    pub amount: Option<ConvertAmount>,
    /// Subtotal.
    #[serde(default)]
    pub subtotal: Option<ConvertAmount>,
    /// Total.
    #[serde(default)]
    pub total: Option<ConvertAmount>,
    /// Fees.
    #[serde(default)]
    pub fees: Vec<ConvertFee>,
    /// Total fee.
    #[serde(default)]
    pub total_fee: Option<ConvertFee>,
    /// Source account details.
    #[serde(default)]
    pub source: Option<ConvertAccountDetail>,
    /// Target account details.
    #[serde(default)]
    pub target: Option<ConvertAccountDetail>,
    /// Source currency.
    #[serde(default)]
    pub source_currency: Option<String>,
    /// Target currency.
    #[serde(default)]
    pub target_currency: Option<String>,
    /// Source account ID.
    #[serde(default)]
    pub source_id: Option<String>,
    /// Target account ID.
    #[serde(default)]
    pub target_id: Option<String>,
    /// Exchange rate.
    #[serde(default)]
    pub exchange_rate: Option<ConvertAmount>,
    /// User reference.
    #[serde(default)]
    pub user_reference: Option<String>,
}

/// Response containing a trade.
#[derive(Debug, Clone, Deserialize)]
pub struct ConvertTradeResponse {
    /// The trade.
    pub trade: ConvertTrade,
}

/// Trade incentive metadata for waiving fees.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TradeIncentiveMetadata {
    /// User incentive ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_incentive_id: Option<String>,
    /// Promo code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_val: Option<String>,
}

/// Request to create a convert quote.
#[derive(Debug, Clone, Serialize)]
pub struct CreateConvertQuoteRequest {
    /// Source account ID (the account to convert from).
    pub from_account: String,
    /// Target account ID (the account to convert to).
    pub to_account: String,
    /// Amount to convert.
    pub amount: String,
    /// Trade incentive metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_incentive_metadata: Option<TradeIncentiveMetadata>,
}

impl CreateConvertQuoteRequest {
    /// Create a new convert quote request.
    pub fn new(
        from_account: impl Into<String>,
        to_account: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        Self {
            from_account: from_account.into(),
            to_account: to_account.into(),
            amount: amount.into(),
            trade_incentive_metadata: None,
        }
    }

    /// Set the trade incentive metadata.
    pub fn with_incentive(mut self, metadata: TradeIncentiveMetadata) -> Self {
        self.trade_incentive_metadata = Some(metadata);
        self
    }
}

/// Request to commit a convert trade.
#[derive(Debug, Clone, Serialize)]
pub struct CommitConvertTradeRequest {
    /// Source account ID.
    pub from_account: String,
    /// Target account ID.
    pub to_account: String,
}

impl CommitConvertTradeRequest {
    /// Create a new commit convert trade request.
    pub fn new(from_account: impl Into<String>, to_account: impl Into<String>) -> Self {
        Self {
            from_account: from_account.into(),
            to_account: to_account.into(),
        }
    }
}

/// Parameters for getting a convert trade.
#[derive(Debug, Clone, Serialize)]
pub struct GetConvertTradeParams {
    /// Source account ID.
    pub from_account: String,
    /// Target account ID.
    pub to_account: String,
}

impl GetConvertTradeParams {
    /// Create new get convert trade parameters.
    pub fn new(from_account: impl Into<String>, to_account: impl Into<String>) -> Self {
        Self {
            from_account: from_account.into(),
            to_account: to_account.into(),
        }
    }
}
