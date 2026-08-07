use leptos::prelude::*;

mod anim;
mod app;
mod components;

use app::App;

fn main() {
    // Im Release-Build nur Warnungen und Fehler: Debug-Logs kosten Bundle-Groesse
    // und fluten die Konsole bei den Polling-Schleifen.
    #[cfg(debug_assertions)]
    let level = log::Level::Debug;
    #[cfg(not(debug_assertions))]
    let level = log::Level::Warn;

    _ = console_log::init_with_level(level);
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(|| view! { <App/> })
}