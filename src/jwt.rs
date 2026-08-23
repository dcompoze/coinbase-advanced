use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, Ed25519KeyPair};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::{JWT_EXPIRY_SECONDS, JWT_ISSUER};
use crate::credentials::Credentials;
use crate::error::{Error, Result};

/// The kind of private key held by a set of credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyKind {
    /// EC P-256 key in SEC1 PEM format (`BEGIN EC PRIVATE KEY`).
    EcdsaSec1,
    /// EC P-256 key in PKCS#8 PEM format (`BEGIN PRIVATE KEY`).
    EcdsaPkcs8,
    /// Ed25519 key in PKCS#8 PEM format (`BEGIN PRIVATE KEY`).
    Ed25519Pkcs8,
    /// Ed25519 key as raw base64 (32-byte seed or 64-byte seed plus public key).
    Ed25519Raw,
}

impl KeyKind {
    /// The JWT `alg` header value for this key kind.
    fn algorithm(self) -> &'static str {
        match self {
            KeyKind::EcdsaSec1 | KeyKind::EcdsaPkcs8 => "ES256",
            KeyKind::Ed25519Pkcs8 | KeyKind::Ed25519Raw => "EdDSA",
        }
    }
}

/// Detect the kind of private key from its textual representation.
///
/// Accepts SEC1 EC PEM, PKCS#8 PEM (EC or Ed25519), and raw base64 Ed25519 keys
/// (32-byte seed or 64-byte seed plus public key).
pub(crate) fn detect_key_kind(private_key: &str) -> Result<KeyKind> {
    let key = private_key.trim();

    if key.contains("BEGIN EC PRIVATE KEY") {
        return Ok(KeyKind::EcdsaSec1);
    }

    if key.contains("BEGIN PRIVATE KEY") {
        let der = pem_body(
            key,
            "-----BEGIN PRIVATE KEY-----",
            "-----END PRIVATE KEY-----",
        )?;
        // Ed25519 OID 1.3.101.112 encoded as 06 03 2B 65 70.
        // Only the AlgorithmIdentifier near the start of the DER is checked
        // so random key material cannot cause a misdetection.
        const ED25519_OID: [u8; 5] = [0x06, 0x03, 0x2b, 0x65, 0x70];
        let prefix = der.get(..16).unwrap_or(&der);
        if prefix.windows(ED25519_OID.len()).any(|w| w == ED25519_OID) {
            return Ok(KeyKind::Ed25519Pkcs8);
        }
        return Ok(KeyKind::EcdsaPkcs8);
    }

    // No PEM markers, try raw base64 Ed25519.
    let compact: String = key.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64_decode(&compact)
        .map_err(|_| Error::config("Private key must be PEM or base64 Ed25519"))?;
    match bytes.len() {
        32 | 64 => Ok(KeyKind::Ed25519Raw),
        n => Err(Error::config(format!(
            "Unsupported raw key length {} (expected 32 or 64 bytes)",
            n
        ))),
    }
}

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

/// Generate a JWT with an explicit host in the `uri` claim.
///
/// The host must match the host actually being called (production or sandbox).
pub(crate) fn generate_jwt_with_host(
    credentials: &Credentials,
    method: &str,
    host: &str,
    path: &str,
) -> Result<String> {
    let uri = format!("{} {}{}", method.to_uppercase(), host, path);
    generate_jwt_internal(credentials, Some(uri))
}

/// Generate a JWT for WebSocket authentication (no URI claim).
pub(crate) fn generate_ws_jwt(credentials: &Credentials) -> Result<String> {
    generate_jwt_internal(credentials, None)
}

