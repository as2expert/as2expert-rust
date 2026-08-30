//! Typed errors returned by the AS2Expert client.

use serde_json::Value;

/// The kind of an [`Error`], mirroring the HTTP status classes the API returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// 401 / 403 — missing/invalid token or insufficient scope.
    Auth,
    /// 400 / 422 — server-side validation failed (see [`Error::fields`]).
    Validation,
    /// 404 — the resource does not exist.
    NotFound,
    /// 429 — too many requests (see [`Error::retry_after`]).
    RateLimit,
    /// 5xx — the API failed.
    Server,
    /// The request never completed (connection / timeout / TLS / decode).
    Transport,
    /// Any other non-success status.
    Api,
}

/// An error from an AS2Expert API call.
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    /// HTTP status code, when the request reached the server.
    pub status: Option<u16>,
    /// Application error code from the payload, when present.
    pub code: Option<String>,
    /// Field-level validation errors (`kind == Validation`).
    pub fields: Vec<Value>,
    /// Seconds to wait before retrying (`kind == RateLimit`), when known.
    pub retry_after: Option<f64>,
    /// The raw error payload, when the body parsed as JSON.
    pub payload: Option<Value>,
}

impl Error {
    pub(crate) fn transport(msg: impl Into<String>) -> Self {
        Error {
            kind: ErrorKind::Transport,
            message: msg.into(),
            status: None,
            code: None,
            fields: Vec::new(),
            retry_after: None,
            payload: None,
        }
    }

    /// Build the appropriate error for an HTTP status + parsed payload.
    pub(crate) fn from_status(status: u16, message: String, payload: Option<Value>) -> Self {
        let kind = match status {
            401 | 403 => ErrorKind::Auth,
            400 | 422 => ErrorKind::Validation,
            404 => ErrorKind::NotFound,
            429 => ErrorKind::RateLimit,
            s if s >= 500 => ErrorKind::Server,
            _ => ErrorKind::Api,
        };
        let obj = payload.as_ref().and_then(|v| v.as_object());
        let code = obj
            .and_then(|o| o.get("code"))
            .and_then(|v| v.as_str())
            .map(String::from);
        let fields = obj
            .and_then(|o| o.get("fields"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let retry_after = obj
            .and_then(|o| o.get("retry_after"))
            .and_then(|v| v.as_f64());
        Error {
            kind,
            message,
            status: Some(status),
            code,
            fields,
            retry_after,
            payload,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status {
            Some(s) => write!(f, "AS2Expert API error ({}): {}", s, self.message),
            None => write!(f, "AS2Expert transport error: {}", self.message),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, Error>;
