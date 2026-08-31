//! Low-level async transport: one POST per API call, with retries on 429/5xx.

use std::time::Duration;

use serde_json::Value;

use crate::error::{Error, Result};

pub(crate) const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub(crate) const USER_AGENT: &str = concat!("as2expert-rust/", env!("CARGO_PKG_VERSION"));

/// Shared, cheap-to-clone transport around a [`reqwest::Client`].
#[derive(Clone)]
pub(crate) struct Transport {
    http: reqwest::Client,
    base_url: String,
    token: String,
    max_retries: u32,
}

impl Transport {
    pub(crate) fn new(
        token: String,
        base_url: String,
        timeout: Duration,
        max_retries: u32,
        user_agent: Option<String>,
    ) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent(user_agent.unwrap_or_else(|| USER_AGENT.to_string()))
            .build()
            .map_err(|e| Error::transport(e.to_string()))?;
        Ok(Transport {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            max_retries,
        })
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    /// POST `body` to `path` (e.g. `"/messages/send"`), returning the decoded
    /// `data` field (or the whole payload when there is no `data`).
    pub(crate) async fn post(&self, path: &str, body: Value) -> Result<Value> {
        self.post_with_headers(path, body, &[]).await
    }

    pub(crate) async fn post_with_headers(
        &self,
        path: &str,
        body: Value,
        extra: &[(&str, String)],
    ) -> Result<Value> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let mut attempt: u32 = 0;
        loop {
            let mut req = self.http.post(&url).bearer_auth(&self.token).json(&body);
            for (k, v) in extra {
                req = req.header(*k, v);
            }
            let resp = req.send().await;
            match resp {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let text = r.text().await.unwrap_or_default();
                    let payload: Option<Value> = serde_json::from_str(&text).ok();
                    if (200..300).contains(&status) {
                        return Ok(unwrap_data(payload));
                    }
                    if should_retry(status) && attempt < self.max_retries {
                        attempt += 1;
                        backoff(attempt).await;
                        continue;
                    }
                    return Err(Error::from_status(
                        status,
                        message_of(&payload, &text),
                        payload,
                    ));
                }
                Err(e) => {
                    if attempt < self.max_retries && (e.is_timeout() || e.is_connect()) {
                        attempt += 1;
                        backoff(attempt).await;
                        continue;
                    }
                    return Err(Error::transport(e.to_string()));
                }
            }
        }
    }
}

fn unwrap_data(payload: Option<Value>) -> Value {
    match payload {
        Some(Value::Object(mut o)) => o.remove("data").unwrap_or(Value::Object(o)),
        Some(other) => other,
        None => Value::Null,
    }
}

fn message_of(payload: &Option<Value>, fallback: &str) -> String {
    payload
        .as_ref()
        .and_then(|v| v.as_object())
        .and_then(|o| {
            o.get("msg")
                .or_else(|| o.get("message"))
                .or_else(|| o.get("error"))
        })
        .and_then(|v| v.as_str())
        .map(String::from)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if fallback.is_empty() {
                "request failed".to_string()
            } else {
                fallback.chars().take(300).collect()
            }
        })
}

fn should_retry(status: u16) -> bool {
    status == 429 || status >= 500
}

async fn backoff(attempt: u32) {
    // Exponential: 200ms, 400ms, 800ms, ... capped at 4s.
    let shift = attempt.clamp(1, 5) - 1;
    let ms = 200u64.saturating_mul(1u64 << shift);
    tokio::time::sleep(Duration::from_millis(ms.min(4000))).await;
}
