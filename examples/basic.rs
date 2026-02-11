//! Basic example demonstrating account and product queries.
//!
//! Run with: cargo run --example basic
//!
//! Requires environment variables:
//! - COINBASE_API_KEY
//! - COINBASE_PRIVATE_KEY

use coinbase_advanced::models::GetBestBidAskParams;
use coinbase_advanced::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt::init();

    // Load credentials from environment
    let credentials = Credentials::from_env()?;
    println!("Loaded credentials for: {}", credentials.api_key());

    // Build the client
    let client = RestClient::builder().credentials(credentials).build()?;

    // Get server time (public endpoint)
    println!("\n--- Server Time ---");
    let time = client.public().get_time().await?;
    println!("ISO: {}", time.iso);
    println!("Epoch: {} seconds", time.epoch_seconds);

    // List accounts
    println!("\n--- Accounts ---");
    let response = client.accounts().list_all().await?;
    for account in response.accounts.iter().take(5) {
        println!(
            "{}: {} {} (available: {} {})",
            account.name,
            account.available_balance.value,
            account.currency,
            account.available_balance.value,
            account.available_balance.currency
        );
    }
    if response.accounts.len() > 5 {
        println!("... and {} more accounts", response.accounts.len() - 5);
    }

    // Get products
    println!("\n--- Products (first 5) ---");
    let response = client.products().list_all().await?;
    for product in response.products.iter().take(5) {
        println!(
            "{}: {} @ ${}",
            product.product_id, product.base_name, product.price
        );
    }
    println!("Total products: {}", response.products.len());

    // Get a specific product
    println!("\n--- BTC-USD Details ---");
    let btc = client.products().get("BTC-USD").await?;
    println!("Product: {} / {}", btc.base_name, btc.quote_name);
    println!("Price: ${}", btc.price);
    println!("24h Volume: {}", btc.volume_24h);
    println!("24h Change: {}%", btc.price_percentage_change_24h);

    // Get best bid/ask
    println!("\n--- Best Bid/Ask ---");
    let params = GetBestBidAskParams::new().product_ids(&["BTC-USD", "ETH-USD"]);
    let bid_ask = client.products().get_best_bid_ask(params).await?;
    for pricebook in &bid_ask.pricebooks {
        println!(
            "{}: bid={:?}, ask={:?}",
            pricebook.product_id,
            pricebook.bids.first().map(|b| &b.price),
            pricebook.asks.first().map(|a| &a.price)
        );
    }

    // Get fee tier
    println!("\n--- Fee Tier ---");
    let fees = client.fees().get_transaction_summary().await?;
    println!("Pricing tier: {}", fees.fee_tier.pricing_tier);
    println!("Maker fee: {}%", fees.fee_tier.maker_fee_rate);
    println!("Taker fee: {}%", fees.fee_tier.taker_fee_rate);

    println!("\nDone!");
    Ok(())
}
