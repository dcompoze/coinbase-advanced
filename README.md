# coinbase-advanced

A Rust async client library for the [Coinbase Advanced Trade API](https://docs.cdp.coinbase.com/advanced-trade-api/docs/welcome).

- Complete REST API coverage for Coinbase Advanced Trade
- WebSocket support for real-time market data with auto-reconnect
- JWT authentication with ECDSA (ES256) and Ed25519 (EdDSA) keys
- Optional client-side rate limiting and retries with exponential backoff
- Automatic cursor pagination helpers (`list_all`)
- Async/await with Tokio/Reqwest

## Library

Authentication:

Obtain API credentials from the [Coinbase Developer Platform](https://portal.cdp.coinbase.com/).
Both ECDSA (EC P-256 PEM) and Ed25519 (PKCS#8 PEM or raw base64) keys are supported.

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
use coinbase_advanced::rate_limit::RateLimitConfig;

let client = RestClient::builder()
    .credentials(Credentials::from_env()?)
    .sandbox(true)
    .rate_limiting(true)
    .retry_config(RateLimitConfig::new().with_max_retries(3))
    .build()?;
```

WebSocket reliability options:

```rust
let client = WebSocketClient::builder()
    .credentials(Credentials::from_env()?)
    .auto_reconnect(true)
    .validate_sequence(true)
    .build()?;
```

Note: On 2026-09-09 Coinbase moves international derivatives (INTX) to a
Deribit-powered gateway with a JSON-RPC API. The `/intx` endpoints in this
library follow the current Advanced Trade REST API.

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
