use as2expert::{sign_payload, verify_signature};

#[test]
fn signature_roundtrip() {
    let secret = "secret-0123456789abcdef";
    let sig = sign_payload(secret, "1000", "{\"a\":1}");
    assert!(sig.starts_with("sha256="));
    // matches within tolerance
    assert!(verify_signature(
        secret,
        "1000",
        "{\"a\":1}",
        &sig,
        300,
        1000
    ));
    // accepts a bare hex signature (no prefix)
    let bare = sig.trim_start_matches("sha256=");
    assert!(verify_signature(
        secret,
        "1000",
        "{\"a\":1}",
        bare,
        300,
        1000
    ));
}

#[test]
fn rejects_stale_timestamp() {
    let secret = "secret-0123456789abcdef";
    let sig = sign_payload(secret, "1000", "{\"a\":1}");
    assert!(!verify_signature(
        secret,
        "1000",
        "{\"a\":1}",
        &sig,
        300,
        99999
    ));
}

#[test]
fn rejects_tampered_body() {
    let secret = "secret-0123456789abcdef";
    let sig = sign_payload(secret, "1000", "{\"a\":1}");
    assert!(!verify_signature(
        secret,
        "1000",
        "{\"a\":2}",
        &sig,
        300,
        1000
    ));
}

#[test]
fn rejects_bad_timestamp_and_empty() {
    let secret = "secret";
    let sig = sign_payload(secret, "1000", "body");
    assert!(!verify_signature(
        secret,
        "not-a-number",
        "body",
        &sig,
        300,
        1000
    ));
    assert!(!verify_signature("", "1000", "body", &sig, 300, 1000));
    assert!(!verify_signature(secret, "1000", "body", "", 300, 1000));
}
