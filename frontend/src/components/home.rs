use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::components::dashboard::{fetch_logs, fetch_satellites, format_date, LogEntry};

/// A measurement older than this counts as loss of signal.
const STALE_AFTER_SECONDS: u64 = 30;

/// Size of the live window. At roughly six measurements per second this covers
/// about half a minute, which is all the "last N" endpoint can give us -- there
/// is no time-range query on the backend.
const WINDOW: usize = 200;

fn now_seconds() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

#[derive(Clone)]
struct SatStatus {
    name: String,
    model: String,
    nation: String,
    /// City the satellite was over at its last contact.
    city: String,
    /// Orbit height in km at its last contact.
    height: Option<f64>,
    last_contact: Option<u64>,
    sensors_seen: usize,
}

#[component]
fn StatTile(label: &'static str, value: String, hint: String) -> impl IntoView {
    view! {
        <div class="p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-sm">
            <p class="text-xs uppercase tracking-wide text-slate-400">{label}</p>
            <p class="mt-2 text-3xl font-bold text-slate-100 tabular-nums">{value}</p>
            <p class="mt-1 text-xs text-slate-500">{hint}</p>
        </div>
    }
}

use crate::anim;

#[component]
pub fn Home() -> impl IntoView {
    let (logs, set_logs) = signal(Vec::<LogEntry>::new());
    let (sats, set_sats) = signal(Vec::<String>::new());
    let (loaded, set_loaded) = signal(false);

    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    spawn_local(async move {
        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(names) = fetch_satellites().await {
                if !alive.load(Ordering::Relaxed) {
                    break;
                }
                if names != sats.get_untracked() {
                    set_sats.set(names);
                }
            }

            if let Ok(data) = fetch_logs(WINDOW).await {
                if !alive.load(Ordering::Relaxed) {
                    break;
                }
                set_logs.set(data.data);
                set_loaded.set(true);
            }

            TimeoutFuture::new(3000).await;
        }
    });

    // --- derived values -----------------------------------------------------

    // Newest measurement per satellite, plus the specs that came with it.
    let statuses = move || -> Vec<SatStatus> {
        let entries = logs.get();
        let mut last: HashMap<String, &LogEntry> = HashMap::new();
        let mut sensors: HashMap<String, HashSet<String>> = HashMap::new();

        for entry in entries.iter() {
            let name = entry.specs.name.clone();
            if name.is_empty() || name == "TEST_SAT" || name == "test_sat" {
                continue;
            }
            sensors
                .entry(name.clone())
                .or_default()
                .insert(entry.sensor_name.clone());

            last.entry(name)
                .and_modify(|prev| {
                    if entry.timestamp > prev.timestamp {
                        *prev = entry;
                    }
                })
                .or_insert(entry);
        }

        let mut names: Vec<String> = sats.get().into_iter().filter(|n| !n.is_empty() && n != "TEST_SAT" && n != "test_sat").collect();
        // A satellite that fell out of the window still deserves a row, and one
        // that only just appeared may not be in /satellites yet.
        for name in last.keys() {
            if !name.is_empty() && name != "TEST_SAT" && name != "test_sat" && !names.contains(name) {
                names.push(name.clone());
            }
        }
        names.sort();

        names
            .into_iter()
            .map(|name| {
                let entry = last.get(&name);
                SatStatus {
                    model: entry.map(|e| e.specs.model.clone()).unwrap_or_else(|| "—".to_string()),
                    nation: entry.map(|e| e.specs.nation.clone()).unwrap_or_else(|| "—".to_string()),
                    city: entry.map(|e| e.position.city.clone()).unwrap_or_else(|| "—".to_string()),
                    height: entry.map(|e| e.position.height),
                    last_contact: entry.map(|e| e.timestamp),
                    sensors_seen: sensors.get(&name).map(|s| s.len()).unwrap_or(0),
                    name,
                }
            })
            .collect()
    };

    let tiles = move || {
        let entries = logs.get();
        let now = now_seconds();
        let all = statuses();

        let online = all
            .iter()
            .filter(|s| {
                s.last_contact
                    .map(|t| now.saturating_sub(t) < STALE_AFTER_SECONDS)
                    .unwrap_or(false)
            })
            .count();

        let distinct_sensors: HashSet<&String> =
            entries.iter().map(|e| &e.sensor_name).collect();

        let newest = entries.iter().map(|e| e.timestamp).max();
        let oldest = entries.iter().map(|e| e.timestamp).min();

        let rate = match (newest, oldest) {
            (Some(n), Some(o)) if n > o => entries.len() as f64 / (n - o) as f64,
            _ => 0.0,
        };

        let since = newest
            .map(|t| format!("{} s", now.saturating_sub(t)))
            .unwrap_or_else(|| "—".to_string());

        let window_hint = match (newest, oldest) {
            (Some(n), Some(o)) if n > o => format!("Fenster: {} s", n - o),
            _ => "Fenster: —".to_string(),
        };

        view! {
            <StatTile
                label="Satelliten online"
                value=format!("{} / {}", online, all.len())
                hint=format!("Signal jünger als {} s", STALE_AFTER_SECONDS)
            />
            <StatTile
                label="Aktive Sensoren"
                value=distinct_sensors.len().to_string()
                hint=window_hint
            />
            <StatTile
                label="Messungen / s"
                value=format!("{:.1}", rate)
                hint=format!("über {} Messungen", entries.len())
            />
            <StatTile
                label="Letzter Kontakt"
                value=since
                hint="seit der neuesten Messung".to_string()
            />
        }
    };

    Effect::new(move |_| {
        anim::reveal_once("[data-anim=\"reveal\"]", 0.1);
    });

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div data-anim="reveal" class="flex flex-col gap-4 border-b border-slate-800 pt-2 pb-6 md:flex-row md:items-center md:justify-between">
                <div>
                    <div class="flex items-center gap-2">
                        <h1 class="text-sheen text-3xl font-bold tracking-tight">"Home"</h1>
                        <span class="relative flex h-2 w-2" aria-hidden="true">
                            <span class="animate-live-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400"></span>
                            <span class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
                        </span>
                    </div>
                </div>
            </div>

            <div
                data-anim="reveal" class="mt-8 space-y-4 rounded-2xl border border-slate-800 bg-slate-900 p-8 shadow-sm transition-all duration-300 hover:shadow-md"
            >
                <h2 class="text-2xl font-semibold text-slate-200">"Satelliten-Telemetrie"</h2>
                <p class="text-slate-400 leading-relaxed">
                    "Dieses Projekt simuliert die Bodenstation einer Satellitenflotte. Sensordaten
                    werden erzeugt, gespeichert und hier in Echtzeit dargestellt."
                </p>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 pt-2">
                    <div class="p-4 rounded-xl bg-slate-800/50 border border-slate-700">
                        <p class="text-xs uppercase tracking-wide text-slate-400 mb-1">"1 · Datenerzeugung"</p>
                        <p class="text-sm text-slate-300">
                            "Ein Python-Generator erzeugt Druck- und Temperaturwerte für 13 Sensoren
                            und sendet sie per HTTP POST an das Backend."
                        </p>
                    </div>
                    <div class="p-4 rounded-xl bg-slate-800/50 border border-slate-700">
                        <p class="text-xs uppercase tracking-wide text-slate-400 mb-1">"2 · Backend"</p>
                        <p class="text-sm text-slate-300">
                            "Ein Go-Server (chi) validiert die Messwerte, legt sie in MongoDB ab und
                            stellt sie über eine REST-API bereit."
                        </p>
                    </div>
                    <div class="p-4 rounded-xl bg-slate-800/50 border border-slate-700">
                        <p class="text-xs uppercase tracking-wide text-slate-400 mb-1">"3 · Frontend"</p>
                        <p class="text-sm text-slate-300">
                            "Diese Oberfläche ist in Rust mit Leptos geschrieben und läuft als
                            WebAssembly im Browser. Sie fragt die API im Sekundentakt ab."
                        </p>
                    </div>
                </div>
            </div>

            <div data-anim="reveal" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 mt-8">
                {move || if loaded.get() {
                    tiles().into_any()
                } else {
                    view! {
                        <div class="col-span-full text-slate-500">"Lade Kennzahlen..."</div>
                    }.into_any()
                }}
            </div>

            <div class="mt-8 grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div data-anim="reveal" class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-sm transition-all duration-300 hover:shadow-md">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-semibold text-slate-200">"Flottenstatus"</h3>
                        <a href="/dashboard" class="text-xs text-blue-400 hover:text-blue-300">"Zum Dashboard →"</a>
                    </div>

                    {move || {
                        let all = statuses();
                        if all.is_empty() {
                            return view! {
                                <p class="text-sm text-slate-500">"Noch keine Satelliten empfangen."</p>
                            }.into_any();
                        }

                        let now = now_seconds();

                        view! {
                            <div class="divide-y divide-slate-800">
                                {all.into_iter().map(|s| {
                                    let fresh = s.last_contact
                                        .map(|t| now.saturating_sub(t) < STALE_AFTER_SECONDS)
                                        .unwrap_or(false);
                                    let contact = s.last_contact
                                        .map(|t| format!("vor {} s", now.saturating_sub(t)))
                                        .unwrap_or_else(|| "kein Kontakt".to_string());
                                    let height = s.height
                                        .map(|h| format!("{:.1} km", h))
                                        .unwrap_or_else(|| "—".to_string());

                                    view! {
                                        <div class="flex items-center justify-between gap-3 py-3">
                                            <div class="flex items-center gap-2.5 min-w-0">
                                                <span class=if fresh {
                                                    "w-2 h-2 rounded-full bg-emerald-500 animate-pulse shrink-0"
                                                } else {
                                                    "w-2 h-2 rounded-full bg-amber-500 shrink-0"
                                                }></span>
                                                <div class="min-w-0">
                                                    <p class="text-sm font-medium text-slate-100 truncate">{s.name}</p>
                                                    <p class="text-xs text-slate-400 truncate">
                                                        {format!("{} · {}", s.model, s.nation)}
                                                    </p>
                                                </div>
                                            </div>
                                            // Hidden on the narrowest screens so the name never gets
                                            // squeezed out by the position block.
                                            <div class="hidden sm:block text-right shrink-0">
                                                <p class="text-xs text-slate-300 truncate">{s.city}</p>
                                                <p class="text-xs text-slate-500 tabular-nums">{height}</p>
                                            </div>
                                            <div class="text-right shrink-0">
                                                <p class="text-xs text-slate-300 tabular-nums">{contact}</p>
                                                <p class="text-xs text-slate-500">
                                                    {format!("{} Sensoren", s.sensors_seen)}
                                                </p>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </div>

                <div data-anim="reveal" class="p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-sm transition-all duration-300 hover:shadow-md">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-semibold text-slate-200">"Live-Ticker"</h3>
                        <span class="text-xs text-slate-500">"neueste zuerst"</span>
                    </div>

                    {move || {
                        let entries = logs.get();
                        if entries.is_empty() {
                            return view! {
                                <p class="text-sm text-slate-500">"Warte auf Daten..."</p>
                            }.into_any();
                        }

                        view! {
                            <div class="divide-y divide-slate-800 max-h-80 overflow-y-auto">
                                {entries.iter().rev().take(15).map(|e| {
                                    // A tank carries only one of the two probes.
                                    let reading = match (e.temperature, e.pressure) {
                                        (Some(t), Some(p)) => format!("{:.1} K · {:.2} Bar", t, p),
                                        (Some(t), None) => format!("{:.1} K", t),
                                        (None, Some(p)) => format!("{:.2} Bar", p),
                                        (None, None) => "—".to_string(),
                                    };

                                    view! {
                                        <div class="flex items-center justify-between gap-3 py-2">
                                            <div class="min-w-0">
                                                <p class="text-xs font-medium text-slate-100 truncate">
                                                    {format!("{} · {}", e.specs.name, e.sensor_name)}
                                                </p>
                                                <p class="text-xs text-slate-500 truncate">
                                                    <span class="font-mono">{format_date(e.timestamp)}</span>
                                                    {format!(" · {} · {:.1} km", e.position.city, e.position.height)}
                                                </p>
                                            </div>
                                            <p class="text-xs text-slate-300 tabular-nums shrink-0">{reading}</p>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
