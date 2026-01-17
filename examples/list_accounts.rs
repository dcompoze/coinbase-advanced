//! Example: List all accounts
//!
//! This example demonstrates how to authenticate with the Coinbase API
//! and list all trading accounts.
//!
//! Set the following environment variables:
//! - COINBASE_API_KEY
//! - COINBASE_PRIVATE_KEY

use coinbase_client::{Credentials, RestClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Create credentials from environment
    let credentials = Credentials::from_env()?;
    println!("Loaded credentials for: {}", credentials.api_key());

    // Build the client
    let client = RestClient::builder()
        .credentials(credentials)
        .build()?;

    println!("Fetching accounts...\n");

    // List all accounts
    let response = client.accounts().list_all().await?;

    println!("Found {} accounts:", response.accounts.len());
    println!("{:-<60}", "");

    for account in &response.accounts {
        if account.available_balance.value != "0" || account.hold.value != "0" {
            println!(
                "{:8} | Available: {:>15} | Hold: {:>15}",
                account.currency, account.available_balance.value, account.hold.value
            );
        }
    }

    println!("{:-<60}", "");
    println!("Has more pages: {}", response.has_next);

    Ok(())
}
