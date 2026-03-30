use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub fn ws_connect(url: &str) -> Result<WebSocket, JsValue> {
    WebSocket::new(url)
}
