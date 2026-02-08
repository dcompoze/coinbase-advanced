use secrecy::{ExposeSecret, SecretString};
use std::env;

use crate::error::{Error, Result};

/// Credentials for authenticating with the Coinbase API.
#[derive(Clone)]
pub struct Credentials {
    /// The API key (e.g., "organizations/{org_id}/apiKeys/{key_id}")
    api_key: String,
    /// The private key in PEM format (EC P-256)
    private_key: SecretString,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("api_key", &self.api_key)
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

impl Credentials {
    /// Create new credentials from an API key and private key.
    ///
    /// # Arguments
    /// * `api_key` - The CDP API key identifier
    /// * `private_key` - The EC private key in PEM format
    ///
    /// # Example
    /// ```no_run
    /// use coinbase_advanced::Credentials;
    ///
    /// let creds = Credentials::new(
    ///     "organizations/xxx/apiKeys/yyy",
    ///     "-----BEGIN EC PRIVATE KEY-----\n...\n-----END EC PRIVATE KEY-----\n"
    /// ).unwrap();
    /// ```
    pub fn new(api_key: impl Into<String>, private_key: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();
        let private_key = private_key.into();

        // Basic validation.
        if api_key.is_empty() {
            return Err(Error::config("API key cannot be empty"));
        }
        if private_key.is_empty() {
            return Err(Error::config("Private key cannot be empty"));
        }
        if !private_key.contains("BEGIN EC PRIVATE KEY") {
            return Err(Error::config(
                "Private key must be in PEM format (EC PRIVATE KEY)",
            ));
        }

        Ok(Self {
            api_key,
            private_key: SecretString::from(private_key),
        })
    }

    /// Create credentials from environment variables.
    ///
    /// Reads from:
    /// - `COINBASE_API_KEY` - The CDP API key
    /// - `COINBASE_PRIVATE_KEY` - The EC private key in PEM format
    ///
    /// Note: The private key should have literal `\n` characters replaced with actual newlines,
    /// or be stored in a file and read separately.
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("COINBASE_API_KEY")
            .map_err(|_| Error::config("COINBASE_API_KEY environment variable not set"))?;

        let private_key = env::var("COINBASE_PRIVATE_KEY")
            .map_err(|_| Error::config("COINBASE_PRIVATE_KEY environment variable not set"))?;

        // Handle escaped newlines in environment variable.
        let private_key = private_key.replace("\\n", "\n");

        Self::new(api_key, private_key)
    }

    /// Get the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the private key (exposed for JWT signing).
    pub(crate) fn private_key(&self) -> &str {
        self.private_key.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "organizations/test-org/apiKeys/test-key";
    const TEST_PRIVATE_KEY: &str = "-----BEGIN EC PRIVATE KEY-----
MHQCAQEEIBkg4LVWM9nuwNKXPgFvbVwUxYdLlpfazMKfqTgs1RwQoAcGBSuBBAAK
oUQDQgAEm8+paLliHKY9RI5gZ8SBOHwAFcPf27pePzVTaWLSmzxanOT/MO6DPqMW
1pNcpaLerRLCPCchK31waXYjKEf3Dw==
-----END EC PRIVATE KEY-----
";

    #[test]
    fn test_new_credentials() {
        let creds = Credentials::new(TEST_KEY, TEST_PRIVATE_KEY).unwrap();
        assert_eq!(creds.api_key(), TEST_KEY);
    }

    #[test]
    fn test_empty_api_key() {
        let result = Credentials::new("", TEST_PRIVATE_KEY);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_private_key() {
        let result = Credentials::new(TEST_KEY, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_private_key_format() {
        let result = Credentials::new(TEST_KEY, "not a pem key");
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_redacts_private_key() {
        let creds = Credentials::new(TEST_KEY, TEST_PRIVATE_KEY).unwrap();
        let debug = format!("{:?}", creds);
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("BEGIN EC PRIVATE KEY"));
    }
}