fn generate_jwt_internal(credentials: &Credentials, uri: Option<String>) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::jwt(format!("Failed to get current time: {}", e)))?
        .as_secs();

    let header = JwtHeader {
        alg: credentials.key_kind().algorithm(),
        kid: credentials.api_key(),
        nonce: generate_nonce()?,
        typ: "JWT",
    };

    let claims = JwtClaims {
        iss: JWT_ISSUER,
        sub: credentials.api_key(),
        nbf: now,
        exp: now + JWT_EXPIRY_SECONDS,
        uri,
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

/// Sign the JWT with the algorithm matching the credential key kind.
fn sign_jwt<H: Serialize, C: Serialize>(
    header: &H,
    claims: &C,
    credentials: &Credentials,
) -> Result<String> {
    let header_b64 = base64_url_encode(
        &serde_json::to_vec(header)
            .map_err(|e| Error::jwt(format!("Failed to encode header: {}", e)))?,
    );
    let claims_b64 = base64_url_encode(
        &serde_json::to_vec(claims)
            .map_err(|e| Error::jwt(format!("Failed to encode claims: {}", e)))?,
    );

    let signing_input = format!("{}.{}", header_b64, claims_b64);

    let signature = match credentials.key_kind() {
        KeyKind::EcdsaSec1 | KeyKind::EcdsaPkcs8 => {
            sign_es256(signing_input.as_bytes(), credentials.private_key())?
        }
        KeyKind::Ed25519Pkcs8 => {
            sign_ed25519_pkcs8(signing_input.as_bytes(), credentials.private_key())?
        }
        KeyKind::Ed25519Raw => {
            sign_ed25519_raw(signing_input.as_bytes(), credentials.private_key())?
        }
    };
    let signature_b64 = base64_url_encode(&signature);

    Ok(format!("{}.{}", signing_input, signature_b64))
}

/// Sign data with ES256 using the provided PEM private key.
fn sign_es256(data: &[u8], pem_key: &str) -> Result<Vec<u8>> {
    let der = parse_ec_private_key_pem(pem_key)?;

    let rng = SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &der, &rng)
        .map_err(|e| Error::jwt(format!("Failed to parse private key: {}", e)))?;

    let signature = key_pair
        .sign(&rng, data)
        .map_err(|_| Error::jwt("Failed to sign JWT"))?;

    Ok(signature.as_ref().to_vec())
}

/// Sign data with Ed25519 using a PKCS#8 PEM key.
fn sign_ed25519_pkcs8(data: &[u8], key: &str) -> Result<Vec<u8>> {
    let der = pem_body(
        key.trim(),
        "-----BEGIN PRIVATE KEY-----",
        "-----END PRIVATE KEY-----",
    )?;
    let key_pair = Ed25519KeyPair::from_pkcs8_maybe_unchecked(&der)
        .map_err(|e| Error::jwt(format!("Failed to parse Ed25519 key: {}", e)))?;

    Ok(key_pair.sign(data).as_ref().to_vec())
}

/// Sign data with Ed25519 using a raw base64 key.
fn sign_ed25519_raw(data: &[u8], key: &str) -> Result<Vec<u8>> {
    let compact: String = key.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64_decode(&compact)?;
    let key_pair = match bytes.len() {
        32 => Ed25519KeyPair::from_seed_unchecked(&bytes)
            .map_err(|e| Error::jwt(format!("Failed to parse Ed25519 seed: {}", e)))?,
        64 => Ed25519KeyPair::from_seed_and_public_key(&bytes[..32], &bytes[32..])
            .map_err(|e| Error::jwt(format!("Failed to parse Ed25519 key: {}", e)))?,
        n => {
            return Err(Error::jwt(format!(
                "Unsupported raw Ed25519 key length {}",
                n
            )));
        }
    };

    Ok(key_pair.sign(data).as_ref().to_vec())
}

