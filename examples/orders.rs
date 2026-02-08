//! Order management example.
//!
//! Run with: cargo run --example orders
//!
//! WARNING: This example places real orders! Use sandbox mode for testing.
//!
//! Requires environment variables:
//! - COINBASE_API_KEY
//! - COINBASE_PRIVATE_KEY

use coinbase_advanced::models::ListFillsParams;
use coinbase_advanced::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    tracing_subscriber::fmt::init();

    let credentials = Credentials::from_env()?;

    // Use sandbox mode for testing to avoid real trades
    let client = RestClient::builder()
        .credentials(credentials)
        .sandbox(true) // IMPORTANT: Use sandbox for testing
        .build()?;

    println!("Connected to Coinbase (sandbox mode)");

    // List existing orders
    println!("\n--- Open Orders ---");
    let response = client.orders().list_all().await?;
    if response.orders.is_empty() {
        println!("No open orders");
    } else {
        for order in &response.orders {
            println!(
                "{}: {} {} @ {:?} ({:?})",
                order.order_id,
                order.side,
                order.product_id,
                order.created_time.as_deref().unwrap_or("unknown"),
                order.status
            );
        }
    }

    // List recent fills
    println!("\n--- Recent Fills ---");
    let fills_response = client.orders().list_fills(ListFillsParams::default()).await?;
    for fill in fills_response.fills.iter().take(5) {
        println!(
            "{}: {} {} @ {} (fee: {})",
            fill.trade_id,
            fill.product_id,
            fill.size,
            fill.price,
            fill.commission
        );
    }

    // Example: Place a limit order (commented out for safety)
    // Uncomment to actually place an order in sandbox
    /*
    println!("\n--- Placing Limit Order ---");
    let order = client.limit_order_gtc()
        .buy("BTC-USD")
        .base_size("0.0001")
        .limit_price("10000.00") // Low price, unlikely to fill
        .post_only(true)
        .send()
        .await?;

    println!("Order placed: {}", order.order_id);
    println!("Success: {}", order.success);

    // Cancel the order
    println!("\n--- Cancelling Order ---");
    let cancelled = client.orders().cancel(&[&order.order_id]).await?;
    for result in &cancelled.results {
        println!("{}: success={}", result.order_id, result.success);
    }
    */

    // Example: Market order builder
    println!("\n--- Order Builders Available ---");
    println!("client.market_order()     - Market IOC order");
    println!("client.limit_order_gtc()  - Limit Good-Til-Cancelled");
    println!("client.limit_order_gtd()  - Limit Good-Til-Date");
    println!("client.stop_limit_order_gtc() - Stop-Limit GTC");

    println!("\nExample usage:");
    println!("  client.market_order()");
    println!("      .buy(\"BTC-USD\")");
    println!("      .quote_size(\"100.00\")  // Buy $100 worth");
    println!("      .send().await?;");

    println!("\nDone!");
    Ok(())
}
