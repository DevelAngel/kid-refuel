use leptos::mount::mount_to_body;
use refuel_app::prelude::App;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    tracing::info!("hydrate mode - hydrating");
    mount_to_body(App);
}
