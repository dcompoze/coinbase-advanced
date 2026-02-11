use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::{JWT_EXPIRY_SECONDS, JWT_ISSUER};
use crate::credentials::Credentials;
use crate::error::{Error, Result};

/// JWT header for Coinbase API authentication.
#[derive(Debug, Serialize)]
struct JwtHeader<'a> {
    alg: &'static str,
    kid: &'a str,
    nonce: String,
    typ: &'static str,
}

/// JWT claims for Coinbase API authentication.
#[derive(Debug, Serialize)]
struct JwtClaims<'a> {
    iss: &'static str,
    sub: &'a str,
    nbf: u64,
    exp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
}

/// Generate a JWT for authenticating with the Coinbase API.
///
/// # Arguments
/// * `credentials` - The API credentials
/// * `method` - The HTTP method (GET, POST, etc.)
/// * `path` - The request path (e.g., "/api/v3/brokerage/accounts")
///
/// # Returns
/// A signed JWT string suitable for the Authorization header.
pub fn generate_jwt(credentials: &Credentials, method: &str, path: &str) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::jwt(format!("Failed to get current time: {}", e)))?
        .as_secs();

    // Generate random nonce.
    let nonce = generate_nonce()?;

    // Build header.
    let header = JwtHeader {
        alg: "ES256",
        kid: credentials.api_key(),
        nonce,
        typ: "JWT",
    };

    // Build URI claim: "<METHOD> api.coinbase.com<path>"
    let uri = format!("{} api.coinbase.com{}", method.to_uppercase(), path);

    // Build claims.
    let claims = JwtClaims {
        iss: JWT_ISSUER,
        sub: credentials.api_key(),
        nbf: now,
        exp: now + JWT_EXPIRY_SECONDS,
        uri: Some(uri),
    };

    // Encode and sign.
    sign_jwt(&header, &claims, credentials)
}

/// Generate a JWT for WebSocket authentication (no URI claim).
pub(crate) fn generate_ws_jwt(credentials: &Credentials) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::jwt(format!("Failed to get current time: {}", e)))?
        .as_secs();

    let nonce = generate_nonce()?;

    let header = JwtHeader {
        alg: "ES256",
        kid: credentials.api_key(),
        nonce,
        typ: "JWT",
    };

    let claims = JwtClaims {
        iss: JWT_ISSUER,
        sub: credentials.api_key(),
        nbf: now,
        exp: now + JWT_EXPIRY_SECONDS,
        uri: None,
    };

    sign_jwt(&header, &claims, credentials)
}

/// Generate a random hex nonce.
fn generate_nonce() -> Result<String> {
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 16];
    ring::rand::SecureRandom::fill(&rng, &mut nonce_bytes)
        .map_err(|_| Error::jwt("Failed to generate random nonce"))?;
    Ok(hex::encode(nonce_bytes))
}

/// Sign the JWT with ES256.
fn sign_jwt<H: Serialize, C: Serialize>(
    header: &H,
    claims: &C,
    credentials: &Credentials,
) -> Result<String> {
    // Encode header and claims.
    let header_b64 = base64_url_encode(
        &serde_json::to_vec(header)
            .map_err(|e| Error::jwt(format!("Failed to encode header: {}", e)))?,
    );
    let claims_b64 = base64_url_encode(
        &serde_json::to_vec(claims)
            .map_err(|e| Error::jwt(format!("Failed to encode claims: {}", e)))?,
    );

    // Create signing input.
    let signing_input = format!("{}.{}", header_b64, claims_b64);

    // Parse the private key and sign.
    let signature = sign_es256(signing_input.as_bytes(), credentials.private_key())?;
    let signature_b64 = base64_url_encode(&signature);

    Ok(format!("{}.{}", signing_input, signature_b64))
}

/// Sign data with ES256 using the provided PEM private key.
fn sign_es256(data: &[u8], pem_key: &str) -> Result<Vec<u8>> {
    // Parse PEM to get the DER-encoded key.
    let der = parse_ec_private_key_pem(pem_key)?;

    // Create the key pair.
    let rng = SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &der, &rng)
        .map_err(|e| Error::jwt(format!("Failed to parse private key: {}", e)))?;

    // Sign the data.
    let signature = key_pair
        .sign(&rng, data)
        .map_err(|_| Error::jwt("Failed to sign JWT"))?;

    Ok(signature.as_ref().to_vec())
}

