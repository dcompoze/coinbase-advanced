use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Method, Response};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_tracing::TracingMiddleware;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use url::Url;

use crate::constants::{
    API_BASE_URL, API_PATH_PREFIX, API_SANDBOX_BASE_URL, DEFAULT_TIMEOUT_SECONDS, USER_AGENT as UA,
};
use crate::credentials::Credentials;
use crate::error::{Error, Result};
use crate::jwt::generate_jwt_with_host;
use crate::rate_limit::{RateLimitConfig, RateLimitInfo, RateLimiter};
use crate::rest::{
    AccountsApi, ConvertApi, DataApi, FeesApi, FuturesApi, OrdersApi, PaymentMethodsApi,
    PerpetualsApi, PortfoliosApi, ProductsApi, PublicApi,
};

/// Builder for constructing a [`RestClient`].
#[derive(Debug, Clone)]
pub struct RestClientBuilder {
    credentials: Option<Credentials>,
    sandbox: bool,
    timeout: Duration,
    rate_limiting: bool,
    retry_config: Option<RateLimitConfig>,
    base_url: Option<String>,
}

impl Default for RestClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RestClientBuilder {
    /// Create a new client builder.
    pub fn new() -> Self {
        Self {
            credentials: None,
            sandbox: false,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            rate_limiting: false,
            retry_config: None,
            base_url: None,
        }
    }

    /// Set the API credentials.
    ///
    /// Required for authenticated endpoints. Public endpoints can be accessed without credentials.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Enable sandbox mode.
    ///
    /// When enabled, requests are sent to the Coinbase sandbox environment.
    pub fn sandbox(mut self, enabled: bool) -> Self {
        self.sandbox = enabled;
        self
    }

    /// Set the request timeout.
    ///
    /// Default is 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable or disable rate limiting.
    ///
    /// When enabled, the client will automatically throttle requests to avoid
    /// hitting Coinbase API rate limits.
    pub fn rate_limiting(mut self, enabled: bool) -> Self {
        self.rate_limiting = enabled;
        self
    }

    /// Override the API base URL.
    ///
    /// Takes precedence over `sandbox`. Useful for testing against a mock server.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Enable automatic retries with exponential backoff.
    ///
    /// Retries transient failures (429 and 5xx responses, connection errors)
    /// according to the given configuration.
    pub fn retry_config(mut self, config: RateLimitConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Build the REST client.
    pub fn build(self) -> Result<RestClient> {
        let base_url = self.base_url.clone().unwrap_or_else(|| {
            if self.sandbox {
                API_SANDBOX_BASE_URL.to_string()
            } else {
                API_BASE_URL.to_string()
            }
        });
        let base_url = base_url.trim_end_matches('/').to_string();

        // The JWT `uri` claim must name the host actually being called.
        let host = Url::parse(&base_url)
            .map_err(Error::Url)?
            .authority()
            .to_string();

        let reqwest_client = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| Error::config(format!("Failed to create HTTP client: {}", e)))?;

        let mut client_builder = reqwest_middleware::ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::default());

        if let Some(ref config) = self.retry_config
            && config.enabled
            && config.max_retries > 0
        {
            let policy = ExponentialBackoff::builder()
                .retry_bounds(config.initial_backoff, config.max_backoff)
                .build_with_max_retries(config.max_retries);
            client_builder = client_builder.with(RetryTransientMiddleware::new_with_policy(policy));
        }

        let (private_limiter, public_limiter) = if self.rate_limiting {
            (
                Some(RateLimiter::for_private_rest()),
                Some(RateLimiter::for_public_rest()),
            )
        } else {
            (None, None)
        };

        Ok(RestClient {
            http_client: client_builder.build(),
            base_url,
            host,
            credentials: self.credentials,
            private_limiter,
            public_limiter,
        })
    }
}

/// REST client for the Coinbase Advanced Trade API.
#[derive(Clone)]
pub struct RestClient {
    http_client: ClientWithMiddleware,
    base_url: String,
    host: String,
    credentials: Option<Credentials>,
    private_limiter: Option<RateLimiter>,
    public_limiter: Option<RateLimiter>,
}

impl RestClient {
    /// Create a new client builder.
    pub fn builder() -> RestClientBuilder {
        RestClientBuilder::new()
    }

