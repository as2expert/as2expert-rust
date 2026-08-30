//! Verify inbound AS2Expert webhook signatures.
//!
//! AS2Expert signs deliveries with HMAC-SHA256 over `"<timestamp>.<body>"`, sent
//! in the headers `X-AS2Expert-Timestamp` and `X-AS2Expert-Signature`
//! (`sha256=<hex>`).

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const MAX_SKEW_SECS: i64 = 300;

/// Compute the `sha256=<hex>` signature for a `timestamp` + `body`.
pub fn sign_payload(secret: &str, timestamp: &str, body: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(format!("{timestamp}.{body}").as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

/// Verify a webhook signature and freshness.
///
/// `now` is the current unix time in seconds (pass your clock; injectable for
/// tests). Returns `true` only if the signature matches and `timestamp` is
/// within `tolerance` seconds of `now`.
pub fn verify_signature(
    secret: &str,
    timestamp: &str,
    body: &str,
    signature: &str,
    tolerance_secs: i64,
    now: i64,
) -> bool {
    if secret.is_empty() || signature.is_empty() {
        return false;
    }
    let ts: i64 = match timestamp.trim().parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    if (now - ts).abs() > tolerance_secs {
        return false;
    }
    let expected = sign_payload(secret, timestamp, body);
    let provided = if let Some(rest) = signature.strip_prefix("sha256=") {
        format!("sha256={rest}")
    } else {
        format!("sha256={signature}")
    };
    constant_time_eq(expected.as_bytes(), provided.as_bytes())
}

/// Default tolerance window in seconds (5 minutes).
pub const fn default_tolerance() -> i64 {
    MAX_SKEW_SECS
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
