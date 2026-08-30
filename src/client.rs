//! The asynchronous AS2Expert client.

use std::time::Duration;

use crate::error::Result;
use crate::http::{Transport, DEFAULT_TIMEOUT_SECS};
use crate::resources::{
    BusinessDocuments, Certificates, Dashboard, Edifact, Messages, Partners, Stations, Webhooks,
};

/// Convenience host presets. Pass an explicit base URL to target any other host.
pub fn environment_url(name: &str) -> Option<&'static str> {
    match name {
        "free" => Some("https://free.as2expert.com/api/v1"),
        "b2b" => Some("https://b2b.as2expert.com/api/v1"),
        _ => None,
    }
}

/// Builder for [`AS2ExpertClient`].
pub struct ClientBuilder {
    token: String,
    base_url: Option<String>,
    timeout: Duration,
    max_retries: u32,
    user_agent: Option<String>,
}

impl ClientBuilder {
    fn new(token: impl Into<String>) -> Self {
        ClientBuilder {
            token: token.into(),
            base_url: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: 2,
            user_agent: None,
        }
    }

    /// Target an explicit API base URL (e.g. a self-hosted deployment).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Target a named environment (`"free"` or `"b2b"`).
    pub fn environment(mut self, name: &str) -> Self {
        if let Some(u) = environment_url(name) {
            self.base_url = Some(u.to_string());
        }
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Build the client. Fails if no base URL/environment was set or the
    /// underlying HTTP client cannot be constructed.
    pub fn build(self) -> Result<AS2ExpertClient> {
        let base_url = self.base_url.ok_or_else(|| {
            crate::error::Error::transport(
                "set base_url(..) or environment(\"free\"|\"b2b\") before build()",
            )
        })?;
        let t = Transport::new(
            self.token,
            base_url,
            self.timeout,
            self.max_retries,
            self.user_agent,
        )?;
        Ok(AS2ExpertClient {
            messages: Messages::new(t.clone()),
            partners: Partners::new(t.clone()),
            certificates: Certificates::new(t.clone()),
            stations: Stations::new(t.clone()),
            webhooks: Webhooks::new(t.clone()),
            business_documents: BusinessDocuments::new(t.clone()),
            edifact: Edifact::new(t.clone()),
            dashboard: Dashboard::new(t.clone()),
            transport: t,
        })
    }
}

/// Asynchronous client for the AS2Expert REST API.
///
/// ```no_run
/// # async fn run() -> as2expert::Result<()> {
/// use as2expert::AS2ExpertClient;
/// let client = AS2ExpertClient::builder("TOKEN").environment("free").build()?;
/// let out = client.edifact.convert("UNB+...'", "json").await?;
/// println!("{}", out["filename"]);
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct AS2ExpertClient {
    pub messages: Messages,
    pub partners: Partners,
    pub certificates: Certificates,
    pub stations: Stations,
    pub webhooks: Webhooks,
    pub business_documents: BusinessDocuments,
    pub edifact: Edifact,
    pub dashboard: Dashboard,
    transport: Transport,
}

impl AS2ExpertClient {
    /// Start building a client with the given API token.
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(token)
    }

    /// The base URL this client targets.
    pub fn base_url(&self) -> &str {
        self.transport.base_url()
    }
}
