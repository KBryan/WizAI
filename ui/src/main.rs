/* @refresh reload */
use leptos::wasm_bindgen::prelude::set_panic_hook;

pub fn main() {
    set_panic_hook();
    console_error_panic_hook::set_once();

    leptos::mount_to_body(|| {
        use wizai_ui::app::App;
        leptos::view! { <App /> }
    });
}