/// Parse a PEM-encoded EC private key to PKCS#8 DER format.
fn parse_ec_private_key_pem(pem: &str) -> Result<Vec<u8>> {
    // Find the base64 content between the PEM headers.
    let pem = pem.trim();

    // Handle both "EC PRIVATE KEY" (SEC1) and "PRIVATE KEY" (PKCS#8) formats.
    let (start_marker, end_marker, is_sec1) = if pem.contains("BEGIN EC PRIVATE KEY") {
        (
            "-----BEGIN EC PRIVATE KEY-----",
            "-----END EC PRIVATE KEY-----",
            true,
        )
    } else if pem.contains("BEGIN PRIVATE KEY") {
        (
            "-----BEGIN PRIVATE KEY-----",
            "-----END PRIVATE KEY-----",
            false,
        )
    } else {
        return Err(Error::jwt("Invalid PEM format: missing BEGIN marker"));
    };

    let start = pem
        .find(start_marker)
        .ok_or_else(|| Error::jwt("Invalid PEM format: missing BEGIN marker"))?
        + start_marker.len();
    let end = pem
        .find(end_marker)
        .ok_or_else(|| Error::jwt("Invalid PEM format: missing END marker"))?;

    let b64_content: String = pem[start..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let der = base64_decode(&b64_content)?;

    if is_sec1 {
        // Convert SEC1 to PKCS#8 format.
        convert_sec1_to_pkcs8(&der)
    } else {
        // Already in PKCS#8 format.
        Ok(der)
    }
}

/// Convert SEC1 EC private key to PKCS#8 format.
///
/// SEC1 format (from "EC PRIVATE KEY"):
/// ECPrivateKey ::= SEQUENCE {
///   version        INTEGER { ecPrivkeyVer1(1) },
///   privateKey     OCTET STRING,
///   parameters [0] ECParameters {{ NamedCurve }} OPTIONAL,
///   publicKey  [1] BIT STRING OPTIONAL
/// }
///
/// PKCS#8 format (for ring):
/// PrivateKeyInfo ::= SEQUENCE {
///   version         Version,
///   algorithm       AlgorithmIdentifier,
///   privateKey      OCTET STRING (contains SEC1 ECPrivateKey)
/// }
fn convert_sec1_to_pkcs8(sec1_der: &[u8]) -> Result<Vec<u8>> {
    // Construct the PKCS#8 structure.
    // The SEC1 key needs to be wrapped in an OCTET STRING.
    let sec1_len = sec1_der.len();

    // Build OCTET STRING for the private key.
    let mut octet_string = Vec::new();
    octet_string.push(0x04); // OCTET STRING tag
    if sec1_len < 128 {
        octet_string.push(sec1_len as u8);
    } else {
        octet_string.push(0x81);
        octet_string.push(sec1_len as u8);
    }
    octet_string.extend_from_slice(sec1_der);

    // Build AlgorithmIdentifier.
    let alg_id: &[u8] = &[
        0x30, 0x13, // SEQUENCE
        0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, // OID ecPublicKey
        0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, // OID prime256v1
    ];

    // Build version.
    let version: &[u8] = &[0x02, 0x01, 0x00]; // INTEGER 0

    // Calculate total length.
    let content_len = version.len() + alg_id.len() + octet_string.len();

    // Build final PKCS#8 structure.
    let mut pkcs8 = Vec::new();
    pkcs8.push(0x30); // SEQUENCE tag
    if content_len < 128 {
        pkcs8.push(content_len as u8);
    } else if content_len < 256 {
        pkcs8.push(0x81);
        pkcs8.push(content_len as u8);
    } else {
        pkcs8.push(0x82);
        pkcs8.push((content_len >> 8) as u8);
        pkcs8.push((content_len & 0xff) as u8);
    }
    pkcs8.extend_from_slice(version);
    pkcs8.extend_from_slice(alg_id);
    pkcs8.extend_from_slice(&octet_string);

    Ok(pkcs8)
}

/// Base64 URL-safe encoding without padding.
fn base64_url_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = data.get(i + 1).copied().unwrap_or(0) as usize;
        let b2 = data.get(i + 2).copied().unwrap_or(0) as usize;

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(alphabet[(n >> 18) & 0x3f] as char);
        result.push(alphabet[(n >> 12) & 0x3f] as char);

        if i + 1 < data.len() {
            result.push(alphabet[(n >> 6) & 0x3f] as char);
        }
        if i + 2 < data.len() {
            result.push(alphabet[n & 0x3f] as char);
        }

        i += 3;
    }

    result
}

/// Standard Base64 decoding.
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    lookup[b'-' as usize] = 62; // URL-safe variant
    lookup[b'_' as usize] = 63; // URL-safe variant

    let input: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);

    let mut i = 0;
    while i < input.len() {
        let b0 = lookup[input[i] as usize] as usize;
        let b1 = input
            .get(i + 1)
            .map(|&b| lookup[b as usize] as usize)
            .unwrap_or(0);
        let b2 = input
            .get(i + 2)
            .map(|&b| lookup[b as usize] as usize)
            .unwrap_or(0);
        let b3 = input
            .get(i + 3)
            .map(|&b| lookup[b as usize] as usize)
            .unwrap_or(0);

        if b0 == 255 || b1 == 255 {
            return Err(Error::jwt("Invalid base64 character"));
        }

        let n = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;

        result.push((n >> 16) as u8);
        if i + 2 < input.len() && b2 != 255 {
            result.push((n >> 8) as u8);
        }
        if i + 3 < input.len() && b3 != 255 {
            result.push(n as u8);
        }

        i += 4;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_url_encode() {
        assert_eq!(base64_url_encode(b"hello"), "aGVsbG8");
        assert_eq!(base64_url_encode(b"hello world"), "aGVsbG8gd29ybGQ");
    }

    #[test]
    fn test_generate_ws_jwt_compiles() {
        // Just verify the function exists and is callable
        // Actual JWT generation requires valid credentials
        let _ = generate_ws_jwt;
    }

    #[test]
    fn test_base64_decode() {
        let decoded = base64_decode("aGVsbG8").unwrap();
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn test_generate_nonce() {
        let nonce = generate_nonce().unwrap();
        assert_eq!(nonce.len(), 32); // 16 bytes = 32 hex chars
    }
}
