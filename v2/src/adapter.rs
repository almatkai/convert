use crate::engine::{self, Row};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tezbar_extension_sdk::{BoxFuture, CoreResult, Extension, ExtensionContext};
use uuid::Uuid;

#[derive(Clone)]
struct Token {
    id: String,
    row: usize,
    bytes: String,
    retry: bool,
}
#[derive(Default)]
struct Model {
    generation: String,
    query: String,
    preference: Option<Value>,
    preference_error: Option<String>,
    rows: Vec<Row>,
    tokens: Vec<Token>,
    revision: u64,
    client: Option<u64>,
    client_floor: u64,
    message: String,
    uncertain: Option<(usize, String)>,
    pending: bool,
}
type Models = Arc<Mutex<HashMap<String, Arc<Mutex<Model>>>>>;
#[derive(Default)]
pub struct Converter {
    models: Models,
}
struct SessionGuard {
    models: Models,
    id: String,
    model: Arc<Mutex<Model>>,
}
impl Drop for SessionGuard {
    fn drop(&mut self) {
        let mut map = self.models.lock().unwrap();
        if map
            .get(&self.id)
            .is_some_and(|m| Arc::ptr_eq(m, &self.model))
        {
            map.remove(&self.id);
        }
    }
}
struct DeliveryGuard {
    model: Arc<Mutex<Model>>,
    revision: u64,
    row: usize,
    bytes: String,
    finished: bool,
}
impl Drop for DeliveryGuard {
    fn drop(&mut self) {
        if !self.finished {
            let mut m = self.model.lock().unwrap();
            if m.revision == self.revision {
                m.pending = false;
                m.uncertain = Some((self.row, self.bytes.clone()));
                m.message =
                    "Copy result is unknown. Retry sends exactly the displayed bytes.".into();
                m.renew();
            }
        }
    }
}
impl Model {
    fn renew(&mut self) {
        self.tokens.clear();
        if self.pending {
            return;
        }
        if let Some((row, bytes)) = &self.uncertain {
            self.tokens.push(Token {
                id: Uuid::new_v4().to_string(),
                row: *row,
                bytes: bytes.clone(),
                retry: true,
            });
        } else {
            for (i, r) in self.rows.iter().enumerate() {
                self.tokens.push(Token {
                    id: Uuid::new_v4().to_string(),
                    row: i,
                    bytes: r.copy.clone(),
                    retry: false,
                });
            }
        }
    }
    fn render(&self) -> Value {
        let mut result=tezbar_extension_sdk::view("Convert Unit",json!({"type":"List","props":{"searchBarPlaceholder":"Type your unit here... (eg.: 22px or #006699)","searchText":self.query,"enableFiltering":false,"onSearchTextChange":true,"__hasServerSearch":true,"nativeModelRevision":self.revision,"clientInputRevision":self.client,"nativeSessionGeneration":self.generation},"children":[{"type":"List.Section","children":self.rows.iter().enumerate().map(|(i,r)|json!({"type":"List.Item","props":{"id":format!("result-{i}"),"title":r.title,"subtitle":r.subtitle,"accessories":[{"text":r.accessory}],"icon":r.swatch.as_ref().map(|s|json!({"source":"CircleFilled","tintColor":{"light":s,"dark":s,"adjustContrast":false}})),"actionIds":self.tokens.iter().filter(|t|t.row==i).map(|t|t.id.clone()).collect::<Vec<_>>()}})).collect::<Vec<_>>()}]}),self.tokens.iter().map(|t|json!({"id":t.id,"kind":"action","title":if t.retry {"Retry Copy Exact Value"}else{"Copy to Clipboard"}})).collect());
        result["refreshMs"] = Value::Null;
        if !self.message.is_empty() {
            result["message"] = json!(self.message);
        }
        result
    }
    fn revoke(&mut self) {
        self.revision += 1;
        self.tokens.clear();
        self.rows.clear();
        self.uncertain = None;
        self.pending = false;
        self.message.clear();
        self.client = None;
    }
}
fn bounded_preferences(p: &Value) -> Result<Option<Value>, String> {
    let v = p.get("basePixel").cloned();
    if v.as_ref().is_some_and(|v| v.to_string().len() > 1024) {
        return Err("Base PX Value exceeds 1024 bytes".into());
    }
    Ok(v)
}
fn ack(v: &Value) -> bool {
    v.as_object().is_some_and(|o| {
        o.get("ok") == Some(&Value::Bool(true))
            && o.keys().all(|k| k == "ok" || k == "message")
            && o.get("message").is_none_or(Value::is_string)
    })
}
impl Extension for Converter {
    fn id(&self) -> &'static str {
        "com.tezbar.native.convert"
    }
    fn version(&self) -> &'static str {
        "0.1.0"
    }
    fn commands(&self) -> &'static [&'static str] {
        &["convert"]
    }
    fn shutdown<'a>(&'a self, _: &'a ExtensionContext) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            self.models.lock().unwrap().clear();
            Ok(json!({}))
        })
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        _: &'a str,
        op: &'a str,
        p: &'a Value,
        _state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            let sid = p["sessionId"].as_str().ok_or("Missing session")?;
            if sid.len() > 256 {
                return Err("Session ID exceeds 256 bytes".into());
            }
            if op == "extension:dispose-session" {
                return Ok(tezbar_extension_sdk::unchanged());
            } // Scope guard owns exact generation cleanup.
            if op == "extension:run-command" {
                let model = Arc::new(Mutex::new(Model {
                    generation: Uuid::new_v4().to_string(),
                    ..Model::default()
                }));
                {
                    let mut map = self.models.lock().unwrap();
                    if map.len() >= 128 {
                        return Err("Too many converter sessions".into());
                    }
                    if map.contains_key(sid) {
                        return Err("Converter session already exists".into());
                    }
                    map.insert(sid.into(), model.clone());
                }
                let guard = SessionGuard {
                    models: self.models.clone(),
                    id: sid.into(),
                    model: model.clone(),
                };
                ctx.spawn(async move {
                    let _guard = guard;
                    std::future::pending::<()>().await;
                })?;
                let mut m = model.lock().unwrap();
                match bounded_preferences(&p["preferences"]) {
                    Ok(v) => m.preference = v,
                    Err(e) => {
                        m.preference_error = Some(e.clone());
                        m.message = e;
                    }
                };
                return Ok(m.render());
            }
            let model = self
                .models
                .lock()
                .unwrap()
                .get(sid)
                .cloned()
                .ok_or("Session is no longer available")?;
            if op == "extension:search-text-changed" {
                let mut m = model.lock().unwrap();
                let incoming_client = p
                    .get("clientInputRevision")
                    .and_then(Value::as_u64)
                    .filter(|n| *n <= 9_007_199_254_740_991);
                // An old complete input is inert. Do not let crossed arrivals
                // revoke a newer model or consume its current action tokens.
                if incoming_client.is_some_and(|revision| revision <= m.client_floor) {
                    return Ok(tezbar_extension_sdk::unchanged());
                }
                m.revoke(); // External ownership, before validation; never restored by staged JSON rollback.
                if p.get("clientInputRevision").is_some() {
                    let Some(revision) = incoming_client else {
                        m.message = "Input revision is invalid.".into();
                        return Ok(m.render());
                    };
                    // A newer invalid text draft still owns its revision.
                    m.client_floor = revision;
                    m.client = Some(revision);
                }
                let Some(q) = p["searchText"].as_str() else {
                    m.message = "Enter a text value.".into();
                    return Ok(m.render());
                };
                if q.len() > 4096 {
                    m.message = "Input exceeds 4096 bytes.".into();
                    return Ok(m.render());
                }
                m.query = q.into();
                if let Some(error) = m.preference_error.clone() {
                    m.message = error;
                    return Ok(m.render());
                }
                m.rows = engine::convert(&m.query, m.preference.as_ref());
                if m.rows
                    .iter()
                    .any(|r| r.title.len() > 8192 || r.copy.len() > 8192)
                {
                    m.rows.clear();
                    m.message = "Converted value exceeds 8192 bytes.".into();
                } else {
                    m.renew();
                }
                return Ok(m.render());
            }
            if matches!(op, "extension:refresh-session" | "extension:load-more") {
                return Ok(model.lock().unwrap().render());
            }
            if op != "extension:invoke-action" {
                return Err("Unsupported converter operation".into());
            }
            let token = {
                let mut m = model.lock().unwrap();
                let Some(i) = m
                    .tokens
                    .iter()
                    .position(|t| Some(t.id.as_str()) == p["actionId"].as_str())
                else {
                    return Ok(tezbar_extension_sdk::unchanged());
                };
                let token = m.tokens.remove(i); // Consume before await; not part of SDK staged JSON state.
                m.tokens.clear();
                m.pending = true;
                m.message.clear();
                token
            };
            let revision = model.lock().unwrap().revision;
            let mut guard = DeliveryGuard {
                model: model.clone(),
                revision,
                row: token.row,
                bytes: token.bytes.clone(),
                finished: false,
            };
            let response = tokio::time::timeout(
                Duration::from_secs(5),
                ctx.host("host.clipboard.write", json!({"text":token.bytes})),
            )
            .await;
            // The own pinned V1 producer stays open after CopyToClipboard. No visibility RPC.
            let mut m = model.lock().unwrap();
            guard.finished = true;
            if m.revision != revision {
                return Ok(tezbar_extension_sdk::unchanged());
            }
            m.pending = false;
            if matches!(response,Ok(Ok(ref v)) if ack(v)) {
                m.uncertain = None;
                m.message = "Copied to clipboard".into();
            } else {
                m.uncertain = Some((token.row, token.bytes));
                m.message =
                    "Copy result is unknown. Retry sends exactly the displayed bytes.".into();
            }
            m.renew();
            Ok(m.render())
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn producer_ack() {
        assert!(ack(&json!({"ok":true})));
        assert!(ack(&json!({"ok":true,"message":"Copied"})));
        for v in [
            json!(null),
            json!({"ok":false}),
            json!({"ok":true,"other":true}),
            json!({"ok":true,"message":null}),
        ] {
            assert!(!ack(&v));
        }
    }
}
