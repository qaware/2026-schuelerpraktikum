//! Orbitansicht: die Bahnebenen der Satelliten, schraeg von oben.
//!
//! Gezeichnet wird die Szene von `OrbitScene` aus `public/orbit-visualization.js`
//! (uebernommen aus dem Projekt `webflow-orbit`). Diese Seite haengt sie nur ein
//! und raeumt sie wieder ab -- ihre Daten holt sich die Szene selbst von
//! `/satellites/log`, und zwischen zwei Abfragen laesst sie einen eigenen
//! Propagator laufen, den die Messungen nur korrigieren statt ihn zu ersetzen.
//!
//! Rust bleibt damit fuer die Datenkarten darunter zustaendig. Die haben eine
//! eigene Abfrageschleife, weil sie pro Satellit Werte brauchen, die im
//! Sammel-Log nicht stehen: Modell, Nation und Sensorzahl stehen in den Specs
//! unter `/satellites/{name}`.
//!
//! Die Bahnbeschleunigung sitzt im Generator (`ORBIT_SPEEDUP`) und nicht hier:
//! so bewegen sich die Satelliten in den Daten selbst. Die Konstante unten muss
//! dazu passen -- sonst laeuft der Propagator der Messung davon oder hinterher,
//! und jede Abfrage holt den Satelliten sichtbar zurueck.

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::anim;
use crate::components::dashboard::{
    fetch_satellite_logs, fetch_satellites, fetch_specs, format_time, LogEntry, SatelliteSpecs,
};

// --- Physik -------------------------------------------------------------------

const EARTH_RADIUS_KM: f64 = 6371.0;
/// Gravitationsparameter der Erde in km^3/s^2.
const MU: f64 = 398_600.4418;

// --- Abfrage ------------------------------------------------------------------

/// Abfrageintervall der Datenkarten.
///
/// Dient zugleich als `pollMs` der Bahnansicht, damit beide denselben Stand
/// zeigen.
const POLL_MS: u32 = 2500;

/// Wie viele Logeintraege die Datenkarten je Satellit ziehen.
///
/// Gebraucht wird nur der neueste Stand, aber Temperatur und Druck sitzen auf
/// verschiedenen Sensoren: der Wert muss also jeden Sensor mindestens einmal
/// erwischen. 250 ist dafuer reichlich bemessen.
const CARD_LOG_AMOUNT: usize = 250;

// --- Bahnansicht (OrbitScene aus webflow-orbit) --------------------------------

/// Wie viele Logeintraege die Bahnansicht je Abfrage zieht.
///
/// `/satellites/log` liefert die neuesten Eintraege ueber alle Satelliten
/// hinweg, und jeder Satellit meldet je Zeitstempel einmal pro Sensor. Der
/// Wert muss also ein Vielfaches der Sensorzahl abdecken: 250 reicht fuer gut
/// ein Dutzend Abfragetakte, sodass auch ein selten meldender Satellit noch
/// enthalten ist.
const SCENE_LOG_AMOUNT: usize = 250;

/// Zeitraffer der Simulation, wie ihn `ORBIT_SPEEDUP` im Generator setzt.
///
/// Die Bahnansicht laesst zwischen zwei Abfragen einen eigenen Propagator
/// laufen. Steht dieser Faktor zu niedrig, zieht jede neue Messung den
/// Satelliten wieder nach vorn; steht er zu hoch, laeuft er ihr davon und wird
/// zurueckgeholt. In beiden Faellen ruckelt es im Abfragetakt.
const ORBIT_SPEEDUP: f64 = 120.0;

const COLORS: [&str; 6] = ["#38bdf8", "#f472b6", "#4ade80", "#fbbf24", "#a78bfa", "#fb7185"];

/// Momentaufnahme eines Satelliten fuer seine Datenkarte.
#[derive(Clone, Debug, PartialEq)]
struct SatView {
    name: String,
    model: String,
    nation: String,
    sensor_count: usize,
    inclination: f64,
    height_km: f64,
    city: String,
    latitude: f64,
    longitude: f64,
    temperature: Option<f64>,
    pressure: Option<f64>,
    timestamp: u64,
}

/// Umlaufzeit nach dem dritten Keplerschen Gesetz.
fn orbital_period_seconds(height_km: f64) -> f64 {
    let a = EARTH_RADIUS_KM + height_km;
    2.0 * std::f64::consts::PI * (a.powi(3) / MU).sqrt()
}

