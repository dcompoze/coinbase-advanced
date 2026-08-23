//! # Coinbase Advanced Trade API Client
//!
//! A Rust client library for the Coinbase Advanced Trade API.
//!
//! ## Features
//!
//! - REST API client with JWT authentication (ES256 and EdDSA keys)
//! - WebSocket streaming with auto-reconnect and resubscribe
//! - Strongly typed request/response models
//! - Optional client-side rate limiting and retries with backoff
//! - Async/await support with tokio
//! - Support for both production and sandbox environments
//!
//! ## Quick Start
//!
//! ```no_run
//! use coinbase_advanced::{Credentials, RestClient};
//!
//! #[tokio::main]
//! async fn main() -> coinbase_advanced::Result<()> {
//!     // Create credentials from environment variables
//!     let credentials = Credentials::from_env()?;
//!
//!     // Build the client
//!     let client = RestClient::builder()
//!         .credentials(credentials)
//!         .build()?;
//!
//!     // Make API calls...
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! The Coinbase Advanced Trade API uses JWT (JSON Web Tokens) for authentication.
//! You'll need:
//!
//! - An API key (in the format `organizations/{org_id}/apiKeys/{key_id}`)
//! - A private key: EC P-256 in PEM format (ES256) or Ed25519 as PKCS#8 PEM
//!   or raw base64 (EdDSA)
//!
//! These can be obtained from the Coinbase Developer Platform.
//!
//! ## Sandbox Mode
//!
//! For testing, you can use the sandbox environment:
//!
//! ```no_run
//! # use coinbase_advanced::{Credentials, RestClient};
//! let client = RestClient::builder()
//!     .credentials(Credentials::from_env().unwrap())
//!     .sandbox(true)
//!     .build()
//!     .unwrap();
//! ```

mod client;
mod constants;
mod credentials;
mod error;
mod jwt;

pub mod models;
pub mod rate_limit;
pub mod rest;
pub mod ws;

pub use client::{RestClient, RestClientBuilder};
pub use credentials::Credentials;
pub use error::{Error, Result};

pub use rest::{
    AccountsApi, ConvertApi, DataApi, FeesApi, FuturesApi, OrdersApi, PaymentMethodsApi,
    PerpetualsApi, PortfoliosApi, ProductsApi, PublicApi, ServerTime,
};

pub mod consts {
    pub use crate::constants::*;
}
