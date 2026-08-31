//! Blocking wrapper around [`AS2ExpertClient`], behind the `blocking` feature.
//!
//! It owns a private current-thread Tokio runtime and drives the async client to
//! completion on each call, so you can use the API from synchronous code.

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::client::{environment_url, AS2ExpertClient};
use crate::error::{Error, Result};

/// Synchronous client. Cheap to clone (shares the runtime and async client).
#[derive(Clone)]
pub struct BlockingClient {
    inner: AS2ExpertClient,
    rt: Arc<tokio::runtime::Runtime>,
}

/// Builder for [`BlockingClient`].
pub struct BlockingBuilder {
    token: String,
    base_url: Option<String>,
    timeout: Duration,
    max_retries: u32,
    user_agent: Option<String>,
}

impl BlockingBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }
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
    pub fn build(self) -> Result<BlockingClient> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::transport(format!("failed to start runtime: {e}")))?;
        let mut b = AS2ExpertClient::builder(self.token)
            .timeout(self.timeout)
            .max_retries(self.max_retries);
        if let Some(u) = self.base_url {
            b = b.base_url(u);
        }
        if let Some(ua) = self.user_agent {
            b = b.user_agent(ua);
        }
        Ok(BlockingClient {
            inner: b.build()?,
            rt: Arc::new(rt),
        })
    }
}

impl BlockingClient {
    pub fn builder(token: impl Into<String>) -> BlockingBuilder {
        BlockingBuilder {
            token: token.into(),
            base_url: None,
            timeout: Duration::from_secs(crate::http::DEFAULT_TIMEOUT_SECS),
            max_retries: 2,
            user_agent: None,
        }
    }

    pub fn base_url(&self) -> &str {
        self.inner.base_url()
    }

    fn block<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }

    // --- Messages ---
    pub fn messages_list(&self, filter: Value) -> Result<Vec<Value>> {
        self.block(self.inner.messages.list(filter))
    }
    pub fn messages_get(&self, id: impl Into<Value>) -> Result<Value> {
        self.block(self.inner.messages.get(id))
    }
    pub fn messages_download(&self, id: impl Into<Value>) -> Result<Vec<u8>> {
        self.block(self.inner.messages.download(id))
    }
    pub fn messages_send(
        &self,
        partner: impl Into<Value>,
        subject: impl Into<String>,
        file_name: impl Into<String>,
        content: &[u8],
    ) -> Result<Value> {
        self.block(
            self.inner
                .messages
                .send(partner, subject, file_name, content),
        )
    }

    // --- Partners / Certificates / Stations ---
    pub fn partners_list(&self, filter: Value) -> Result<Vec<Value>> {
        self.block(self.inner.partners.list(filter))
    }
    pub fn partners_create(&self, partner: Value) -> Result<Value> {
        self.block(self.inner.partners.create(partner))
    }
    pub fn certificates_list(&self) -> Result<Vec<Value>> {
        self.block(self.inner.certificates.list())
    }
    pub fn stations_list(&self, filter: Value) -> Result<Vec<Value>> {
        self.block(self.inner.stations.list(filter))
    }

    // --- EDIFACT ---
    pub fn edifact_analyze(&self, edifact: impl Into<String>) -> Result<Value> {
        self.block(self.inner.edifact.analyze(edifact))
    }
    pub fn edifact_convert(&self, edifact: impl Into<String>, format: &str) -> Result<Value> {
        self.block(self.inner.edifact.convert(edifact, format))
    }
    pub fn edifact_acknowledge(&self, edifact: impl Into<String>) -> Result<Value> {
        self.block(self.inner.edifact.acknowledge(edifact))
    }

    /// Escape hatch: run any async call against the wrapped async client.
    pub fn run<F, T>(&self, f: impl FnOnce(&AS2ExpertClient) -> F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.block(f(&self.inner))
    }
}