fn format_period(seconds: f64) -> String {
    if seconds < 10_800.0 {
        format!("{:.0} min", seconds / 60.0)
    } else if seconds < 259_200.0 {
        format!("{:.1} h", seconds / 3600.0)
    } else {
        format!("{:.0} Tage", seconds / 86_400.0)
    }
}

fn format_height(km: f64) -> String {
    if km >= 100_000.0 {
        format!("{:.2} Mio. km", km / 1_000_000.0)
    } else {
        format!("{:.0} km", km)
    }
}

/// Grad mit Himmelsrichtung, wie bei Koordinatenangaben ueblich.
fn format_lat(deg: f64) -> String {
    format!("{:.2}° {}", deg.abs(), if deg >= 0.0 { "N" } else { "S" })
}

fn format_lon(deg: f64) -> String {
    format!("{:.2}° {}", deg.abs(), if deg >= 0.0 { "O" } else { "W" })
}

fn build_view(name: &str, specs: Option<&SatelliteSpecs>, entries: &[LogEntry]) -> Option<SatView> {
    let newest = entries.iter().max_by_key(|e| e.timestamp)?;

    // Temperatur und Druck getrennt suchen: ein Tank meldet nur eine der
    // beiden Groessen, der neueste Eintrag hat also nicht zwangslaeufig beide.
    let temperature = entries
        .iter()
        .filter(|e| e.temperature.is_some())
        .max_by_key(|e| e.timestamp)
        .and_then(|e| e.temperature);
    let pressure = entries
        .iter()
        .filter(|e| e.pressure.is_some())
        .max_by_key(|e| e.timestamp)
        .and_then(|e| e.pressure);

    Some(SatView {
        name: name.to_string(),
        model: specs.map(|s| s.model.clone()).unwrap_or_default(),
        nation: specs.map(|s| s.nation.clone()).unwrap_or_default(),
        sensor_count: specs.map(|s| s.sensors.len()).unwrap_or(0),
        inclination: specs.map(|s| s.inclination).unwrap_or(0.0),
        height_km: newest.position.height,
        city: newest.position.city.clone(),
        latitude: newest.position.latitude,
        longitude: newest.position.longitude,
        temperature,
        pressure,
        timestamp: newest.timestamp,
    })
}

