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
