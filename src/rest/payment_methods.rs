//! Payment Methods API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{GetPaymentMethodResponse, ListPaymentMethodsResponse, PaymentMethod};

/// API for managing payment methods.
///
/// This API provides endpoints for listing and retrieving payment methods.
pub struct PaymentMethodsApi<'a> {
    client: &'a RestClient,
}

impl<'a> PaymentMethodsApi<'a> {
    /// Create a new Payment Methods API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// List all payment methods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let payment_methods = client.payment_methods().list().await?;
    /// for pm in payment_methods {
    ///     println!("{}: {} ({})", pm.id, pm.name, pm.payment_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<PaymentMethod>> {
        let response: ListPaymentMethodsResponse = self.client.get("/payment_methods").await?;
        Ok(response.payment_methods)
    }

    /// Get a specific payment method by ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let payment_method = client.payment_methods().get("payment-method-id").await?;
    /// println!("Payment method: {} - {}", payment_method.name, payment_method.payment_type);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, payment_method_id: &str) -> Result<PaymentMethod> {
        let endpoint = format!("/payment_methods/{}", payment_method_id);
        let response: GetPaymentMethodResponse = self.client.get(&endpoint).await?;
        Ok(response.payment_method)
    }
}