#[component]
pub fn Orbit() -> impl IntoView {
    // Zwei getrennte Signale, und das ist der Kern des Aufbaus:
    //
    // `order` (Name + Nennhoehe, nach Hoehe sortiert) aendert sich nur, wenn
    // Satelliten dazukommen oder wegfallen. Daran haengt, welche Karten es
    // gibt und in welcher Farbe. `views` traegt die Messwerte und darf sich
    // beliebig oft aendern: die Werte stehen in reaktiven Closures, Leptos
    // tauscht dort nur Textknoten -- die Karten selbst bleiben stehen.
    let (order, set_order) = signal(Vec::<(String, f64)>::new());
    let (views, set_views) = signal(HashMap::<String, SatView>::new());
    let (loaded, set_loaded) = signal(false);
    let (offline, set_offline) = signal(false);

    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || {
        cleanup_flag.store(false, Ordering::Relaxed);
        // Die Bahnansicht bringt ihren eigenen Ticker *und* ihr eigenes
        // Polling mit -- ohne dieses Abraeumen fragt sie im Hintergrund
        // weiter das Backend ab, auch auf jeder anderen Seite.
        anim::orbit_scene_destroy();
    });

    spawn_local(async move {
        // Specs sind unveraenderlich, also einmal holen und behalten.
        let mut specs_cache: HashMap<String, SatelliteSpecs> = HashMap::new();

        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            match fetch_satellites().await {
                Ok(names) => {
                    let mut fresh: HashMap<String, SatView> = HashMap::new();

                    for name in &names {
                        if !alive.load(Ordering::Relaxed) {
                            return;
                        }
                        if !specs_cache.contains_key(name) {
                            if let Ok(spec) = fetch_specs(name).await {
                                specs_cache.insert(name.clone(), spec);
                            }
                        }
                        if let Ok(logs) = fetch_satellite_logs(name, CARD_LOG_AMOUNT).await {
                            if let Some(view) =
                                build_view(name, specs_cache.get(name), &logs.data)
                            {
                                fresh.insert(name.clone(), view);
                            }
                        }
                    }

                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }

                    // Kartenliste nur anfassen, wenn sich die Satellitenmenge
                    // geaendert hat.
                    let mut names_now: Vec<String> = fresh.keys().cloned().collect();
                    names_now.sort();
                    let mut names_before: Vec<String> = order
                        .get_untracked()
                        .iter()
                        .map(|(n, _)| n.clone())
                        .collect();
                    names_before.sort();

                    if names_now != names_before {
                        let mut next: Vec<(String, f64)> = fresh
                            .values()
                            .map(|v| (v.name.clone(), v.height_km))
                            .collect();
                        next.sort_by(|a, b| {
                            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        set_order.set(next);
                    }

                    set_views.set(fresh);
                    if !loaded.get_untracked() {
                        set_loaded.set(true);
                    }
                    if offline.get_untracked() {
                        set_offline.set(false);
                    }
                }
                Err(_) => {
                    if !offline.get_untracked() {
                        set_offline.set(true);
                    }
                }
            }

            TimeoutFuture::new(POLL_MS).await;
        }
    });

    Effect::new(move |_| {
        let _ = order.get();
        anim::reveal_once("[data-anim=\"reveal\"]", 0.1);
    });

    // Bahnansicht aufbauen -- bewusst ohne reaktive Abhaengigkeit, also genau
    // einmal beim Betreten der Seite.
    //
    // Sie holt ihre Daten selbst (`dataUrl`/`pollMs`) statt sie von der
    // Schleife oben durchgereicht zu bekommen: die Schleife oben fragt pro
    // Satellit einzeln ab, weil die Karten die Specs dazu brauchen, die Szene
    // kommt mit dem Sammel-Log in einer Abfrage aus. Ein Neuaufbau je Abfrage
    // waere ohnehin ausgeschlossen -- er wuerde den laufenden Propagator und
    // alle Korrektur-Tweens verwerfen.
    Effect::new(move |_| {
        let palette = COLORS
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(",");

        anim::orbit_scene_create(
            "#orbit-scene",
            &format!(
                r##"{{"dataUrl":"/satellites/log?amount={}","pollMs":{},"timeScale":{:.1},"palette":[{}]}}"##,
                SCENE_LOG_AMOUNT, POLL_MS, ORBIT_SPEEDUP, palette
            ),
        );
    });

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div data-anim="reveal" class="flex flex-col gap-4 border-b border-gray-200 pt-2 pb-6 md:flex-row md:items-center md:justify-between">
                <div>
                    <div class="flex items-center gap-2">
                        <h1 class="text-sheen text-3xl font-bold tracking-tight">"Orbit"</h1>
                        <span class="relative flex h-2 w-2" aria-hidden="true">
                            <span class="animate-live-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400"></span>
                            <span class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
                        </span>
                    </div>
                    <p class="mt-1 text-sm text-gray-500">
                        "Bahnhöhe, Neigung und Position stammen aus den Messungen; Umlaufzeit und Bahngeschwindigkeit ergeben sich daraus nach Kepler."
                    </p>
                </div>
                {move || offline.get().then(|| view! {
                    <span class="animate-pop-in rounded-full bg-red-100 px-3 py-1 text-xs font-semibold text-red-700">
                        "Backend nicht erreichbar"
                    </span>
                })}
            </div>

            // --- Bahnansicht ---------------------------------------------------
            // Das SVG darin baut das Modul in orbit-visualization.js auf; hier
            // steht nur der leere Mount. Deshalb auch kein data-anim="reveal":
            // die Szene blendet ihre Bahnen selbst ein (`intro`), und beide
            // Einblendungen zugleich sehen aus wie ein Fehler.
            <div class="my-8 overflow-hidden rounded-3xl border border-slate-800 shadow-xl">
                // Die Ecken macht die Karte, nicht die Szene: `.os-root` bringt
                // eigene 18px mit, die innerhalb der 24px hier als heller Rest
                // in den Ecken stehen bleiben wuerden. Inline, weil die Klasse
                // aus orbit-visualization.css nach output.css geladen wird und
                // eine Tailwind-Utility bei gleicher Spezifitaet verlieren
                // wuerde.
                <div id="orbit-scene" style="border-radius: 0"></div>

                <div class="flex flex-wrap items-center gap-x-5 gap-y-1 border-t border-slate-800 bg-slate-900/60 px-5 py-3 text-[11px] text-slate-400">
                    <span class="font-semibold text-slate-300">"Aus den Daten:"</span>
                    <span>"Bahnhöhe"</span>
                    <span>"Neigung"</span>
                    <span>"Position (für die Lage auf der Bahn)"</span>
                    <span class="font-semibold text-slate-300">"Berechnet:"</span>
                    <span>"Umlaufzeit und Bahngeschwindigkeit nach Kepler"</span>
                    <span>"Ringabstände logarithmisch, nicht maßstäblich"</span>
                </div>
            </div>

            // --- Datenkarten ---------------------------------------------------
            {move || (!loaded.get()).then(|| view! {
                <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
                    {(0..3).map(|_| view! {
                        <div class="rounded-2xl border border-gray-200 bg-white p-5 shadow-sm">
                            <div class="skeleton mb-3 h-5 w-24 rounded-full"></div>
                            <div class="skeleton mb-2 h-4 w-full rounded-full"></div>
                            <div class="skeleton h-4 w-2/3 rounded-full"></div>
                        </div>
                    }).collect_view()}
                </div>
            })}

            <div class="mb-10 grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
                {move || {
                    let items = order.get();
                    items.iter().enumerate().map(|(i, (name, _))| {
                        let color = COLORS[i % COLORS.len()];
                        let title = name.clone();

                        // Ein Feld der Karte als reaktiver Wert.
                        //
                        // Vorher hing das ganze <dl> an einem `match` ueber
                        // `views`: jede Abfrage warf 13 Zeilen weg und baute
                        // sie neu -- ein Layout-Sprung genau im Abfragetakt.
                        // Jetzt steht die Liste fest und nur die Textknoten
                        // wechseln.
                        let field = {
                            let name = name.clone();
                            move |read: fn(&SatView) -> String| {
                                let key = name.clone();
                                Signal::derive(move || {
                                    views.with(|m| {
                                        m.get(&key).map(read).unwrap_or_else(|| "—".to_string())
                                    })
                                })
                            }
                        };

                        view! {
                            <div
                                data-anim="reveal"
                                class="rounded-2xl border border-gray-200 bg-white p-5 shadow-sm transition-all duration-300 hover:-translate-y-1 hover:shadow-lg"
                            >
                                <div class="mb-3 flex items-center gap-2">
                                    <span
                                        class="inline-block h-3 w-3 rounded-full"
                                        style=format!("background-color: {}", color)
                                    ></span>
                                    <h2 class="text-base font-bold text-gray-900">{title}</h2>
                                </div>
                                <dl class="space-y-1.5 text-sm">
                                    <Row label="Modell" value=field(|v| v.model.clone()) />
                                    <Row label="Nation" value=field(|v| v.nation.clone()) />
                                    <Row label="Breite" value=field(|v| format_lat(v.latitude)) />
                                    <Row label="Länge" value=field(|v| format_lon(v.longitude)) />
                                    <Row label="Bahnhöhe" value=field(|v| format_height(v.height_km)) />
                                    <Row label="Neigung" value=field(|v| format!("{:.2}°", v.inclination)) />
                                    <Row
                                        label="Umlaufzeit"
                                        value=field(|v| format_period(orbital_period_seconds(v.height_km)))
                                    />
                                    <Row label="Bodenstation" value=field(|v| v.city.clone()) />
                                    <Row
                                        label="Temperatur"
                                        value=field(|v| v.temperature
                                            .map(|t| format!("{:.2} K", t))
                                            .unwrap_or_else(|| "—".into()))
                                    />
                                    <Row
                                        label="Druck"
                                        value=field(|v| v.pressure
                                            .map(|p| format!("{:.2} Bar", p))
                                            .unwrap_or_else(|| "—".into()))
                                    />
                                    <Row label="Sensoren" value=field(|v| v.sensor_count.to_string()) />
                                    <Row label="Letzte Messung" value=field(|v| format_time(v.timestamp)) />
                                </dl>
                            </div>
                        }
                    }).collect_view()
                }}
            </div>
        </div>
    }
}

/// Eine Zeile der Datenkarte.
///
/// `value` ist bewusst ein `Signal` und kein `String`: so bleibt die Zeile beim
/// Aktualisieren stehen und nur ihr Textknoten wechselt.
#[component]
fn Row(label: &'static str, value: Signal<String>) -> impl IntoView {
    view! {
        <div class="flex items-baseline justify-between gap-3 border-b border-gray-100 pb-1 last:border-b-0">
            <dt class="text-xs text-gray-500">{label}</dt>
            <dd class="text-right font-semibold text-gray-900">{move || value.get()}</dd>
        </div>
    }
}
