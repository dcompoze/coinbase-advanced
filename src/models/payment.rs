//! Payment method types.

use serde::Deserialize;

/// A payment method available to the user.
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentMethod {
    /// Unique identifier for the payment method.
    pub id: String,
    /// The payment method type.
    #[serde(rename = "type")]
    pub payment_type: String,
    /// Name for the payment method.
    pub name: String,
    /// Currency symbol for the payment method.
    pub currency: String,
    /// The verified status of the payment method.
    #[serde(default)]
    pub verified: bool,
    /// Whether this payment method can perform buys.
    #[serde(default)]
    pub allow_buy: bool,
    /// Whether this payment method can perform sells.
    #[serde(default)]
    pub allow_sell: bool,
    /// Whether this payment method can perform deposits.
    #[serde(default)]
    pub allow_deposit: bool,
    /// Whether this payment method can perform withdrawals.
    #[serde(default)]
    pub allow_withdraw: bool,
    /// Time at which this payment method was created.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Time at which this payment method was updated.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Response containing a list of payment methods.
#[derive(Debug, Clone, Deserialize)]
pub struct ListPaymentMethodsResponse {
    /// The payment methods.
    pub payment_methods: Vec<PaymentMethod>,
}

/// Response containing a single payment method.
#[derive(Debug, Clone, Deserialize)]
pub struct GetPaymentMethodResponse {
    /// The payment method.
    pub payment_method: PaymentMethod,
}
