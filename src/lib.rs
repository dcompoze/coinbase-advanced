//! # Coinbase Advanced Trade API Client
//!
//! A Rust client library for the Coinbase Advanced Trade API.
//!
//! ## Features
//!
//! - REST API client with JWT authentication
//! - Strongly typed request/response models
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
//! - An EC private key in PEM format
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

pub mod rest;
pub mod models;
pub mod rate_limit;
pub mod ws;

// Re-export main types.
pub use client::{RestClient, RestClientBuilder};
pub use credentials::Credentials;
pub use error::{Error, Result};

// Re-export API types for convenience.
pub use rest::{
    AccountsApi, ConvertApi, DataApi, FeesApi, FuturesApi, OrdersApi, PaymentMethodsApi,
    PerpetualsApi, PortfoliosApi, ProductsApi, PublicApi, ServerTime,
};

// Re-export constants for advanced usage.
pub mod consts {
    pub use crate::constants::*;
}