/// Extract and decode the base64 body between PEM markers.
fn pem_body(pem: &str, start_marker: &str, end_marker: &str) -> Result<Vec<u8>> {
    let start = pem
        .find(start_marker)
        .ok_or_else(|| Error::jwt("Invalid PEM format: missing BEGIN marker"))?
        + start_marker.len();
    // Search for the END marker after the BEGIN marker so a
    // malformed blob cannot produce an inverted slice.
    let end = pem[start..]
        .find(end_marker)
        .ok_or_else(|| Error::jwt("Invalid PEM format: missing END marker"))?
        + start;

    let b64_content: String = pem[start..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    base64_decode(&b64_content)
}

/// Parse a PEM-encoded EC private key to PKCS#8 DER format.
fn parse_ec_private_key_pem(pem: &str) -> Result<Vec<u8>> {
    let pem = pem.trim();

    if pem.contains("BEGIN EC PRIVATE KEY") {
        let der = pem_body(
            pem,
            "-----BEGIN EC PRIVATE KEY-----",
            "-----END EC PRIVATE KEY-----",
        )?;
        convert_sec1_to_pkcs8(&der)
    } else if pem.contains("BEGIN PRIVATE KEY") {
        pem_body(
            pem,
            "-----BEGIN PRIVATE KEY-----",
            "-----END PRIVATE KEY-----",
        )
    } else {
        Err(Error::jwt("Invalid PEM format: missing BEGIN marker"))
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
    // The SEC1 key needs to be wrapped in an OCTET STRING.
    let mut octet_string = Vec::new();
    octet_string.push(0x04); // OCTET STRING tag
    push_der_length(&mut octet_string, sec1_der.len());
    octet_string.extend_from_slice(sec1_der);

    let alg_id: &[u8] = &[
        0x30, 0x13, // SEQUENCE
        0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, // OID ecPublicKey
        0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, // OID prime256v1
    ];

    let version: &[u8] = &[0x02, 0x01, 0x00]; // INTEGER 0

    let content_len = version.len() + alg_id.len() + octet_string.len();

    let mut pkcs8 = Vec::new();
    pkcs8.push(0x30); // SEQUENCE tag
    push_der_length(&mut pkcs8, content_len);
    pkcs8.extend_from_slice(version);
    pkcs8.extend_from_slice(alg_id);
    pkcs8.extend_from_slice(&octet_string);

    Ok(pkcs8)
}

/// Push a DER length field (short, one, or two byte long form).
fn push_der_length(out: &mut Vec<u8>, length: usize) {
    if length < 128 {
        out.push(length as u8);
    } else if length < 256 {
        out.push(0x81);
        out.push(length as u8);
    } else {
        out.push(0x82);
        out.push((length >> 8) as u8);
        out.push((length & 0xff) as u8);
    }
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

/// Base64 decoding that accepts both the standard and URL-safe alphabets.
///
/// Padding is allowed only at the end. Any other invalid character or an
/// invalid length is rejected instead of producing wrong bytes.
pub(crate) fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    lookup[b'-' as usize] = 62; // URL-safe variant
    lookup[b'_' as usize] = 63; // URL-safe variant

    let input = input.trim_end_matches('=').as_bytes();
    if input.len() % 4 == 1 {
        return Err(Error::jwt("Invalid base64 length"));
    }

    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    for chunk in input.chunks(4) {
        let mut n: usize = 0;
        for &byte in chunk {
            let value = lookup[byte as usize];
            if value == 255 {
                return Err(Error::jwt("Invalid base64 character"));
            }
            n = (n << 6) | value as usize;
        }
        n <<= 6 * (4 - chunk.len());

        result.push((n >> 16) as u8);
        if chunk.len() >= 3 {
            result.push((n >> 8) as u8);
        }
        if chunk.len() == 4 {
            result.push(n as u8);
        }
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
    fn test_base64_decode() {
        let decoded = base64_decode("aGVsbG8").unwrap();
        assert_eq!(decoded, b"hello");
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
    }

    #[test]
    fn test_base64_decode_rejects_invalid_input() {
        // A length of 4n + 1 is never valid base64.
        assert!(base64_decode("aGVsb").is_err());
        assert!(base64_decode("aGV$bG8").is_err());
        assert!(base64_decode("aGVsbG$").is_err());
        // Interior padding is rejected.
        assert!(base64_decode("aG=sbG8=").is_err());
    }

    #[test]
    fn test_pem_body_inverted_markers() {
        // END before BEGIN must error, not panic.
        let pem = "-----END EC PRIVATE KEY-----\nAAAA\n-----BEGIN EC PRIVATE KEY-----";
        assert!(detect_key_kind(pem).is_ok());
        let creds = Credentials::new("test-key", pem).unwrap();
        assert!(generate_ws_jwt(&creds).is_err());
    }

    #[test]
    fn test_generate_nonce() {
        let nonce = generate_nonce().unwrap();
        assert_eq!(nonce.len(), 32); // 16 bytes = 32 hex chars
    }

    #[test]
    fn test_detect_sec1_key() {
        let pem = "-----BEGIN EC PRIVATE KEY-----\nAAAA\n-----END EC PRIVATE KEY-----";
        assert_eq!(detect_key_kind(pem).unwrap(), KeyKind::EcdsaSec1);
    }

    #[test]
    fn test_detect_raw_ed25519_key() {
        // 32 byte seed as base64.
        let raw32 = base64_url_encode(&[7u8; 32]);
        assert_eq!(detect_key_kind(&raw32).unwrap(), KeyKind::Ed25519Raw);
        // 64 byte seed plus public key as base64.
        let raw64 = base64_url_encode(&[7u8; 64]);
        assert_eq!(detect_key_kind(&raw64).unwrap(), KeyKind::Ed25519Raw);
    }

    #[test]
    fn test_detect_invalid_key() {
        assert!(detect_key_kind("not a key !!!").is_err());
    }

    #[test]
    fn test_key_kind_algorithm() {
        assert_eq!(KeyKind::EcdsaSec1.algorithm(), "ES256");
        assert_eq!(KeyKind::Ed25519Raw.algorithm(), "EdDSA");
    }

    // Throwaway keys generated for tests only.
    const TEST_EC_SEC1: &str = "-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIFRQqrwlq7sCUJ56eM3bLnEQxtWNkOr9lA6oaQ/0sKfLoAoGCCqGSM49
AwEHoUQDQgAEat2hFxJwUbhH4oZp9z5rj7J6nU7FYt6pfE6Ei3gvMWAZIqJ8TdME
S5IRIotaS4KLpQhofOyNZ7i7rcCAipIZrw==
-----END EC PRIVATE KEY-----";

    const TEST_EC_PKCS8: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgVFCqvCWruwJQnnp4
zdsucRDG1Y2Q6v2UDqhpD/Swp8uhRANCAARq3aEXEnBRuEfihmn3PmuPsnqdTsVi
3ql8ToSLeC8xYBkionxN0wRLkhEii1pLgoulCGh87I1nuLutwICKkhmv
-----END PRIVATE KEY-----";

    const TEST_ED25519_PKCS8: &str = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIO3AE3FBIOkUlxJkqy4Ou+5/gSU6rZEJHXhsgAAkKQaC
-----END PRIVATE KEY-----";

    fn jwt_header_alg(jwt: &str) -> String {
        let header_b64 = jwt.split('.').next().unwrap();
        let header: serde_json::Value =
            serde_json::from_slice(&base64_decode(header_b64).unwrap()).unwrap();
        header["alg"].as_str().unwrap().to_string()
    }

    #[test]
    fn test_sign_es256_sec1_key() {
        let creds = Credentials::new("test-key", TEST_EC_SEC1).unwrap();
        let jwt = generate_jwt_with_host(&creds, "GET", "api.coinbase.com", "/x").unwrap();
        assert_eq!(jwt.split('.').count(), 3);
        assert_eq!(jwt_header_alg(&jwt), "ES256");
    }

    #[test]
    fn test_sign_es256_pkcs8_key() {
        let creds = Credentials::new("test-key", TEST_EC_PKCS8).unwrap();
        let jwt = generate_jwt_with_host(&creds, "GET", "api.coinbase.com", "/x").unwrap();
        assert_eq!(jwt_header_alg(&jwt), "ES256");
    }

    #[test]
    fn test_sign_eddsa_pkcs8_key() {
        let creds = Credentials::new("test-key", TEST_ED25519_PKCS8).unwrap();
        assert_eq!(creds.key_kind(), KeyKind::Ed25519Pkcs8);
        let jwt = generate_jwt_with_host(&creds, "GET", "api.coinbase.com", "/x").unwrap();
        assert_eq!(jwt.split('.').count(), 3);
        assert_eq!(jwt_header_alg(&jwt), "EdDSA");
    }

    #[test]
    fn test_sign_eddsa_raw_key() {
        let raw = base64_url_encode(&[7u8; 32]);
        let creds = Credentials::new("test-key", raw).unwrap();
        assert_eq!(creds.key_kind(), KeyKind::Ed25519Raw);
        let jwt = generate_ws_jwt(&creds).unwrap();
        assert_eq!(jwt_header_alg(&jwt), "EdDSA");
    }
}
