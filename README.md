# as2expert (Rust)

Official Rust client for the **AS2Expert** REST API — send and receive AS2/EDI
messages, manage trading partners, certificates and stations, drive Business
Documents, and validate/convert **EDIFACT**.

- Asynchronous by default (`reqwest` + Tokio); optional `blocking` wrapper.
- Typed errors, automatic retries on `429`/`5xx`, HMAC webhook verification.
- Configurable host: `free`, `b2b`, or any self-hosted deployment.

```toml
[dependencies]
as2expert = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

## Quick start (async)

```rust
use as2expert::AS2ExpertClient;
use serde_json::json;

#[tokio::main]
async fn main() -> as2expert::Result<()> {
    let client = AS2ExpertClient::builder("YOUR_TOKEN")
        .environment("free")            // or .base_url("https://your-host/api/v1")
        .build()?;

    // Send an EDI file to a partner (bytes are base64-encoded for you)
    client.messages
        .send("140", "Order 4711", "order.edi", b"UNB+...'")
        .await?;

    // List and download inbound messages
    for msg in client.messages.list(json!({ "limit": 20 })).await? {
        println!("{} {}", msg["id"], msg["asunto"]);
        let bytes = client.messages.download(&msg["id"]).await?;
        let _ = bytes;
    }
    Ok(())
}
```

Every method maps to a single POST call (the API is POST-only) and returns a
[`serde_json::Value`]; list methods return `Vec<Value>`.

## Blocking client

Enable the `blocking` feature for synchronous code (it owns a private Tokio
runtime):

```toml
as2expert = { version = "0.1", features = ["blocking"] }
```

```rust
use as2expert::blocking::BlockingClient;

let client = BlockingClient::builder("YOUR_TOKEN").environment("free").build()?;
let out = client.edifact_convert("UNB+...'", "json")?;
println!("{}", out["filename"]);
# Ok::<(), as2expert::Error>(())
```

## EDIFACT

```rust
# async fn run(client: as2expert::AS2ExpertClient) -> as2expert::Result<()> {
// Parse + validate + translate to JSON ("xml" / "text" also supported)
let out = client.edifact.convert(raw_edi, "json").await?;
println!("{} {}", out["filename"], out["content"]);

// Build a functional acknowledgement (CONTRL / APERAK)
let ack = client.edifact.acknowledge(raw_edi).await?;
println!("{} {}", ack["kind"], ack["control_reference"]);
# Ok(()) }
# fn raw_edi() {}
```

## Errors

Every call returns `Result<T, as2expert::Error>`. Inspect `Error::kind`
([`ErrorKind`]) to branch:

| `ErrorKind` | When |
|-------------|------|
| `Auth` | `401` / `403` |
| `Validation` | `400` / `422` (see `Error::fields`) |
| `NotFound` | `404` |
| `RateLimit` | `429` (see `Error::retry_after`) |
| `Server` | `5xx` |
| `Transport` | network/timeout, no HTTP status |

## Webhooks

AS2Expert signs deliveries with HMAC-SHA256 over `"<timestamp>.<body>"`, sent in
`X-AS2Expert-Timestamp` and `X-AS2Expert-Signature: sha256=<hex>`:

```rust
use as2expert::{verify_signature, default_tolerance};

let ok = verify_signature(
    secret,
    timestamp,   // X-AS2Expert-Timestamp
    body,        // the exact raw request body
    signature,   // X-AS2Expert-Signature
    default_tolerance(),
    now_unix_secs,
);
# fn f(secret: &str, timestamp: &str, body: &str, signature: &str, now_unix_secs: i64) { let _ = as2expert::verify_signature(secret, timestamp, body, signature, 300, now_unix_secs); }
```

## Configuration

`AS2ExpertClient::builder(token)` returns a builder:

- `.environment("free" | "b2b")` **or** `.base_url("https://your-host/api/v1")`
- `.timeout(Duration)`, `.max_retries(u32)`, `.user_agent("...")`

TLS backend features: `rustls` (default) or `native-tls`.

## Development

```bash
cargo test                       # unit tests (webhook HMAC), no network
AS2EXPERT_TOKEN=... cargo run --example smoke   # E2E against free
```

## License

Apache-2.0 — see [LICENSE](LICENSE).
