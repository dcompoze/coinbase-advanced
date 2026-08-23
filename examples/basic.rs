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
    tracing_subscriber::fmt::init();

    let credentials = Credentials::from_env()?;
    println!("Loaded credentials for: {}", credentials.api_key());

    let client = RestClient::builder().credentials(credentials).build()?;

    println!("\n--- Server Time ---");
    let time = client.public().get_time().await?;
    println!("ISO: {}", time.iso);
    println!("Epoch: {} seconds", time.epoch_seconds);

    println!("\n--- Accounts ---");
    let accounts = client.accounts().list_all().await?;
    for account in accounts.iter().take(5) {
        println!(
            "{}: {} {} (available: {} {})",
            account.name,
            account.available_balance.value,
            account.currency,
            account.available_balance.value,
            account.available_balance.currency
        );
    }
    if accounts.len() > 5 {
        println!("... and {} more accounts", accounts.len() - 5);
    }

    println!("\n--- Products (first 5) ---");
    let response = client.products().list_all().await?;
    for product in response.products.iter().take(5) {
        println!(
            "{}: {} @ ${}",
            product.product_id, product.base_name, product.price
        );
    }
    println!("Total products: {}", response.products.len());

    println!("\n--- BTC-USD Details ---");
    let btc = client.products().get("BTC-USD").await?;
    println!("Product: {} / {}", btc.base_name, btc.quote_name);
    println!("Price: ${}", btc.price);
    println!("24h Volume: {}", btc.volume_24h);
    println!("24h Change: {}%", btc.price_percentage_change_24h);

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

    println!("\n--- Fee Tier ---");
    let fees = client.fees().get_transaction_summary().await?;
    println!("Pricing tier: {}", fees.fee_tier.pricing_tier);
    println!("Maker fee: {}%", fees.fee_tier.maker_fee_rate);
    println!("Taker fee: {}%", fees.fee_tier.taker_fee_rate);

    println!("\nDone!");
    Ok(())
}
