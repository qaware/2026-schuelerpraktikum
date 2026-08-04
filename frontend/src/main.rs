use leptos::prelude::*;

mod app;
mod components;

use app::App;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    leptos::mount::mount_to_body(|| view! { <App/> })
}