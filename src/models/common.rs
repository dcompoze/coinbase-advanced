//! Common types shared across API modules.

use serde::{Deserialize, Serialize};

/// A monetary amount with value and currency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// The numeric value as a string.
    pub value: String,
    /// The currency code (e.g., "USD", "BTC").
    pub currency: String,
}

/// Pagination parameters for list requests.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PaginationParams {
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor for the next page of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Pagination information in responses.
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    /// Cursor for the next page (None if no more pages).
    pub cursor: Option<String>,
    /// Whether there are more results.
    pub has_next: bool,
}

impl PaginationParams {
    /// Create new pagination parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit (max results per page).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the cursor for pagination.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}
