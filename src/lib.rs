//! # as2expert
//!
//! Official Rust client for the **AS2Expert** REST API — send and receive AS2/EDI
//! messages, manage trading partners, certificates and stations, drive Business
//! Documents, and validate/convert **EDIFACT**.
//!
//! The default client is asynchronous ([`AS2ExpertClient`]); enable the
//! `blocking` feature for a synchronous wrapper ([`blocking::BlockingClient`]).
//!
//! ```no_run
//! # async fn run() -> as2expert::Result<()> {
//! use as2expert::AS2ExpertClient;
//! use serde_json::json;
//!
//! let client = AS2ExpertClient::builder("YOUR_TOKEN")
//!     .environment("free") // or .base_url("https://your-host/api/v1")
//!     .build()?;
//!
//! client
//!     .messages
//!     .send("140", "Order 4711", "order.edi", b"UNB+...'")
//!     .await?;
//!
//! for msg in client.messages.list(json!({ "limit": 20 })).await? {
//!     println!("{} {}", msg["id"], msg["asunto"]);
//! }
//! # Ok(()) }
//! ```
//!
//! Every method maps to a single POST call (the API is POST-only) and returns a
//! [`serde_json::Value`]; list methods return `Vec<Value>`.

// `Error` deliberately carries rich diagnostics (the parsed payload, validation
// fields, retry hints). Boxing every `Result` to shave bytes off a rare,
// I/O-bound error path is not worth the ergonomic cost here.
#![allow(clippy::result_large_err)]

mod client;
mod error;
mod http;
mod resources;
mod webhooks_verify;

pub use client::{environment_url, AS2ExpertClient, ClientBuilder};
pub use error::{Error, ErrorKind, Result};
pub use resources::{
    BusinessDocuments, Certificates, Dashboard, Edifact, Messages, Partners, Stations, Webhooks,
};
pub use webhooks_verify::{default_tolerance, sign_payload, verify_signature};

#[cfg(feature = "blocking")]
pub mod blocking;