    /// Check if the client has credentials configured.
    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    /// Access the Accounts API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let accounts = client.accounts().list_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn accounts(&self) -> AccountsApi<'_> {
        AccountsApi::new(self)
    }

    /// Access the Products API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let products = client.products().list_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn products(&self) -> ProductsApi<'_> {
        ProductsApi::new(self)
    }

    /// Access the Public API.
    ///
    /// These endpoints do not require authentication.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::RestClient;
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder().build()?;
    ///
    /// let time = client.public().get_time().await?;
    /// println!("Server time: {}", time.iso);
    /// # Ok(())
    /// # }
    /// ```
    pub fn public(&self) -> PublicApi<'_> {
        PublicApi::new(self)
    }

    /// Access the Orders API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::ListOrdersParams};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let orders = client.orders().list_all(ListOrdersParams::new()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn orders(&self) -> OrdersApi<'_> {
        OrdersApi::new(self)
    }

    /// Access the Fees API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let summary = client.fees().get_transaction_summary().await?;
    /// println!("Fee tier: {}", summary.fee_tier.pricing_tier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn fees(&self) -> FeesApi<'_> {
        FeesApi::new(self)
    }

    /// Access the Data API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let permissions = client.data().get_key_permissions().await?;
    /// println!("Can trade: {}", permissions.can_trade);
    /// # Ok(())
    /// # }
    /// ```
    pub fn data(&self) -> DataApi<'_> {
        DataApi::new(self)
    }

    /// Access the Payment Methods API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let payment_methods = client.payment_methods().list().await?;
    /// for pm in payment_methods {
    ///     println!("{}: {}", pm.name, pm.payment_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn payment_methods(&self) -> PaymentMethodsApi<'_> {
        PaymentMethodsApi::new(self)
    }

    /// Access the Portfolios API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let portfolios = client.portfolios().list().await?;
    /// for portfolio in portfolios {
    ///     println!("{}: {}", portfolio.uuid, portfolio.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn portfolios(&self) -> PortfoliosApi<'_> {
        PortfoliosApi::new(self)
    }

    /// Access the Convert API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials, models::CreateConvertQuoteRequest};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let request = CreateConvertQuoteRequest::new("USD-account", "USDC-account", "100.00");
    /// let quote = client.convert().create_quote(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn convert(&self) -> ConvertApi<'_> {
        ConvertApi::new(self)
    }

    /// Access the Perpetuals (INTX) API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let response = client.perpetuals().list_positions("portfolio-uuid").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn perpetuals(&self) -> PerpetualsApi<'_> {
        PerpetualsApi::new(self)
    }

    /// Access the Futures (CFM) API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use coinbase_advanced::{RestClient, Credentials};
    /// # async fn example() -> coinbase_advanced::Result<()> {
    /// let client = RestClient::builder()
    ///     .credentials(Credentials::from_env()?)
    ///     .build()?;
    ///
    /// let positions = client.futures().list_positions().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn futures(&self) -> FuturesApi<'_> {
        FuturesApi::new(self)
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a full URL for an API endpoint.
    fn build_url(&self, endpoint: &str) -> Result<Url> {
        let path = format!("{}{}", API_PATH_PREFIX, endpoint);
        let url_str = format!("{}{}", self.base_url, path);
        Url::parse(&url_str).map_err(Error::Url)
    }

    /// Make a GET request to an authenticated endpoint.
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        self.execute::<(), (), T>(Method::GET, endpoint, None, None, true)
            .await
    }

    /// Make a GET request with query parameters.
    pub async fn get_with_query<Q: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &Q,
    ) -> Result<T> {
        self.execute::<Q, (), T>(Method::GET, endpoint, Some(query), None, true)
            .await
    }

    /// Make a POST request.
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        self.execute::<(), B, T>(Method::POST, endpoint, None, Some(body), true)
            .await
    }

    /// Make a PUT request.
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        self.execute::<(), B, T>(Method::PUT, endpoint, None, Some(body), true)
            .await
    }

    /// Make a DELETE request.
    pub async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        self.execute::<(), (), T>(Method::DELETE, endpoint, None, None, true)
            .await
    }

    /// Make a public (unauthenticated) GET request.
    pub async fn public_get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        self.execute::<(), (), T>(Method::GET, endpoint, None, None, false)
            .await
    }

    /// Make a public GET request with query parameters.
    pub async fn public_get_with_query<Q: Serialize, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &Q,
    ) -> Result<T> {
        self.execute::<Q, (), T>(Method::GET, endpoint, Some(query), None, false)
            .await
    }

    /// Execute a request against the API.
    ///
    /// Authenticated requests use the private rate limit bucket and carry a
    /// bearer JWT, public requests use the public bucket and no auth header.
    async fn execute<Q: Serialize, B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query: Option<&Q>,
        body: Option<&B>,
        authenticated: bool,
    ) -> Result<T> {
        let limiter = if authenticated {
            &self.private_limiter
        } else {
            &self.public_limiter
        };
        if let Some(limiter) = limiter {
            limiter.acquire().await;
        }

        let mut url = self.build_url(endpoint)?;

        if let Some(q) = query {
            let query_string = serde_urlencoded::to_string(q)
                .map_err(|e| Error::request(format!("Failed to encode query: {}", e)))?;
            if !query_string.is_empty() {
                url.set_query(Some(&query_string));
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));

        if authenticated {
            let credentials = self
                .credentials
                .as_ref()
                .ok_or_else(|| Error::auth("Credentials required for authenticated endpoints"))?;
            // The signed URI is method + host + path, without the query string.
            let jwt = generate_jwt_with_host(credentials, method.as_str(), &self.host, url.path())?;
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", jwt))
                    .map_err(|e| Error::request(format!("Invalid auth header: {}", e)))?,
            );
        }

        let mut request = self.http_client.request(method, url).headers(headers);

        if let Some(b) = body {
            request = request.json(b);
        }

        let response = request.send().await.map_err(map_middleware_error)?;

        self.handle_response(response).await
    }

    /// Handle the API response.
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        // Surface rate limit headers for observability.
        let rate_info = RateLimitInfo::from_headers(response.headers());
        if rate_info.is_exhausted() {
            tracing::warn!(
                limit = ?rate_info.limit,
                reset = ?rate_info.reset,
                "API rate limit exhausted"
            );
        } else if let Some(remaining) = rate_info.remaining {
            tracing::trace!(remaining, "API rate limit remaining");
        }

        // Check for rate limiting.
        // Only the seconds form of `retry-after` is parsed,
        // an HTTP date value yields `None`.
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs);

            return Err(Error::RateLimited { retry_after });
        }

        let body = response.text().await.map_err(Error::Http)?;

        // Check for error status codes.
        if !status.is_success() {
            // Try to parse error message from response.
            let message = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .or_else(|| v.get("error"))
                        .or_else(|| v.get("error_description"))
                        .and_then(|m| m.as_str())
                        .map(String::from)
                })
                .unwrap_or_else(|| format!("HTTP {} error", status.as_u16()));

            return Err(Error::api(status.as_u16(), message, Some(body)));
        }

        // Parse successful response.
        // Some endpoints return an empty body on success,
        // treat it as JSON null so `()` and `Value` targets parse.
        let json = if body.trim().is_empty() {
            "null"
        } else {
            body.as_str()
        };
        serde_json::from_str(json)
            .map_err(|e| Error::parse(format!("Failed to parse response: {}", e), Some(body)))
    }
}

