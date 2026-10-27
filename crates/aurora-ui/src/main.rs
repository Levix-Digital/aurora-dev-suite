//! Aurora Dev Suite: the interface (Leptos, WebAssembly). It talks to the desktop through
//! Tauri, or to the core's fixtures when it runs alone in a browser (research R2).

mod app;
mod backend;
#[cfg(test)]
mod theme_contrast;
mod views;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
