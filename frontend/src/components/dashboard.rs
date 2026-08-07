use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::anim;

// Empty on purpose: requests go to the page's own origin and are forwarded to
// the backend by nginx (Docker) or by Trunk's [[proxy]] (dev). Pointing this at
// http://localhost:8585 directly would be a cross-origin request, which the
// browser blocks because the backend sends no CORS headers.
const API_BASE: &str = "";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SatelliteLogResponse {
    pub amount: usize,
    pub data: Vec<LogEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub sensor_name: String,
    // Tanks only carry one of the two probes, so either value can be absent.
    pub pressure: Option<f64>,
    pub temperature: Option<f64>,
    pub position: Position,
    pub specs: Specs,
    pub timestamp: u64,
}

/// Der subsatellitare Punkt: die Stelle, ueber der der Satellit gerade steht.
///
/// `latitude`/`longitude` sind `serde(default)`, damit eine aeltere Datenquelle
/// ohne diese Felder (etwa der mock_api_server) das Deserialisieren nicht
/// scheitern laesst -- sie landen dann auf 0.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub city: String,
    pub height: f64,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Specs {
    pub name: String,
    pub model: String,
    pub launch_date: String,
    pub sensors: Vec<String>,
    pub nation: String,
}

/// Zustand der Backend-Verbindung.
///
/// Vorher war der gruene Punkt im Kopfbereich fest verdrahtet: ein Backend-
/// Ausfall sah damit genauso aus wie ein gesundes System, nur mit veralteten
/// Zahlen. Jetzt speisen die Polling-Schleifen diesen Zustand.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Conn {
    Connecting,
    Live,
    Offline,
}

/// Der Punkt, auf dem die Maus im Diagramm gerade steht.
///
/// Vorher ein `(f64, f64, String, String, String)`-Tupel, bei dem man am
/// Verwendungsort die Reihenfolge erraten musste.
#[derive(Clone, Debug, PartialEq)]
struct HoveredPoint {
    x: f64,
    y: f64,
    value: String,
    sensor: String,
    time: String,
}

pub async fn fetch_logs(amount: usize) -> Result<SatelliteLogResponse, gloo_net::Error> {
    let url = format!("{}/satellites/log?amount={}", API_BASE, amount);
    Request::get(&url)
        .send()
        .await?
        .json::<SatelliteLogResponse>()
        .await
}

pub async fn fetch_satellite_logs(name: &str, amount: usize) -> Result<SatelliteLogResponse, gloo_net::Error> {
    let url = format!("{}/satellites/{}/log?amount={}", API_BASE, name, amount);
    Request::get(&url)
        .send()
        .await?
        .json::<SatelliteLogResponse>()
        .await
}

/// Baut ein `js_sys::Date` aus einem Unix-Zeitstempel in Sekunden.
fn to_date(timestamp_sec: u64) -> js_sys::Date {
    let ms = (timestamp_sec * 1000) as f64;
    js_sys::Date::new(&JsValue::from_f64(ms))
}

fn format_date(timestamp_sec: u64) -> String {
    to_date(timestamp_sec)
        .to_locale_string("de-DE", &JsValue::UNDEFINED)
        .into()
}

