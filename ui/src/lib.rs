pub mod app;
pub mod components;
pub mod api;
pub mod ws;
pub mod state;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}
