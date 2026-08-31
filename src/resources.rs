//! Resource namespaces. Each method maps to a single POST endpoint and returns
//! the decoded `data` value (`serde_json::Value`). List methods return the
//! `data` array as a `Vec<Value>`.

use base64::Engine;
use serde_json::{json, Value};

use crate::error::{Error, Result};
use crate::http::Transport;

fn as_array(v: Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a,
        Value::Null => Vec::new(),
        other => vec![other],
    }
}

fn decode_b64_field(data: &Value) -> Result<Vec<u8>> {
    let b64 = data
        .get("content_b64")
        .or_else(|| data.get("contenido_base64"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::transport("response had no content_b64"))?;
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| Error::transport(format!("bad base64: {e}")))
}

macro_rules! namespace {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            t: Transport,
        }
        impl $name {
            pub(crate) fn new(t: Transport) -> Self {
                Self { t }
            }
        }
    };
}

namespace!(Messages);
namespace!(Partners);
namespace!(Certificates);
namespace!(Stations);
namespace!(Webhooks);
namespace!(BusinessDocuments);
namespace!(Edifact);
namespace!(Dashboard);

impl Messages {
    /// List messages. `filter` is merged into the request body (e.g. `station`,
    /// `folder`, `limit`). Pass `serde_json::json!({})` for no filter.
    pub async fn list(&self, filter: Value) -> Result<Vec<Value>> {
        Ok(as_array(self.t.post("/messages", filter).await?))
    }

    /// List a station's folders. `filter` may carry `station`; omit it for every
    /// folder on the site. Each folder has `id`, `name`, `parent_id`, `count`,
    /// `icono`, `especial`, and `station_id`/`station_name`.
    pub async fn folders(&self, filter: Value) -> Result<Vec<Value>> {
        Ok(as_array(self.t.post("/messages/folders", filter).await?))
    }

    pub async fn get(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/messages/detail", json!({ "id": id.into() }))
            .await
    }

    /// Download the message payload, returning the raw decoded bytes.
    pub async fn download(&self, id: impl Into<Value>) -> Result<Vec<u8>> {
        let data = self
            .t
            .post("/messages/download", json!({ "id": id.into() }))
            .await?;
        decode_b64_field(&data)
    }

    /// Send a file to a trading partner. `content` is base64-encoded for you.
    pub async fn send(
        &self,
        partner: impl Into<Value>,
        subject: impl Into<String>,
        file_name: impl Into<String>,
        content: &[u8],
    ) -> Result<Value> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(content);
        self.t
            .post(
                "/messages/send",
                json!({
                    "partner": partner.into(),
                    "subject": subject.into(),
                    "file_name": file_name.into(),
                    "file_content": b64,
                }),
            )
            .await
    }

    pub async fn mark_read(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/messages/mark-read", json!({ "id": id.into() }))
            .await
    }

    pub async fn mark_unread(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/messages/mark-unread", json!({ "id": id.into() }))
            .await
    }

    pub async fn move_to(&self, id: impl Into<Value>, folder: impl Into<Value>) -> Result<Value> {
        self.t
            .post(
                "/messages/move",
                json!({ "id": id.into(), "folder": folder.into() }),
            )
            .await
    }

    pub async fn delete(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/messages/delete", json!({ "id": id.into() }))
            .await
    }

    /// Incremental changes; `params` is merged into the request body.
    pub async fn changes(&self, params: Value) -> Result<Value> {
        self.t.post("/messages/changes", params).await
    }

    pub async fn files(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/messages/files", json!({ "id": id.into() }))
            .await
    }

    pub async fn file_download(
        &self,
        id: impl Into<Value>,
        file_id: impl Into<Value>,
    ) -> Result<Vec<u8>> {
        let data = self
            .t
            .post(
                "/messages/file-download",
                json!({ "id": id.into(), "file_id": file_id.into() }),
            )
            .await?;
        decode_b64_field(&data)
    }

    pub async fn export(&self, params: Value) -> Result<Value> {
        self.t.post("/messages/export", params).await
    }
}

impl Partners {
    pub async fn list(&self, filter: Value) -> Result<Vec<Value>> {
        Ok(as_array(self.t.post("/partners", filter).await?))
    }
    pub async fn get(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/partners/detail", json!({ "id": id.into() }))
            .await
    }
    pub async fn create(&self, partner: Value) -> Result<Value> {
        self.t.post("/partners/create", partner).await
    }
}

