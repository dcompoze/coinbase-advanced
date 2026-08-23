//! Example: Get public market data (no auth required)

use coinbase_advanced::{RestClient, models::ListProductsParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RestClient::builder().build()?;

    println!("=== Server Time ===");
    let time = client.public().get_time().await?;
    println!("Server time: {}", time.iso);
    println!();

    println!("=== Public Products ===");
    let products = client
        .public()
        .list_products(ListProductsParams::new().limit(5))
        .await?;

    for product in &products.products {
        println!("{:10} | Price: {:>12}", product.product_id, product.price);
    }

    Ok(())
}
