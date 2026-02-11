# coinbase-advanced

A Rust async client library for the [Coinbase Advanced Trade API](https://docs.cdp.coinbase.com/advanced-trade-api/docs/welcome).

- Complete REST API coverage for Coinbase Advanced Trade
- WebSocket support for real-time market data
- JWT (ES256) authentication
- Optional client-side rate limiting
- Async/await with Tokio/Reqwest

## Library

Authentication:

Obtain API credentials from the [Coinbase Developer Platform](https://portal.cdp.coinbase.com/).

```rust
use coinbase_advanced::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    let credentials = Credentials::from_env()?;

    let client = RestClient::builder()
        .credentials(credentials)
        .build()?;

    Ok(())
}
```

Get account balances:

```rust
use coinbase_advanced::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    let client = RestClient::builder()
        .credentials(Credentials::from_env()?)
        .build()?;

    let accounts = client.accounts().list_all().await?;
    for account in accounts {
        println!("{}: {} {}", account.name, account.available_balance.value, account.currency);
    }

    Ok(())
}
```

Place an order:

```rust
use coinbase_advanced::{Credentials, RestClient};

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    let client = RestClient::builder()
        .credentials(Credentials::from_env()?)
        .build()?;

    let order = client.market_order()
        .buy("BTC-USD")
        .quote_size("100.00")
        .send()
        .await?;

    println!("Order placed: {}", order.order_id);
    Ok(())
}
```

WebSocket streaming:

```rust
use coinbase_advanced::ws::{WebSocketClient, Channel};
use futures::StreamExt;

#[tokio::main]
async fn main() -> coinbase_advanced::Result<()> {
    let client = WebSocketClient::builder().build()?;
    let mut stream = client.connect().await?;

    client.subscribe(&[
        Channel::Ticker { product_ids: vec!["BTC-USD".to_string()] },
    ]).await?;

    while let Some(msg) = stream.next().await {
        println!("{:?}", msg);
    }

    Ok(())
}
```

Configuration:

```rust
let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .sandbox(true)
    .rate_limiting(true)
    .build()?;
```

## API coverage

REST endpoints:

| Endpoint type | Implementation |
|----------|------------------------|
| Accounts | ✓ |
| Products | ✓ |
| Orders | ✓ |
| Fees | ✓ |
| Portfolios | ✓ |
| Convert | ✓ |
| Data | ✓ |
| Payment methods | ✓ |
| Perpetuals | ✓ |
| Futures | ✓ |
| Public | ✓ |

WebSocket endpoints:

| Endpoint type | Implementation |
|----------|------------------------|
| Heartbeats | ✓ |
| Status | ✓ |
| Ticker | ✓ |
| Ticker batch | ✓ |
| Level2 | ✓ |
| Candles | ✓ |
| Market trades | ✓ |
| User | ✓ |
| Futures balance summary | ✓ |

## Project structure

```text
.
├── src/                         # Core library implementation
│   ├── rest/                    # REST API client modules (accounts, orders, products, etc.)
│   ├── models/                  # Request/response types and shared data models
│   ├── ws/                      # WebSocket client, channels, and message parsing
├── examples/                    # Runnable usage examples for common API workflows
└── tests/                       # Integration tests for end-to-end API behavior
```