impl Certificates {
    pub async fn list(&self) -> Result<Vec<Value>> {
        Ok(as_array(self.t.post("/certificates", json!({})).await?))
    }
    pub async fn get(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/certificates/detail", json!({ "id": id.into() }))
            .await
    }
    pub async fn create(&self, cert: Value) -> Result<Value> {
        self.t.post("/certificates/create", cert).await
    }
}

impl Stations {
    pub async fn list(&self, filter: Value) -> Result<Vec<Value>> {
        Ok(as_array(self.t.post("/stations", filter).await?))
    }
    pub async fn get(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/stations/detail", json!({ "id": id.into() }))
            .await
    }
    pub async fn stats(&self, id: impl Into<Value>) -> Result<Value> {
        self.t
            .post("/stations/stats", json!({ "id": id.into() }))
            .await
    }
    pub async fn create(&self, station: Value) -> Result<Value> {
        self.t.post("/stations/create", station).await
    }
}

impl Webhooks {
    pub async fn configure(&self, config: Value) -> Result<Value> {
        self.t.post("/webhooks/configure", config).await
    }
    pub async fn get(&self) -> Result<Value> {
        self.t.post("/webhooks/get", json!({})).await
    }
    pub async fn test(&self) -> Result<Value> {
        self.t.post("/webhooks/test", json!({})).await
    }
    pub async fn logs(&self, params: Value) -> Result<Value> {
        self.t.post("/webhooks/logs", params).await
    }
}

impl BusinessDocuments {
    /// Create a business document. Pass an `idempotency_key` to make retries safe.
    pub async fn create(&self, doc: Value, idempotency_key: Option<&str>) -> Result<Value> {
        let extra = idempotency_key
            .map(|k| vec![("Idempotency-Key", k.to_string())])
            .unwrap_or_default();
        self.t
            .post_with_headers("/business-documents", doc, &extra)
            .await
    }
    pub async fn get(&self, business_document_id: impl Into<Value>) -> Result<Value> {
        self.t
            .post(
                "/business-documents/detail",
                json!({ "business_document_id": business_document_id.into() }),
            )
            .await
    }
    pub async fn diagnostics(&self, params: Value) -> Result<Value> {
        self.t.post("/business-documents/diagnostics", params).await
    }
}

impl Edifact {
    /// Parse + validate an interchange (structure, codes, required elements).
    pub async fn analyze(&self, edifact: impl Into<String>) -> Result<Value> {
        self.t
            .post("/edifact/analyze", json!({ "edifact": edifact.into() }))
            .await
    }
    /// Alias of [`analyze`](Self::analyze) — the same endpoint.
    pub async fn validate(&self, edifact: impl Into<String>) -> Result<Value> {
        self.analyze(edifact).await
    }
    /// Translate an interchange to `"json"`, `"xml"`, or `"text"`.
    pub async fn convert(&self, edifact: impl Into<String>, format: &str) -> Result<Value> {
        self.t
            .post(
                "/edifact/convert",
                json!({ "edifact": edifact.into(), "format": format, "sequence": 1 }),
            )
            .await
    }
    /// Build a functional acknowledgement (`kind`: `"contrl"` or `"aperak"`).
    pub async fn acknowledge(&self, edifact: impl Into<String>) -> Result<Value> {
        self.acknowledge_with(edifact, "contrl", true, Value::Array(vec![]))
            .await
    }
    /// Acknowledge with explicit kind / acknowledged flag / error list.
    pub async fn acknowledge_with(
        &self,
        edifact: impl Into<String>,
        kind: &str,
        acknowledged: bool,
        errors: Value,
    ) -> Result<Value> {
        self.t
            .post(
                "/edifact/acknowledge",
                json!({
                    "edifact": edifact.into(),
                    "kind": kind,
                    "acknowledged": acknowledged,
                    "errors": errors,
                }),
            )
            .await
    }
    /// Build a minimal valid skeleton for a message type + release. Set
    /// `compose` to also serialize it to an interchange.
    pub async fn skeleton(
        &self,
        message_type: &str,
        release: &str,
        compose: bool,
    ) -> Result<Value> {
        self.t
            .post(
                "/edifact/skeleton",
                json!({ "message_type": message_type, "release": release, "compose": compose }),
            )
            .await
    }
}

impl Dashboard {
    pub async fn kpis(&self) -> Result<Value> {
        self.t.post("/dashboard/kpis", json!({})).await
    }
}
