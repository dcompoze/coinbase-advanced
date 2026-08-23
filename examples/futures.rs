//! Futures (CFM) example.
//!
//! Run with: cargo run --example futures
//!
//! The account must be enabled for futures trading, otherwise the API
//! returns errors. This example prints errors instead of failing.
//!
//! Requires environment variables:
//! - COINBASE_API_KEY
//! - COINBASE_PRIVATE_KEY

use coinbase_advanced::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    tracing_subscriber::fmt::init();

    let client = RestClient::builder()
        .credentials(Credentials::from_env()?)
        .build()?;

    println!("--- Futures Balance Summary ---");
    match client.futures().get_balance_summary().await {
        Ok(summary) => {
            println!("Buying power: {:?}", summary.futures_buying_power);
            println!("Total USD balance: {:?}", summary.total_usd_balance);
        }
        Err(e) => println!("Not available: {}", e),
    }

    println!("\n--- Futures Positions ---");
    match client.futures().list_positions().await {
        Ok(positions) if positions.is_empty() => println!("No open positions"),
        Ok(positions) => {
            for position in &positions {
                println!(
                    "{}: side={:?}, expiration={:?}",
                    position.product_id, position.side, position.expiration_time
                );
            }
        }
        Err(e) => println!("Not available: {}", e),
    }

    println!("\n--- Futures Sweeps ---");
    match client.futures().list_sweeps().await {
        Ok(sweeps) if sweeps.is_empty() => println!("No scheduled sweeps"),
        Ok(sweeps) => {
            for sweep in &sweeps {
                println!("{:?}", sweep);
            }
        }
        Err(e) => println!("Not available: {}", e),
    }

    println!("\n--- Intraday Margin Setting ---");
    match client.futures().get_intraday_margin_setting().await {
        Ok(setting) => println!("{:?}", setting),
        Err(e) => println!("Not available: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
