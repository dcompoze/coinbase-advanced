//! Data API types.

use serde::Deserialize;

/// API key permissions.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyPermissions {
    /// Whether the API key can view data.
    pub can_view: bool,
    /// Whether the API key can trade.
    pub can_trade: bool,
    /// Whether the API key can transfer funds.
    pub can_transfer: bool,
    /// The portfolio UUID associated with this key.
    pub portfolio_uuid: String,
    /// The portfolio type.
    pub portfolio_type: String,
}