/// Map a middleware error to a crate error.
fn map_middleware_error(error: reqwest_middleware::Error) -> Error {
    match error {
        reqwest_middleware::Error::Reqwest(e) => Error::Http(e),
        // Retry middleware wraps the final transport error,
        // unwrap it so `Error::is_retryable` still classifies it.
        reqwest_middleware::Error::Middleware(e) => match e.downcast::<reqwest::Error>() {
            Ok(e) => Error::Http(e),
            Err(e) => Error::request(format!("Middleware error: {}", e)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = RestClientBuilder::new();
        assert!(builder.credentials.is_none());
        assert!(!builder.sandbox);
    }

    #[test]
    fn test_builder_sandbox() {
        let client = RestClient::builder().sandbox(true).build().unwrap();
        assert_eq!(client.base_url(), API_SANDBOX_BASE_URL);
    }

    #[test]
    fn test_builder_production() {
        let client = RestClient::builder().sandbox(false).build().unwrap();
        assert_eq!(client.base_url(), API_BASE_URL);
    }

    #[test]
    fn test_build_url() {
        let client = RestClient::builder().build().unwrap();
        let url = client.build_url("/accounts").unwrap();
        assert_eq!(
            url.as_str(),
            "https://api.coinbase.com/api/v3/brokerage/accounts"
        );
    }
}
