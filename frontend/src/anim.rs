//! Bruecke zu den GSAP-Helfern aus `public/animations.js`.
//!
//! Alle Aufrufe sind mit `catch` deklariert und werden hier verschluckt: eine
//! fehlende Animation darf die App nie zum Absturz bringen. Wenn `gsap.min.js`
//! nicht laedt, definiert `animations.js` No-ops -- und sollte selbst das
//! fehlen, landet der Fehler im `Result` und wird ignoriert.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "satAnim"], js_name = revealOnce)]
    fn js_reveal_once(selector: &str, stagger: f64) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "satAnim"], js_name = drawPaths)]
    fn js_draw_paths(root_selector: &str, stagger: f64) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "satAnim"], js_name = popDots)]
    fn js_pop_dots(root_selector: &str, stagger: f64) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "satAnim"], js_name = countTo)]
    fn js_count_to(selector: &str, value: f64) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "satAnim"], js_name = orbitSceneCreate)]
    fn js_orbit_scene_create(mount_selector: &str, options_json: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "satAnim"], js_name = orbitSceneDestroy)]
    fn js_orbit_scene_destroy() -> Result<(), JsValue>;
}

/// Blendet alle noch nicht animierten Treffer von `selector` gestaffelt ein.
pub fn reveal_once(selector: &str, stagger: f64) {
    let _ = js_reveal_once(selector, stagger);
}

/// Zeichnet die Messreihen innerhalb von `root_selector` ein.
pub fn draw_paths(root_selector: &str, stagger: f64) {
    let _ = js_draw_paths(root_selector, stagger);
}

/// Laesst die Messpunkte innerhalb von `root_selector` aufpoppen.
pub fn pop_dots(root_selector: &str, stagger: f64) {
    let _ = js_pop_dots(root_selector, stagger);
}

/// Zaehlt das Element hinter `selector` auf `value` hoch.
pub fn count_to(selector: &str, value: u64) {
    let _ = js_count_to(selector, value as f64);
}

/// Baut die Bahnansicht (OrbitScene) in das Element hinter `mount_selector`.
///
/// `options_json` ist die Konfiguration des Moduls -- siehe `DEFAULTS` in
/// `public/orbit-visualization.js`. `mount` wird drueben gesetzt und muss hier
/// nicht mit hinein.
///
/// Die Szene holt ihre Daten selbst (`dataUrl`/`pollMs`) und haelt sie mit
/// einem eigenen Propagator in Bewegung. Rust reicht deshalb keine Messwerte
/// durch, sondern nur einmal die Konfiguration.
pub fn orbit_scene_create(mount_selector: &str, options_json: &str) {
    let _ = js_orbit_scene_create(mount_selector, options_json);
}

/// Raeumt die Bahnansicht ab: Ticker, Tweens, Polling und DOM.
pub fn orbit_scene_destroy() {
    let _ = js_orbit_scene_destroy();
}
