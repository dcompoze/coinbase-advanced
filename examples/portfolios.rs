//! Portfolio management example.
//!
//! Run with: cargo run --example portfolios
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

    println!("--- Portfolios ---");
    let portfolios = client.portfolios().list().await?;
    for portfolio in &portfolios {
        println!(
            "{}: {} ({:?}{})",
            portfolio.uuid,
            portfolio.name,
            portfolio.portfolio_type,
            if portfolio.deleted { ", deleted" } else { "" }
        );
    }

    if let Some(portfolio) = portfolios.first() {
        println!("\n--- Breakdown of {} ---", portfolio.name);
        let breakdown = client.portfolios().get_breakdown(&portfolio.uuid).await?;

        if let Some(balances) = &breakdown.portfolio_balances {
            println!("Balances: {:?}", balances);
        }
        println!("Spot positions: {}", breakdown.spot_positions.len());
        for position in breakdown.spot_positions.iter().take(5) {
            println!("  {:?}", position);
        }
    }

    // Example: Create, edit, and delete a portfolio (commented out for safety)
    /*
    use coinbase_advanced::models::{CreatePortfolioRequest, EditPortfolioRequest};

    let created = client.portfolios()
        .create(CreatePortfolioRequest::new("My Portfolio"))
        .await?;
    println!("Created portfolio: {}", created.uuid);

    client.portfolios()
        .edit(&created.uuid, EditPortfolioRequest::new("Renamed Portfolio"))
        .await?;

    client.portfolios().delete(&created.uuid).await?;
    */

    println!("\nDone!");
    Ok(())
}
