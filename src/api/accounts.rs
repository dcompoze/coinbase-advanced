//! Accounts API endpoints.

use crate::client::RestClient;
use crate::error::Result;
use crate::models::{Account, GetAccountResponse, ListAccountsParams, ListAccountsResponse};

/// API for managing accounts.
///
/// Accounts represent wallets for holding different currencies.
/// Each account holds a single currency.
pub struct AccountsApi<'a> {
    client: &'a RestClient,
}

impl<'a> AccountsApi<'a> {
    /// Create a new Accounts API instance.
    pub(crate) fn new(client: &'a RestClient) -> Self {
        Self { client }
    }

    /// List all accounts.
    ///
    /// Returns a paginated list of accounts. Use the `cursor` from the response
    /// to fetch the next page.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_client::{RestClient, Credentials, models::ListAccountsParams};
    /// # async fn example() -> coinbase_client::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// // List first 10 accounts
    /// let response = client.accounts()
    ///     .list(ListAccountsParams::new().limit(10))
    ///     .await?;
    ///
    /// for account in response.accounts {
    ///     println!("{}: {} {}", account.name, account.available_balance.value, account.currency);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, params: ListAccountsParams) -> Result<ListAccountsResponse> {
        self.client.get_with_query("/accounts", &params).await
    }

    /// List all accounts with default parameters.
    pub async fn list_all(&self) -> Result<ListAccountsResponse> {
        self.list(ListAccountsParams::default()).await
    }

    /// Get a single account by UUID.
    ///
    /// # Arguments
    ///
    /// * `account_uuid` - The unique identifier of the account.
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
    /// let account = client.accounts()
    ///     .get("account-uuid-here")
    ///     .await?;
    ///
    /// println!("Balance: {} {}", account.available_balance.value, account.currency);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, account_uuid: &str) -> Result<Account> {
        let endpoint = format!("/accounts/{}", account_uuid);
        let response: GetAccountResponse = self.client.get(&endpoint).await?;
        Ok(response.account)
    }
}