pub fn format_time(timestamp_sec: u64) -> String {
    let date = to_date(timestamp_sec);
    format!(
        "{:02}:{:02}:{:02}",
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

pub async fn fetch_satellites() -> Result<Vec<String>, gloo_net::Error> {
    #[derive(Deserialize)]
    struct SatRes {
        names: Vec<String>,
    }
    let url = format!("{}/satellites", API_BASE);
    let res: SatRes = Request::get(&url)
        .send()
        .await?
        .json()
        .await?;
    Ok(res.names)
}

/// Antwort von `GET /satellites/{name}`.
///
/// Eigene Struct, obwohl `Specs` inhaltlich dasselbe traegt: dieser Endpunkt
/// nennt das Feld `launchdate`, die in den Logeintraegen eingebetteten Specs
/// dagegen `launch_date`. Ein gemeinsamer Typ wuerde bei einem der beiden
/// stillschweigend am Deserialisieren scheitern.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SatelliteSpecs {
    pub name: String,
    pub model: String,
    pub launchdate: String,
    pub sensors: Vec<String>,
    pub nation: String,
    /// Bahnneigung in Grad.
    #[serde(default)]
    pub inclination: f64,
}

pub async fn fetch_specs(name: &str) -> Result<SatelliteSpecs, gloo_net::Error> {
    let url = format!("{}/satellites/{}", API_BASE, name);
    Request::get(&url).send().await?.json().await
}

pub async fn fetch_sensors(name: &str) -> Result<Vec<String>, gloo_net::Error> {
    #[derive(Deserialize)]
    struct SensorRes {
        sensor_names: Vec<String>,
    }
    let url = format!("{}/satellites/{}/sensors", API_BASE, name);
    let res: SensorRes = Request::get(&url).send().await?.json().await?;
    Ok(res.sensor_names)
}

#[component]
pub fn SatelliteChart(name: String, index: usize) -> impl IntoView {
    // `None` heisst "Sensorliste noch nicht geladen". Vorher lag diese Liste
    // doppelt vor -- einmal als LocalResource fuer die Anzeige und einmal in der
    // Polling-Schleife, die sie separat nachgeladen hat.
    let (sensors, set_sensors) = signal(None::<Vec<String>>);
    let (chart_logs, set_chart_logs) = signal(Vec::<LogEntry>::new());
    let (selected_metric, set_selected_metric) = signal("temperature".to_string());
    let (deselected_sensors, set_deselected_sensors) = signal(HashSet::<String>::new());

    let (hovered_point, set_hovered_point) = signal(None::<HoveredPoint>);
    let (viewport_size, set_viewport_size) = signal(25usize);
    let (expanded, set_expanded) = signal(false);
    let (stale, set_stale) = signal(false);
    // Zaehlt erfolgreiche Abfragen. Die Einblend-Animationen laufen nur beim
    // ersten Datensatz -- sonst wuerde das Diagramm sich alle 2 Sekunden neu
    // aufbauen und flackern.
    let (batches, set_batches) = signal(0u32);

    // Without this the polling loop outlives the component: every visit to
    // /dashboard would leave another one running forever.
    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    let poll_name = name.clone();
    spawn_local(async move {
        let name = poll_name;

        // Erst die Sensorliste holen, mit Wiederholung solange das fehlschlaegt.
        let sensor_count = loop {
            if !alive.load(Ordering::Relaxed) {
                return;
            }
            match fetch_sensors(&name).await {
                Ok(s) => {
                    let count = s.len();
                    set_sensors.set(Some(s));
                    if stale.get_untracked() {
                        set_stale.set(false);
                    }
                    break count;
                }
                Err(_) => {
                    if !stale.get_untracked() {
                        set_stale.set(true);
                    }
                    TimeoutFuture::new(2000).await;
                }
            }
        };

        if sensor_count == 0 {
            return;
        }

        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            // The endpoint caps total rows, not rows per sensor, so scale
            // the request by how many sensors share that budget.
            let amount = viewport_size.get_untracked() * sensor_count;
            match fetch_satellite_logs(&name, amount).await {
                Ok(data) => {
                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }
                    set_chart_logs.set(data.data);
                    // Nur bis 2 hochzaehlen: der Wert steuert lediglich, ob die
                    // Einzeichen-Animation laeuft. Wuerde er weiterlaufen,
                    // loeste jeder Poll ein zusaetzliches Neu-Rendern aus.
                    if batches.get_untracked() < 2 {
                        set_batches.update(|b| *b += 1);
                    }
                    if stale.get_untracked() {
                        set_stale.set(false);
                    }
                }
                // Fehler bleiben nicht mehr unsichtbar: die Karte zeigt einen
                // Hinweis, statt stillschweigend alte Werte weiterzuzeigen.
                Err(_) => {
                    if !stale.get_untracked() {
                        set_stale.set(true);
                    }
                }
            }

            TimeoutFuture::new(2000).await;
        }
    });

    // Einzeichnen genau einmal, beim ersten eingetroffenen Datensatz. Der
    // Effekt laeuft nach dem Rendern; die JS-Seite wartet zusaetzlich einen
    // Frame ab, bevor sie die Pfade anfasst.
    Effect::new(move |_| {
        if batches.get() == 1 {
            let root = format!("#chart-{}", index);
            anim::draw_paths(&root, 0.12);
            anim::pop_dots(&root, 0.008);
        }
    });

    let chart_svg = move || {
        let logs = chart_logs.get();
        if logs.is_empty() {
            return view! {
                <div class="flex h-full flex-col justify-end gap-2 p-2">
                    // Skeleton statt reinem Text: signalisiert "es kommt noch was".
                    <div class="skeleton h-2/5 w-full rounded-lg"></div>
                    <div class="skeleton h-1/4 w-4/5 rounded-lg"></div>
                    <div class="skeleton h-3 w-24 rounded-full"></div>
                </div>
            }.into_any();
        }

        let metric = selected_metric.get();
        let get_val = |e: &LogEntry| -> Option<f64> {
            if metric == "temperature" { e.temperature } else { e.pressure }
        };
        let deselected = deselected_sensors.get();

        // Referenzen statt Clones: der Chart wird alle 2 Sekunden neu gebaut,
        // und jeder LogEntry enthaelt mehrere Strings.
        let mut grouped: HashMap<&str, Vec<&LogEntry>> = HashMap::new();
        for entry in &logs {
            grouped.entry(entry.sensor_name.as_str()).or_default().push(entry);
        }

        for data in grouped.values_mut() {
            data.sort_by_key(|e| e.timestamp);
        }

        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        let mut min_ts = u64::MAX;
        let mut max_ts = u64::MIN;

        for entry in &logs {
            if deselected.contains(&entry.sensor_name) { continue; }
            let Some(val) = get_val(entry) else { continue; };
            if val < min_val { min_val = val; }
            if val > max_val { max_val = val; }
            if entry.timestamp < min_ts { min_ts = entry.timestamp; }
            if entry.timestamp > max_ts { max_ts = entry.timestamp; }
        }

        if min_val == f64::INFINITY {
            min_val = 0.0;
            max_val = 10.0;
            if let Some(first) = logs.first() {
                min_ts = first.timestamp;
                max_ts = min_ts + 10;
            } else {
                min_ts = 0;
                max_ts = 10;
            }
        }

        if min_ts == max_ts {
            max_ts += 1;
        }

        let range_val = if (max_val - min_val).abs() < 0.001 { 1.0 } else { max_val - min_val };
        let y_padding_val = range_val * 0.1;
        let padded_min_val = min_val - y_padding_val;
        let padded_max_val = max_val + y_padding_val;
        let padded_range = padded_max_val - padded_min_val;

        // The viewBox grows with the card. preserveAspectRatio="none" stretches
        // everything inside it, so keeping the coordinate system proportional to
        // the rendered size is what stops labels and markers from ballooning
        // when the card is expanded.
        let is_expanded = expanded.get();
        let (width, height, padding) = if is_expanded {
            (1200.0, 460.0, 55.0)
        } else {
            (600.0, 220.0, 40.0)
        };
        let dot_r = if is_expanded { 4.0 } else { 3.5 };
        let hit_r = if is_expanded { 12.0 } else { 10.0 };
        let x_ticks: u64 = if is_expanded { 8 } else { 4 };

        let get_x = |ts: u64| {
            let normalized = (ts as f64 - min_ts as f64) / (max_ts as f64 - min_ts as f64);
            padding + normalized * (width - 2.0 * padding)
        };

        let get_y = |val: f64| {
            let normalized = (val - padded_min_val) / padded_range;
            (height - padding) - (normalized * (height - 2.0 * padding))
        };

        let colors = ["#2563eb", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899"];

        let mut paths = Vec::new();
        let mut legend = Vec::new();

        let mut sensor_names: Vec<&str> = grouped.keys().copied().collect();
        sensor_names.sort();

        for (i, sensor_name) in sensor_names.iter().enumerate() {
            let data = &grouped[sensor_name];
            let color = colors[i % colors.len()];
            let is_active = !deselected.contains(*sensor_name);
            // A tank reports only one of the two metrics, so it has nothing to
            // draw on the other tab.
            let has_data = data.iter().any(|e| get_val(e).is_some());

            let sensor_clone = sensor_name.to_string();
            let toggle_sensor = move |_| {
                set_deselected_sensors.update(|set| {
                    if set.contains(&sensor_clone) {
                        set.remove(&sensor_clone);
                    } else {
                        set.insert(sensor_clone.clone());
                    }
                });
            };

            let opacity = if !has_data {
                "opacity-25"
            } else if is_active {
                "opacity-100"
            } else {
                "opacity-40"
            };

            // Als <button>, nicht als <div>: so ist die Legende per Tab
            // erreichbar, mit Enter/Space schaltbar und `aria-pressed` teilt
            // Screenreadern den Zustand mit.
            legend.push(view! {
                <button
                    type="button"
                    class=format!("flex cursor-pointer items-center gap-1 rounded-md px-1 py-0.5 transition-all duration-200 hover:bg-gray-100 hover:scale-105 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-500 {}", opacity)
                    on:click=toggle_sensor
                    aria-pressed=if is_active { "true" } else { "false" }
                    disabled=!has_data
                    title=if has_data { String::new() } else { "Kein Messwert für diese Metrik".to_string() }
                >
                    <span
                        class="inline-block h-3 w-3 rounded-full transition-transform duration-200"
                        style=format!("background-color: {}", if is_active && has_data { color } else { "#9ca3af" })
                    ></span>
                    <span class="text-xs text-gray-600 select-none">{sensor_name.to_string()}</span>
                </button>
            });

            if !is_active || !has_data || data.is_empty() { continue; }

            let mut d = String::new();
            // Gaps in the series must break the line rather than connect across
            // a missing reading.
            let mut pen_down = false;

            for entry in data.iter() {
                let Some(val) = get_val(entry) else {
                    pen_down = false;
                    continue;
                };

                let x = get_x(entry.timestamp);
                let y = get_y(val);

                if pen_down {
                    d.push_str(&format!(" L {:.1} {:.1}", x, y));
                } else {
                    d.push_str(&format!(" M {:.1} {:.1}", x, y));
                    pen_down = true;
                }

                let point = HoveredPoint {
                    x,
                    y,
                    value: format!("{:.2}", val),
                    sensor: sensor_name.to_string(),
                    time: format_time(entry.timestamp),
                };

                // `data-anim` markiert die Punkte fuer GSAP; das Auf-Poppen
                // steuert jetzt popDots() statt einer CSS-Verzoegerung pro Punkt.
                paths.push(view! {
                    <g
                        on:mouseenter=move |_| set_hovered_point.set(Some(point.clone()))
                        on:mouseleave=move |_| set_hovered_point.set(None)
                        class="cursor-pointer"
                    >
                        <circle cx=x cy=y r=hit_r fill="transparent" />
                        <circle
                            cx=x
                            cy=y
                            r=dot_r
                            fill=color
                            data-anim="dot"
                            class="transition-all duration-150 hover:brightness-110"
                        />
                    </g>
                }.into_any());
            }

            // Das Einzeichnen uebernimmt drawPaths(): GSAP kann dort die echte
            // Pfadlaenge messen, statt sie wie die CSS-Variante zu raten.
            paths.push(view! {
                <path
                    d=d
                    fill="none"
                    stroke=color
                    stroke-width="2.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                    data-anim="line"
                />
            }.into_any());
        }

        let step = (max_ts - min_ts) / x_ticks;
        let mut x_axis_labels = Vec::new();
        if step > 0 {
            for i in 0..=x_ticks {
                let ts = min_ts + i * step;
                let x = get_x(ts);
                x_axis_labels.push(view! {
                    <text x=x y={height - 15.0} text-anchor="middle" font-size="10" fill="#6b7280">
                        {format_time(ts)}
                    </text>
                });
            }
        } else {
            x_axis_labels.push(view! {
                <text x={get_x(min_ts)} y={height - 15.0} text-anchor="middle" font-size="10" fill="#6b7280">
                    {format_time(min_ts)}
                </text>
            });
        }

        let y_axis_labels = vec![
            view! { <text x={padding - 5.0} y={get_y(padded_max_val) + 4.0} text-anchor="end" font-size="10" fill="#6b7280">{format!("{:.1}", padded_max_val)}</text> },
            view! { <text x={padding - 5.0} y={get_y(padded_min_val + padded_range / 2.0) + 4.0} text-anchor="end" font-size="10" fill="#6b7280">{format!("{:.1}", padded_min_val + padded_range / 2.0)}</text> },
            view! { <text x={padding - 5.0} y={get_y(padded_min_val) + 4.0} text-anchor="end" font-size="10" fill="#6b7280">{format!("{:.1}", padded_min_val)}</text> },
        ];

        let unit_suffix = if metric == "temperature" { "K" } else { "Bar" };

        let tooltip = move || {
            hovered_point.get().map(|p| {
                let x_pos = if p.x > width - 100.0 { p.x - 100.0 } else if p.x < 60.0 { p.x + 10.0 } else { p.x - 45.0 };
                let y_pos = if p.y < 50.0 { p.y + 20.0 } else { p.y - 45.0 };

                view! {
                    // Hier ist das Wiederholen der Animation erwuenscht: sie
                    // laeuft bei jedem Hover neu.
                    <g transform=format!("translate({}, {})", x_pos, y_pos) class="animate-pop-in origin-center">
                        <rect x="0" y="0" width="90" height="38" fill="#1f2937" rx="6" opacity="0.92" />
                        <text x="45" y="14" text-anchor="middle" font-size="11" fill="white" font-weight="bold">{format!("{} {}", p.value, unit_suffix)}</text>
                        <text x="45" y="24" text-anchor="middle" font-size="9" fill="#d1d5db">{p.sensor}</text>
                        <text x="45" y="33" text-anchor="middle" font-size="8" fill="#9ca3af">{p.time}</text>
                    </g>
                }
            })
        };

        view! {
            <div class="flex h-full w-full flex-col">
                // Hoehe begrenzt: ein Satellit mit einem Dutzend Sensoren
                // brauchte sonst fuenf Legendenzeilen und das Diagramm darunter
                // wurde in der Karte platt gequetscht.
                <div class="mb-2 flex max-h-[3.75rem] shrink-0 flex-wrap gap-x-3 gap-y-1 overflow-y-auto px-2">
                    {legend}
                </div>
                // Die id grenzt die GSAP-Aufrufe auf dieses Diagramm ein --
                // sonst wuerde eine Karte die Punkte aller anderen mit animieren.
                <div class="relative min-h-0 w-full flex-1" id=format!("chart-{}", index)>
                    <svg class="h-full w-full overflow-visible" viewBox=format!("0 0 {} {}", width, height) preserveAspectRatio="none">
                        <line x1={padding} y1={get_y(padded_max_val)} x2={width - padding} y2={get_y(padded_max_val)} stroke="#f3f4f6" stroke-width="1" />
                        <line x1={padding} y1={get_y(padded_min_val + padded_range / 2.0)} x2={width - padding} y2={get_y(padded_min_val + padded_range / 2.0)} stroke="#f3f4f6" stroke-width="1" />
                        <line x1={padding} y1={get_y(padded_min_val)} x2={width - padding} y2={get_y(padded_min_val)} stroke="#e5e7eb" stroke-width="1" />

                        {x_axis_labels}
                        {y_axis_labels}

                        {paths}
                        {tooltip}
                    </svg>
                </div>
            </div>
        }.into_any()
    };

    view! {
        // Both class variants are spelled out as whole literals so Tailwind's
        // source scanner can find them; building them by concatenation would
        // leave the utilities out of the generated stylesheet.
        <div
            // Der gestaffelte Auftritt kommt von GSAP (revealOnce). Wichtig ist
            // dessen clearProps: es raeumt transform/opacity danach wieder ab,
            // sonst blockiert das Inline-transform den hover:-translate-y-1.
            data-anim="reveal"
            class=move || if expanded.get() {
                "col-span-full flex h-[620px] flex-col relative overflow-hidden rounded-2xl border border-gray-200 bg-white p-5 shadow-md transition-all duration-500 ease-out"
            } else {
                "flex h-[340px] flex-col relative overflow-hidden rounded-2xl border border-gray-200 bg-white p-5 shadow-sm transition-all duration-500 ease-out hover:-translate-y-1 hover:border-blue-200 hover:shadow-lg"
            }
        >
            <div class="mb-4 flex items-center justify-between">
                <div class="space-y-1">
                    <div class="flex items-center gap-2">
                        <h2 class="text-sm font-bold text-gray-900">{format!("Satellit: {}", name)}</h2>
                        {move || stale.get().then(|| view! {
                            <span class="animate-pop-in rounded-full bg-amber-100 px-2 py-0.5 text-[10px] font-semibold text-amber-700">
                                "Verbindung…"
                            </span>
                        })}
                    </div>
                    <p class="text-xs text-gray-500">"Sensordaten"</p>
                </div>
                <div class="flex items-center gap-3">
                    <label class="sr-only" for=format!("viewport-{}", index)>"Anzahl der angezeigten Messwerte"</label>
                    <select
                        id=format!("viewport-{}", index)
                        on:change=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                set_viewport_size.set(val);
                            }
                        }
                        class="cursor-pointer rounded-lg border-none bg-gray-100 px-2 py-1.5 text-xs font-medium text-gray-700 transition-colors duration-200 hover:bg-gray-200 focus:ring-0"
                    >
                        <option value="10">"Letzte 10"</option>
                        <option value="25" selected=true>"Letzte 25"</option>
                        <option value="50">"Letzte 50"</option>
                        <option value="100">"Letzte 100"</option>
                    </select>
                    <div class="flex rounded-lg bg-gray-100 p-1 text-xs font-medium">
                        <button
                            on:click=move |_| set_selected_metric.set("temperature".to_string())
                            class=move || if selected_metric.get() == "temperature" {
                                "cursor-pointer rounded-md bg-white px-2.5 py-1.5 font-semibold text-gray-900 shadow-sm transition-all duration-200"
                            } else {
                                "cursor-pointer px-2.5 py-1.5 text-gray-600 transition-all duration-200 hover:text-gray-900"
                            }
                        >
                            "Temperatur"
                        </button>
                        <button
                            on:click=move |_| set_selected_metric.set("pressure".to_string())
                            class=move || if selected_metric.get() == "pressure" {
                                "cursor-pointer rounded-md bg-white px-2.5 py-1.5 font-semibold text-gray-900 shadow-sm transition-all duration-200"
                            } else {
                                "cursor-pointer px-2.5 py-1.5 text-gray-600 transition-all duration-200 hover:text-gray-900"
                            }
                        >
                            "Druck"
                        </button>
                    </div>
                    <button
                        on:click=move |_| set_expanded.update(|e| *e = !*e)
                        class="cursor-pointer rounded-lg bg-gray-100 px-2.5 py-1.5 text-sm leading-none font-medium text-gray-700 transition-all duration-200 hover:scale-110 hover:bg-gray-200 active:scale-95"
                        title=move || if expanded.get() { "Verkleinern" } else { "Vergrößern" }
                    >
                        {move || if expanded.get() { "⤡" } else { "⤢" }}
                    </button>
                </div>
            </div>
            <div class="relative h-full min-h-0 w-full flex-1 rounded-xl">
                {move || match sensors.get() {
                    // Kein <Suspense> mehr: hier steckt keine Resource drin, der
                    // Fallback waere also nie erschienen.
                    None => view! {
                        <div class="flex h-full flex-col justify-end gap-2 p-2">
                            <div class="skeleton h-2/5 w-full rounded-lg"></div>
                            <div class="skeleton h-1/4 w-4/5 rounded-lg"></div>
                        </div>
                    }.into_any(),
                    Some(s) if s.is_empty() => view! {
                        <div class="animate-fade-in flex h-full items-center justify-center text-xs text-gray-400">
                            "Keine Sensoren gefunden."
                        </div>
                    }.into_any(),
                    Some(_) => chart_svg(),
                }}
            </div>
        </div>
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (logs, set_logs) = signal(None::<SatelliteLogResponse>);
    let (anzahl_empfangen, set_anzahl_empfangen) = signal(0u64);
    let (satellites, set_satellites) = signal(Vec::<String>::new());
    let (sat_loaded, set_sat_loaded) = signal(false);
    let (conn, set_conn) = signal(Conn::Connecting);

    // Zusaetzliche Absicherung gegen unnoetiges Neu-Rendern: ein Memo meldet
    // sich nur, wenn sich der Wert per PartialEq wirklich unterscheidet.
    let sat_list = Memo::new(move |_| satellites.get());

    // Blendet Kopfbereich, Karten und Tabelle gestaffelt ein. Laeuft auch, wenn
    // spaeter ein Satellit dazukommt -- revealOnce merkt sich pro Element, was
    // es schon animiert hat, und laesst die bestehenden Karten in Ruhe.
    Effect::new(move |_| {
        let _ = sat_list.get();
        anim::reveal_once("[data-anim=\"reveal\"]", 0.09);
    });

    // Der Zaehler wird von GSAP hochgezaehlt. Leptos schreibt den Text deshalb
    // absichtlich *nicht* -- sonst wuerden Tween und Reaktivitaet sich
    // gegenseitig ueberschreiben.
    Effect::new(move |_| {
        let table_visible = logs.with(|l| l.is_some());
        let value = anzahl_empfangen.get();
        if table_visible {
            anim::count_to("#empfangen-count", value);
        }
    });

    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    // The satellite list must be polled rather than fetched once: a satellite
    // only appears in /satellites after its first measurement reaches the
    // database, and they do not all arrive in the same second.
    let alive_sats = alive.clone();
    spawn_local(async move {
        loop {
            if !alive_sats.load(Ordering::Relaxed) {
                break;
            }

            match fetch_satellites().await {
                Ok(names) => {
                    if !alive_sats.load(Ordering::Relaxed) {
                        break;
                    }
                    // Only publish real changes, otherwise every poll would churn
                    // the chart grid.
                    //
                    // Das gilt fuer *jedes* Signal hier: `set()` benachrichtigt
                    // in Leptos immer, auch wenn der Wert derselbe bleibt. Ein
                    // ungeprueftes `set_sat_loaded.set(true)` im Sekundentakt
                    // baute das Raster neu auf -- und damit auch jede
                    // SatelliteChart-Komponente, die dabei ihren Zustand
                    // (vergroessert, gewaehlte Metrik, abgewaehlte Sensoren)
                    // verlor.
                    if !sat_loaded.get_untracked() {
                        set_sat_loaded.set(true);
                    }
                    if conn.get_untracked() != Conn::Live {
                        set_conn.set(Conn::Live);
                    }
                    if names != satellites.get_untracked() {
                        set_satellites.set(names);
                    }
                }
                Err(_) => {
                    if conn.get_untracked() != Conn::Offline {
                        set_conn.set(Conn::Offline);
                    }
                }
            }

            TimeoutFuture::new(5000).await;
        }
    });

    let alive = alive.clone();
    spawn_local(async move {
        // Consecutive polls overlap heavily, so only genuinely new measurements
        // may raise the counter.
        //
        // Vorher lag dafuer jede je gesehene Messung in einem HashSet -- das
        // wuchs ueber eine offene Session unbegrenzt. Jetzt wird pro Sensor nur
        // der neueste Zeitstempel behalten (begrenzt durch die Sensorzahl) und
        // der Zaehler fortgeschrieben. Preis dafuer: eine verspaetet
        // eintreffende, aeltere Messung wird nicht mitgezaehlt.
        let mut last_seen: HashMap<(String, String), u64> = HashMap::new();
        let mut total: u64 = 0;

        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            match fetch_logs(15).await {
                Ok(data) => {
                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }

                    // Zeitstempel pro Sensor sammeln; begrenzt durch die
                    // Groesse einer Antwort.
                    let mut batch: HashMap<(&str, &str), Vec<u64>> = HashMap::new();
                    for entry in &data.data {
                        batch
                            .entry((entry.specs.name.as_str(), entry.sensor_name.as_str()))
                            .or_default()
                            .push(entry.timestamp);
                    }

                    for ((sat, sensor), mut timestamps) in batch {
                        timestamps.sort_unstable();
                        let key = (sat.to_string(), sensor.to_string());
                        let previous = last_seen.get(&key).copied();

                        total += match previous {
                            Some(last) => timestamps.iter().filter(|ts| **ts > last).count() as u64,
                            None => timestamps.len() as u64,
                        };

                        if let Some(&newest) = timestamps.last() {
                            last_seen.insert(key, newest.max(previous.unwrap_or(0)));
                        }
                    }

                    set_anzahl_empfangen.set(total);
                    set_logs.set(Some(data));
                    if conn.get_untracked() != Conn::Live {
                        set_conn.set(Conn::Live);
                    }
                }
                Err(_) => {
                    if conn.get_untracked() != Conn::Offline {
                        set_conn.set(Conn::Offline);
                    }
                }
            }

            TimeoutFuture::new(3000).await;
        }
    });

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div data-anim="reveal" class="flex flex-col gap-4 border-b border-gray-200 pt-2 pb-6 md:flex-row md:items-center md:justify-between">
                <div>
                    <div class="flex items-center gap-2">
                        <h1 class="text-sheen text-3xl font-bold tracking-tight">"Satelliten Dashboard"</h1>
                        // Der Indikator zeigt jetzt den echten Verbindungszustand.
                        {move || {
                            let (dot, ping, label, label_color) = match conn.get() {
                                Conn::Live => ("bg-emerald-500", "animate-live-ping bg-emerald-400", "Live", "text-emerald-600"),
                                Conn::Connecting => ("bg-amber-400", "animate-live-ping bg-amber-300", "Verbinde…", "text-amber-600"),
                                Conn::Offline => ("bg-red-500", "", "Offline", "text-red-600"),
                            };
                            view! {
                                <span class="relative flex h-2.5 w-2.5" aria-hidden="true">
                                    <span class=format!("absolute inline-flex h-full w-full rounded-full {}", ping)></span>
                                    <span class=format!("relative inline-flex h-2.5 w-2.5 rounded-full {}", dot)></span>
                                </span>
                                <span class=format!("text-xs font-semibold transition-colors duration-300 {}", label_color) role="status">
                                    {label}
                                </span>
                            }
                        }}
                    </div>
                </div>
            </div>

            <div class="my-8 flex flex-col gap-6">
                <div class="grid grid-cols-1 gap-6 md:grid-cols-2 xl:grid-cols-3">
                    // Bewusst drei getrennte Bloecke statt eines if/else:
                    // so haengt die Kartenliste ausschliesslich an
                    // `sat_list`. Lag alles in einer Closure, riss jede
                    // Aenderung des Ladezustands auch die Karten mit.
                    {move || (!sat_loaded.get()).then(|| {
                        // Platzhalterkarten, damit das Raster nicht leer wirkt.
                        // Ohne data-anim: die Platzhalter werden gleich durch die
                        // echten Karten ersetzt, die dann eingeblendet werden.
                        (0..3).map(|_| view! {
                            <div class="h-[340px] rounded-2xl border border-gray-200 bg-white p-5 shadow-sm">
                                <div class="skeleton mb-4 h-4 w-32 rounded-full"></div>
                                <div class="skeleton h-2/3 w-full rounded-lg"></div>
                            </div>
                        }).collect_view()
                    })}
                    {move || (sat_loaded.get() && sat_list.get().is_empty()).then(|| view! {
                        <div class="animate-fade-in col-span-full rounded-2xl border border-dashed border-gray-300 p-10 text-center text-gray-400">
                            "Noch keine Satelliten empfangen."
                        </div>
                    })}
                    {move || sat_list.get().into_iter().enumerate().map(|(i, s)| {
                        view! { <SatelliteChart name=s index=i /> }
                    }).collect_view()}
                </div>

                <div data-anim="reveal" class="min-h-64 w-full overflow-x-auto rounded-3xl border border-gray-200 bg-white p-6 shadow-sm">
                    <div class="mt-2 min-w-[600px]">
                        {move || match logs.get() {
                            None => view! {
                                <div class="space-y-3">
                                    <div class="skeleton mx-auto h-4 w-40 rounded-full"></div>
                                    {(0..5).map(|_| view! { <div class="skeleton h-8 w-full rounded-lg"></div> }).collect_view()}
                                </div>
                            }.into_any(),
                            Some(log_data) => view! {
                                <div class="space-y-4">
                                    <p class="text-center text-sm font-semibold text-gray-700">
                                        "Anzahl Empfangen: "
                                        // Startwert bewusst statisch: den Inhalt
                                        // schreibt ausschliesslich GSAP (countTo).
                                        <span id="empfangen-count" class="inline-block tabular-nums">"0"</span>
                                    </p>
                                    <table class="w-full text-left text-sm text-gray-600">
                                        <thead class="bg-gray-50 text-xs uppercase text-gray-700">
                                            <tr>
                                                <th class="rounded-l-lg px-3 py-2.5">"Zeitstempel"</th>
                                                <th class="px-3 py-2.5">"Satellit"</th>
                                                <th class="px-3 py-2.5">"Sensor"</th>
                                                <th class="px-3 py-2.5">"Temperatur"</th>
                                                <th class="rounded-r-lg px-3 py-2.5">"Druck"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-100">
                                            {log_data.data.into_iter().map(|entry| view! {
                                                <tr class="transition-colors duration-150 hover:bg-blue-50/60">
                                                    <td class="px-3 py-2.5 font-mono text-xs whitespace-nowrap text-gray-500">{format_date(entry.timestamp)}</td>
                                                    <td class="px-3 py-2.5 font-medium text-gray-900">{entry.specs.name}</td>
                                                    <td class="px-3 py-2.5 text-xs">{entry.sensor_name}</td>
                                                    <td class="px-3 py-2.5 whitespace-nowrap">
                                                        {entry.temperature.map(|t| format!("{:.2} K", t)).unwrap_or_else(|| "—".to_string())}
                                                    </td>
                                                    <td class="px-3 py-2.5 whitespace-nowrap">
                                                        {entry.pressure.map(|p| format!("{:.2} Bar", p)).unwrap_or_else(|| "—".to_string())}
                                                    </td>
                                                </tr>
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any(),
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
