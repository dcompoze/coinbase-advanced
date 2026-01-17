//! Account-related types.

use serde::{Deserialize, Serialize};

use super::Balance;

/// A Coinbase trading account.
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    /// Unique identifier for the account.
    pub uuid: String,
    /// Display name of the account.
    pub name: String,
    /// Currency held in this account.
    pub currency: String,
    /// Available balance for trading.
    pub available_balance: Balance,
    /// Whether this is the default account for the currency.
    pub default: bool,
    /// Whether the account is active.
    pub active: bool,
    /// When the account was created.
    pub created_at: String,
    /// When the account was last updated.
    pub updated_at: String,
    /// When the account was deleted (if applicable).
    pub deleted_at: Option<String>,
    /// Account type (e.g., "ACCOUNT_TYPE_CRYPTO").
    #[serde(rename = "type")]
    pub account_type: String,
    /// Whether the account is ready for use.
    pub ready: bool,
    /// Amount on hold (in orders, etc.).
    pub hold: Balance,
    /// The retail portfolio this account belongs to.
    pub retail_portfolio_id: Option<String>,
}

/// Request parameters for listing accounts.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListAccountsParams {
    /// Maximum number of accounts to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Filter by retail portfolio ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retail_portfolio_id: Option<String>,
}

impl ListAccountsParams {
    /// Create new list accounts parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Filter by portfolio ID.
    pub fn portfolio(mut self, portfolio_id: impl Into<String>) -> Self {
        self.retail_portfolio_id = Some(portfolio_id.into());
        self
    }
}

/// Response from listing accounts.
#[derive(Debug, Clone, Deserialize)]
pub struct ListAccountsResponse {
    /// The list of accounts.
    pub accounts: Vec<Account>,
    /// Whether there are more accounts.
    pub has_next: bool,
    /// Cursor for the next page.
    pub cursor: Option<String>,
    /// Total size (if available).
    pub size: Option<u32>,
}

/// Response from getting a single account.
#[derive(Debug, Clone, Deserialize)]
pub struct GetAccountResponse {
    /// The account details.
    pub account: Account,
}
