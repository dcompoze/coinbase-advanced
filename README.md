# coinbase-client

A Rust client library for the [Coinbase Advanced Trade API](https://docs.cdp.coinbase.com/advanced-trade-api/docs/welcome).

[![Crates.io](https://img.shields.io/crates/v/coinbase-client.svg)](https://crates.io/crates/coinbase-client)
[![Documentation](https://docs.rs/coinbase-client/badge.svg)](https://docs.rs/coinbase-client)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/example/coinbase-client/actions/workflows/ci.yml/badge.svg)](https://github.com/example/coinbase-client/actions/workflows/ci.yml)

## Features

- Complete REST API coverage for Coinbase Advanced Trade
- WebSocket support for real-time market data and order updates
- Strongly typed request/response models
- JWT (ES256) authentication
- Optional rate limiting
- Async/await with Tokio
- Sandbox environment support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
coinbase-client = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Authentication

First, obtain API credentials from the [Coinbase Developer Platform](https://portal.cdp.coinbase.com/). You'll need:

- An API key (in the format `organizations/{org_id}/apiKeys/{key_id}`)
- An EC private key in PEM format

```rust
use coinbase_client::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_client::Result<()> {
    // Load credentials from environment variables
    // COINBASE_API_KEY and COINBASE_PRIVATE_KEY
    let credentials = Credentials::from_env()?;

    // Or create credentials directly
    let credentials = Credentials::new(
        "organizations/xxx/apiKeys/yyy",
        "-----BEGIN EC PRIVATE KEY-----\n...\n-----END EC PRIVATE KEY-----"
    )?;

    let client = RestClient::builder()
        .credentials(credentials)
        .build()?;

    Ok(())
}
```

### Get Account Balances

```rust
use coinbase_client::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_client::Result<()> {
    let client = RestClient::builder()
        .credentials(Credentials::from_env()?)
        .build()?;

    // List all accounts
    let accounts = client.accounts().list_all().await?;
    for account in accounts {
        println!("{}: {} {}",
            account.name,
            account.available_balance.value,
            account.currency
        );
    }

    Ok(())
}
```

### Get Market Data

```rust
use coinbase_client::RestClient;

#[tokio::main]
async fn main() -> coinbase_client::Result<()> {
    // Public endpoints don't require authentication
    let client = RestClient::builder().build()?;

    // Get server time
    let time = client.public().get_time().await?;
    println!("Server time: {}", time.iso);

    Ok(())
}
```

### Place an Order

```rust
use coinbase_client::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_client::Result<()> {
    let client = RestClient::builder()
        .credentials(Credentials::from_env()?)
        .build()?;

    // Place a market order to buy $100 of BTC
    let order = client.market_order()
        .buy("BTC-USD")
        .quote_size("100.00")
        .send()
        .await?;

    println!("Order placed: {}", order.order_id);

    // Place a limit order
    let order = client.limit_order_gtc()
        .buy("BTC-USD")
        .base_size("0.001")
        .limit_price("50000.00")
        .send()
        .await?;

    println!("Limit order: {}", order.order_id);

    Ok(())
}
```

### WebSocket Streaming

```rust
use coinbase_client::websocket::{WebSocketClient, Channel};
use futures::StreamExt;

#[tokio::main]
async fn main() -> coinbase_client::Result<()> {
    let client = WebSocketClient::builder().build()?;

    // Connect to WebSocket
    let mut stream = client.connect().await?;

    // Subscribe to ticker updates
    client.subscribe(&[
        Channel::Ticker {
            product_ids: vec!["BTC-USD".to_string(), "ETH-USD".to_string()],
        },
    ]).await?;

    // Process messages
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(message) => println!("Received: {:?}", message),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

## API Coverage

### REST APIs

| API | Status |
|-----|--------|
| Accounts | Complete |
| Products | Complete |
| Orders | Complete |
| Fees | Complete |
| Portfolios | Complete |
| Convert | Complete |
| Data | Complete |
| Payment Methods | Complete |
| Perpetuals (INTX) | Complete |
| Futures (CFM) | Complete |
| Public | Complete |

### WebSocket Channels

| Channel | Status |
|---------|--------|
| Heartbeats | Complete |
| Status | Complete |
| Ticker | Complete |
| Ticker Batch | Complete |
| Level2 | Complete |
| Candles | Complete |
| Market Trades | Complete |
| User | Complete |

## Configuration

### Sandbox Mode

Use the sandbox environment for testing:

```rust
let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .sandbox(true)
    .build()?;
```

### Rate Limiting

Enable client-side rate limiting to avoid hitting API limits:

```rust
let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .rate_limiting(true)
    .build()?;
```

### Custom Timeout

```rust
use std::time::Duration;

let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .timeout(Duration::from_secs(60))
    .build()?;
```

## Error Handling

The library uses a custom `Error` type that covers various failure modes:

```rust
use coinbase_client::{Error, RestClient, Credentials};

async fn example() {
    let client = RestClient::builder()
        .credentials(Credentials::from_env().unwrap())
        .build()
        .unwrap();

    match client.accounts().list_all().await {
        Ok(accounts) => println!("Found {} accounts", accounts.len()),
        Err(Error::RateLimited { retry_after }) => {
            println!("Rate limited, retry after {:?}", retry_after);
        }
        Err(Error::Api { message, status, .. }) => {
            println!("API error {}: {}", status, message);
        }
        Err(e) => println!("Other error: {}", e),
    }
}
```

## Examples

See the [examples](examples/) directory for more detailed examples:

- `basic.rs` - Basic account and product queries
- `orders.rs` - Order placement and management
- `websocket.rs` - Real-time WebSocket streaming

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
