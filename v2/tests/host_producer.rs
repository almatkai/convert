//! Complete pinned producer functions; non-converter branches trap immediately.
//! All reachable substitutions are OS/context boundaries. No actual desktop access.
// The immutable pinned producer triggers this lint only in Clippy 1.85.
#![allow(dead_code, clippy::nonminimal_bool)]
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    sync::{Arc, Mutex},
};
type CoreResult = Result<Value, String>;
mod arboard {
    #[derive(Debug)]
    pub enum Error {
        ContentNotAvailable,
        Other,
    }
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "unavailable")
        }
    }
}
struct Native;
impl Native {
    fn get_text(&mut self) -> Result<String, arboard::Error> {
        panic!("Unexpected clipboard read")
    }
}
mod native {
    pub fn write_files(_: &[String]) -> Result<(), String> {
        panic!("Unexpected files")
    }
    pub fn sequence() -> Option<u64> {
        panic!("Unexpected sequence")
    }
}
mod base64 {
    pub mod engine {
        pub mod general_purpose {
            pub static STANDARD: super::super::Coder = super::super::Coder;
        }
    }
    pub struct Coder;
    impl Coder {
        pub fn decode(&self, _: &str) -> Result<Vec<u8>, String> {
            panic!("Unexpected image")
        }
        pub fn encode(&self, _: &[u8]) -> String {
            panic!("Unexpected image")
        }
    }
}
mod image {
    pub enum ImageFormat {
        Png,
    }
    pub struct Image;
    pub fn load_from_memory(_: &[u8]) -> Result<Image, String> {
        panic!("Unexpected image")
    }
    impl Image {
        pub fn write_to(
            &self,
            _: &mut std::io::Cursor<Vec<u8>>,
            _: ImageFormat,
        ) -> Result<(), String> {
            panic!("Unexpected image")
        }
        pub fn width(&self) -> u32 {
            0
        }
        pub fn height(&self) -> u32 {
            0
        }
    }
}
struct Worker {
    written: Vec<String>,
    fail: bool,
    last_sequence: Option<u64>,
    last_id: String,
    ctx: Context,
}
fn text_arg(a: &[Value], i: usize) -> Result<&str, String> {
    a.get(i)
        .and_then(Value::as_str)
        .ok_or("Expected text".into())
}
fn text_entry(_: &str, _: u64) -> Option<Value> {
    panic!("Unexpected history")
}
fn now_ms() -> u64 {
    panic!("Unexpected history")
}
impl Worker {
    fn native(&mut self) -> Result<&mut Native, String> {
        panic!("Unexpected native read")
    }
    fn write_text(&mut self, text: String) -> Result<(), String> {
        self.written.push(text);
        if self.fail {
            Err("OS write failed".into())
        } else {
            Ok(())
        }
    }
    fn write_image(&mut self, _: &[u8]) -> Result<(), String> {
        panic!("Unexpected image")
    }
    fn cleanup(&mut self, _: bool) -> CoreResult {
        panic!("Unexpected history")
    }
    fn load(&mut self) -> Result<(), String> {
        panic!("Unexpected history")
    }
    fn insert(&mut self, _: Value) -> Result<(), String> {
        panic!("Unexpected history")
    }
    fn items(&self) -> &Vec<Value> {
        panic!("Unexpected history")
    }
    fn save_items(&mut self, _: Vec<Value>) -> Result<(), String> {
        panic!("Unexpected history")
    }
    fn remove_image(&mut self, _: &Value) {
        panic!("Unexpected history")
    }
}
include!("../qa/extracted/clipboard_handle.rs");
#[derive(Default)]
struct Context {
    calls: Mutex<Vec<(String, Vec<Value>)>>,
    worker: Mutex<Option<Box<Worker>>>,
}
impl Context {
    async fn request(&self, channel: &str, args: Vec<Value>) -> CoreResult {
        self.calls
            .lock()
            .unwrap()
            .push((channel.into(), args.clone()));
        match channel {
            "clipboard:write" => self
                .worker
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .handle(channel, &args),
            "core:platform:visibility" => Ok(Value::Null),
            _ => panic!("Unexpected OS call: {channel}"),
        }
    }
    fn emit(&self, _: &str, _: Value) {
        panic!("Unexpected event")
    }
}
mod process_install {
    pub fn component(_: &str) -> Result<(), String> {
        panic!("Unexpected background")
    }
}
mod launcher {
    use super::*;
    pub fn publish(_: &Arc<Process>, _: &Value) -> CoreResult {
        panic!("Unexpected launcher")
    }
}
mod process_notifications {
    pub async fn show(_: &str, _: &str, _: &str) -> Result<(), String> {
        panic!("Unexpected notification")
    }
    pub async fn remove(_: &str) -> Result<(), String> {
        panic!("Unexpected notification")
    }
}
fn background_update(_: &Context, _: &str, _: &str, _: Option<Value>) -> Result<(), String> {
    panic!("Unexpected persistence")
}
async fn read_clipboard_image() -> CoreResult {
    panic!("Unexpected image")
}
async fn write_files(_: &Value) -> CoreResult {
    panic!("Unexpected files")
}
async fn selected_files() -> CoreResult {
    panic!("Unexpected selection")
}
include!("../qa/extracted/string.rs");
#[derive(Default)]
struct Resources {
    leases: HashSet<String>,
    trays: HashMap<String, ()>,
    notifications: HashSet<String>,
}
struct Process {
    is_ready: bool,
    is_alive: bool,
    changed: tokio::sync::Notify,
}
impl Process {
    fn ready(&self) -> bool {
        self.is_ready
    }
    fn alive(&self) -> bool {
        self.is_alive
    }
}
mod producer {
    use super::*;
    pub(super) struct Host {
        pub ctx: Context,
        pub manifest: Value,
        pub resources: Mutex<Resources>,
        pub id: String,
    }
    impl Host {
        fn scoped(&self, _: &str) -> Result<String, String> {
            panic!("Unexpected resource")
        }
        async fn blur(&self, _: bool) -> CoreResult {
            panic!("Unexpected blur")
        }
        async fn set_tray(&self, _: &Arc<Process>, _: Value) -> Result<(), String> {
            panic!("Unexpected tray")
        }
    }
    include!("../qa/extracted/granted.rs");
    include!("../qa/extracted/require.rs");
    include!("../qa/extracted/request.rs");
}
#[tokio::test]
async fn full_producer_ack_and_arguments() {
    let worker = Worker {
        written: vec![],
        fail: false,
        last_sequence: None,
        last_id: String::new(),
        ctx: Context::default(),
    };
    let ctx = Context {
        worker: Mutex::new(Some(Box::new(worker))),
        ..Context::default()
    };
    let mut h = producer::Host {
        ctx,
        manifest: json!({"capabilities":["clipboard.write","window.visibility"]}),
        resources: Mutex::default(),
        id: "convert".into(),
    };
    let p = Arc::new(Process {
        is_ready: true,
        is_alive: true,
        changed: tokio::sync::Notify::new(),
    });
    for bytes in [
        "",
        "雪 café 😀",
        "rgba(1, 2, 3, 0.5)",
        "#fafafa",
        "NaN",
        "Infinity",
    ] {
        let ack = h
            .request(&p, "host.clipboard.write", json!({"text":bytes}))
            .await
            .unwrap();
        assert_eq!(ack, json!({"ok":true}));
        assert_eq!(
            h.ctx
                .worker
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .written
                .last()
                .unwrap(),
            bytes
        );
    }
    assert!(h
        .request(&p, "host.clipboard.write", json!({"text":null}))
        .await
        .is_err());
    assert!(h
        .request(&p, "host.clipboard.write", json!({"text":"x","html":false}))
        .await
        .is_err());
    assert_eq!(
        h.request(&p, "host.window.visibility", json!({"visible":false}))
            .await
            .unwrap(),
        Value::Null
    );
    h.ctx.worker.lock().unwrap().as_mut().unwrap().fail = true;
    assert!(h
        .request(&p, "host.clipboard.write", json!({"text":"x"}))
        .await
        .is_err());
    h.manifest = json!({"capabilities":[]});
    assert!(h
        .request(&p, "host.clipboard.write", json!({"text":"x"}))
        .await
        .is_err());
}
