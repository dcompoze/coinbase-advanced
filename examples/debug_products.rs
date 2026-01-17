//! Debug: Test products endpoint

use coinbase_client::{models::ListProductsParams, Credentials, RestClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("coinbase_client=debug,reqwest=debug")
        .init();

    dotenv::dotenv().ok();

    let credentials = Credentials::from_env()?;
    let client = RestClient::builder().credentials(credentials).build()?;

    // Try the authenticated products endpoint
    println!("Trying authenticated products endpoint...");
    match client.products().list(ListProductsParams::new().limit(1)).await {
        Ok(products) => {
            println!("Success! Got {} products", products.products.len());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    // Try a specific product
    println!("\nTrying to get BTC-USD...");
    match client.products().get("BTC-USD").await {
        Ok(product) => {
            println!("Success! BTC price: {}", product.price);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    Ok(())
}
