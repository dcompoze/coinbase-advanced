//! Example: Get market data
//!
//! This example demonstrates how to fetch market data including
//! products, prices, and order book data.

use coinbase_advanced::{
    models::{GetProductBookParams, ListProductsParams},
    Credentials, RestClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();

    // Create authenticated client
    let credentials = Credentials::from_env()?;
    let client = RestClient::builder().credentials(credentials).build()?;

    // Get server time (public endpoint)
    println!("=== Server Time ===");
    let time = client.public().get_time().await?;
    println!("Server time: {}", time.iso);
    println!();

    // List products using public endpoint (doesn't require special permissions)
    println!("=== Top Products (Public API) ===");
    let products = client
        .public()
        .list_products(ListProductsParams::new().limit(5))
        .await?;

    for product in &products.products {
        println!(
            "{:10} | Price: {:>12} | 24h Change: {:>8}%",
            product.product_id, product.price, product.price_percentage_change_24h
        );
    }
    println!();

    // Get BTC-USD details (authenticated)
    println!("=== BTC-USD Details ===");
    let btc = client.products().get("BTC-USD").await?;
    println!("Product: {}", btc.product_id);
    println!("Price: ${}", btc.price);
    println!("24h Volume: {}", btc.volume_24h);
    println!("24h Change: {}%", btc.price_percentage_change_24h);
    println!("Min Order Size: {} BTC", btc.base_min_size);
    println!();

    // Get order book (using public API)
    println!("=== BTC-USD Order Book (Top 5) ===");
    let book = client
        .public()
        .get_product_book(GetProductBookParams::new("BTC-USD").limit(5))
        .await?;

    println!("Bids:");
    for bid in &book.bids {
        println!("  ${} - {} BTC", bid.price, bid.size);
    }
    println!("Asks:");
    for ask in &book.asks {
        println!("  ${} - {} BTC", ask.price, ask.size);
    }

    Ok(())
}
