# coinbase-advanced

A Rust async client library for the [Coinbase Advanced Trade API](https://docs.cdp.coinbase.com/advanced-trade-api/docs/welcome).

- Complete REST API coverage for Coinbase Advanced Trade
- WebSocket support for real-time market data
- JWT (ES256) authentication
- Optional client-side rate limiting
- Async/await with Tokio

# Usage

## Authentication

Obtain API credentials from the [Coinbase Developer Platform](https://portal.cdp.coinbase.com/).
You need an API key and an EC private key in PEM format.

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

## Get account balances

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

## Place an order

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

## WebSocket streaming

```rust
use coinbase_advanced::websocket::{WebSocketClient, Channel};
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

## Configuration

## Sandbox mode

```rust
let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .sandbox(true)
    .build()?;
```

## Rate limiting

```rust
let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .rate_limiting(true)
    .build()?;
```

# API coverage

| REST API | WebSocket channels |
|----------|-------------------|
| Accounts | Heartbeats |
| Products | Ticker |
| Orders | Level2 |
| Fees | Candles |
| Portfolios | Market Trades |
| Convert | User |
| Data | Status |
| Payment Methods | |
| Perpetuals | |
| Futures | |
| Public | |

# Examples

See the [examples](examples/) directory:

- `basic.rs` - Account and product queries
- `orders.rs` - Order placement and management
- `websocket.rs` - Real-time streaming

Run examples with:

```sh
cargo run --example basic
```
