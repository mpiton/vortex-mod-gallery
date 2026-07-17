//! Real ABI smoke test for the release WASM artifact.

use std::path::PathBuf;

use extism::{Function, UserData, Val, PTR};
use serde_json::{json, Value};

const WASM_REL_PATH: &str = "target/wasm32-wasip1/release/vortex_mod_gallery.wasm";
const IMGUR_URL: &str = "https://imgur.com/a/abc123";
const GENERIC_URL: &str = "https://example.test/gallery/page.html";
const GENERIC_IMAGE_URL: &str = "https://example.test/gallery/sample.jpg";
const IMGUR_BODY: &str = r#"{"data":[{"id":"img1","title":"sample","link":"https://i.imgur.com/img1.jpg","width":1920,"height":1080}],"status":200,"success":true}"#;
const GENERIC_BODY: &str = r#"<html><body><img src="sample.jpg"></body></html>"#;
const SMOKE_CLIENT_ID: &str = "SMOKE_CLIENT";

fn wasm_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(WASM_REL_PATH);
    assert!(
        path.is_file(),
        "missing release WASM artifact at {}; run `cargo build --target wasm32-wasip1 --release` first",
        path.display()
    );
    path
}

fn stub_http_request() -> Function {
    Function::new(
        "http_request",
        [PTR],
        [PTR],
        UserData::<()>::default(),
        |plugin, inputs, outputs, _user_data: UserData<()>| {
            let input = inputs[0]
                .i64()
                .ok_or_else(|| extism::Error::msg("http_request expected i64 input"))?;
            let request: String = plugin.memory_get_val(&Val::I64(input))?;
            let request: Value = serde_json::from_str(&request)?;
            let url = request["url"]
                .as_str()
                .ok_or_else(|| extism::Error::msg("http_request URL is missing"))?;
            let body = if url.contains("api.imgur.com") {
                // The configured client id must travel from `get_config`
                // all the way into the Imgur Authorization header.
                let auth = request["headers"]["Authorization"].as_str().unwrap_or("");
                if auth != format!("Client-ID {SMOKE_CLIENT_ID}") {
                    return Err(extism::Error::msg(format!(
                        "imgur request carries wrong Authorization header: {auth:?}"
                    )));
                }
                IMGUR_BODY
            } else if url == GENERIC_URL {
                GENERIC_BODY
            } else {
                return Err(extism::Error::msg(format!(
                    "unexpected HTTP request URL: {url}"
                )));
            };
            let response = json!({ "status": 200, "headers": {}, "body": body }).to_string();
            let handle = plugin.memory_new(&response)?;
            outputs[0] = Val::I64(handle.offset() as i64);
            Ok(())
        },
    )
}

fn stub_get_config() -> Function {
    Function::new(
        "get_config",
        [PTR],
        [PTR],
        UserData::<()>::default(),
        |plugin, inputs, outputs, _user_data: UserData<()>| {
            let input = inputs[0]
                .i64()
                .ok_or_else(|| extism::Error::msg("get_config expected i64 input"))?;
            let key: String = plugin.memory_get_val(&Val::I64(input))?;
            let value = if key == "imgur_client_id" {
                SMOKE_CLIENT_ID
            } else {
                ""
            };
            let handle = plugin.memory_new(value)?;
            outputs[0] = Val::I64(handle.offset() as i64);
            Ok(())
        },
    )
}

#[test]
fn release_wasm_exports_match_gallery_contract() {
    let manifest = extism::Manifest::new([extism::Wasm::file(wasm_path())]);
    let mut plugin = extism::Plugin::new(&manifest, [stub_http_request(), stub_get_config()], true)
        .expect("load release WASM");

    let can_handle: String = plugin
        .call("can_handle", IMGUR_URL)
        .expect("call can_handle");
    assert_eq!(can_handle.trim(), "true");

    let supports_playlist: String = plugin
        .call("supports_playlist", IMGUR_URL)
        .expect("call supports_playlist");
    assert_eq!(supports_playlist.trim(), "true");

    let links: String = plugin
        .call("extract_links", IMGUR_URL)
        .expect("call extract_links");
    let links: Value = serde_json::from_str(&links).expect("extract_links JSON");
    assert_eq!(links["kind"], "gallery");
    assert_eq!(links["provider"], "imgur");
    assert_eq!(links["images"][0]["url"], "https://i.imgur.com/img1.jpg");

    let generic: String = plugin
        .call("extract_generic", GENERIC_URL)
        .expect("call extract_generic");
    let generic: Value = serde_json::from_str(&generic).expect("extract_generic JSON");
    assert_eq!(generic["provider"], "generic");
    assert_eq!(generic["images"][0]["url"], GENERIC_IMAGE_URL);

    let is_http: String = plugin
        .call("is_http_url", GENERIC_URL)
        .expect("call is_http_url");
    assert_eq!(is_http.trim(), "true");
}
