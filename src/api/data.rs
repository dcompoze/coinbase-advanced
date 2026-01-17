//! Data API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::ApiKeyPermissions;

/// API for retrieving data about the current API key.
///
/// This API provides endpoints for querying API key permissions and capabilities.
pub struct DataApi<'a> {
    client: &'a RestClient,
}

impl<'a> DataApi<'a> {
    /// Create a new Data API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// Get the permissions for the current API key.
    ///
    /// This returns information about what actions the API key is authorized to perform.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let permissions = client.data().get_key_permissions().await?;
    /// println!("Can view: {}", permissions.can_view);
    /// println!("Can trade: {}", permissions.can_trade);
    /// println!("Can transfer: {}", permissions.can_transfer);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_key_permissions(&self) -> Result<ApiKeyPermissions> {
        self.client.get("/key_permissions").await
    }
}
