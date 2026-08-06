use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub city: String,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Specs {
    pub name: String,
    pub model: String,
    pub launch_date: String,
    pub sensors: Vec<String>,
    pub nation: String,
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

/// Static description of a satellite, from GET /satellites/{name}. Note the
/// backend spells this field `launchdate`, unlike `launch_date` inside a log
/// entry's specs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SatelliteDetail {
    pub name: String,
    pub model: String,
    pub launchdate: String,
    pub sensors: Vec<String>,
    pub nation: String,
}

pub async fn fetch_satellite_detail(name: &str) -> Result<SatelliteDetail, gloo_net::Error> {
    let url = format!("{}/satellites/{}", API_BASE, name);
    Request::get(&url)
        .send()
        .await?
        .json::<SatelliteDetail>()
        .await
}

pub fn format_date(timestamp_sec: u64) -> String {
    let ms = (timestamp_sec * 1000) as f64;
    let date = js_sys::Date::new(&JsValue::from_f64(ms));
    date.to_locale_string("de-DE", &JsValue::UNDEFINED).into()
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
pub fn SatelliteChart(name: String) -> impl IntoView {
    let name_clone = name.clone();
    let sensors = LocalResource::new(move || {
        let name = name_clone.clone();
        async move { fetch_sensors(&name).await.unwrap_or_default() }
    });

    let (chart_logs, set_chart_logs) = signal(Vec::<LogEntry>::new());
    let (selected_metric, set_selected_metric) = signal("temperature".to_string());
    let (deselected_sensors, set_deselected_sensors) = signal(HashSet::<String>::new());

    let (hovered_point, set_hovered_point) = signal(None::<(f64, f64, String, String, String)>);
    let (viewport_size, set_viewport_size) = signal(25usize);
    let (expanded, set_expanded) = signal(false);

    // Used as the series label when the height metric is selected, since that
    // value describes the satellite rather than any single sensor.
    let chart_name = name.clone();

    // Without this the polling loop outlives the component: every visit to
    // /dashboard would leave another one running forever.
    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    let name_clone2 = name.clone();
    spawn_local(async move {
        let name = name_clone2;
        let mut sensor_count = 0usize;

        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            if sensor_count == 0 {
                if let Ok(s) = fetch_sensors(&name).await {
                    sensor_count = s.len();
                }
            }

            if sensor_count > 0 {
                // The endpoint caps total rows, not rows per sensor, so scale
                // the request by how many sensors share that budget.
                let amount = viewport_size.get_untracked() * sensor_count;
                if let Ok(data) = fetch_satellite_logs(&name, amount).await {
                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }
                    set_chart_logs.set(data.data);
                }
            }

            TimeoutFuture::new(2000).await;
        }
    });

    let format_time = |ts: u64| -> String {
        let ms = (ts * 1000) as f64;
        let date = js_sys::Date::new(&JsValue::from_f64(ms));
        let hours = date.get_hours();
        let minutes = date.get_minutes();
        let seconds = date.get_seconds();
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    };

    let chart_svg = move || {
        let logs = chart_logs.get();
        if logs.is_empty() {
            return view! { <div class="text-xs text-slate-500 flex items-center justify-center h-full">"Warte auf Daten..."</div> }.into_any();
        }

        let metric = selected_metric.get();
        // Height is a property of the satellite, not of one sensor: every sensor
        // repeats the same value, so it is drawn as one series and duplicate
        // timestamps are dropped instead of stacking identical lines.
        let is_height = metric == "height";
        let get_val = |e: &LogEntry| -> Option<f64> {
            match metric.as_str() {
                "temperature" => e.temperature,
                "pressure" => e.pressure,
                _ => Some(e.position.height),
            }
        };
        let deselected = deselected_sensors.get();

        let mut grouped: HashMap<String, Vec<LogEntry>> = HashMap::new();
        if is_height {
            let series = grouped.entry(chart_name.clone()).or_default();
            let mut seen_ts = HashSet::new();
            for entry in &logs {
                if seen_ts.insert(entry.timestamp) {
                    series.push(entry.clone());
                }
            }
        } else {
            for entry in &logs {
                grouped.entry(entry.sensor_name.clone()).or_default().push(entry.clone());
            }
        }

        for (_, data) in grouped.iter_mut() {
            data.sort_by_key(|e| e.timestamp);
        }

        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        let mut min_ts = u64::MAX;
        let mut max_ts = u64::MIN;

        // Scaled from the grouped series rather than the raw log, so the height
        // view is bounded by the deduplicated points it actually draws.
        for (key, data) in grouped.iter() {
            if !is_height && deselected.contains(key) { continue; }
            for entry in data {
                let Some(val) = get_val(entry) else { continue; };
                if val < min_val { min_val = val; }
                if val > max_val { max_val = val; }
                if entry.timestamp < min_ts { min_ts = entry.timestamp; }
                if entry.timestamp > max_ts { max_ts = entry.timestamp; }
            }
        }

        if min_val == f64::INFINITY {
            min_val = 0.0;
            max_val = 10.0;
            if logs.first().is_some() {
                min_ts = logs.first().unwrap().timestamp;
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

        let colors = ["#60a5fa", "#f87171", "#34d399", "#fbbf24", "#a78bfa", "#f472b6"];
        // sky-400 rather than sky-500, to sit in the same brightness band as the
        // sensor colours above on the dark background.
        let height_color = "#38bdf8";

        // Two decimals are meaningful for pressure but would overflow the
        // tooltip for a value like JWST's 1500012.34 km.
        let fmt_value = |v: f64| if v.abs() >= 10000.0 {
            format!("{:.0}", v)
        } else {
            format!("{:.2}", v)
        };

        let mut paths = Vec::new();
        let mut legend = Vec::new();

        let mut sensor_names: Vec<String> = grouped.keys().cloned().collect();
        sensor_names.sort();

        for (i, sensor_name) in sensor_names.iter().enumerate() {
            let data = &grouped[sensor_name];
            let color = if is_height { height_color } else { colors[i % colors.len()] };
            // The single height series has nothing to toggle against.
            let is_active = is_height || !deselected.contains(sensor_name);
            // A tank reports only one of the two metrics, so it has nothing to
            // draw on the other tab.
            let has_data = data.iter().any(|e| get_val(e).is_some());

            if is_height {
                legend.push(view! {
                    <div class="flex items-center gap-1">
                        <span class="w-3 h-3 rounded-full inline-block" style=format!("background-color: {}", color)></span>
                        <span class="text-xs text-slate-400 select-none">"Bahnhöhe"</span>
                    </div>
                }.into_any());
            } else {
                let sensor_clone = sensor_name.clone();
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

                legend.push(view! {
                    <div
                        class=format!("flex items-center gap-1 cursor-pointer transition-opacity {}", opacity)
                        on:click=toggle_sensor
                        title=if has_data { String::new() } else { "Kein Messwert für diese Metrik".to_string() }
                    >
                        <span class="w-3 h-3 rounded-full inline-block" style=format!("background-color: {}", if is_active && has_data { color } else { "#64748b" })></span>
                        <span class="text-xs text-slate-400 hover:text-slate-100 select-none">{sensor_name.clone()}</span>
                    </div>
                }.into_any());
            }

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

                let val_str = fmt_value(val);
                let sensor_c = if is_height { entry.position.city.clone() } else { sensor_name.clone() };
                let time_c = format_time(entry.timestamp);

                paths.push(view! {
                    <g
                        on:mouseenter=move |_| set_hovered_point.set(Some((x, y, val_str.clone(), sensor_c.clone(), time_c.clone())))
                        on:mouseleave=move |_| set_hovered_point.set(None)
                        class="cursor-pointer"
                    >
                        <circle cx=x cy=y r=hit_r fill="transparent" />
                        <circle cx=x cy=y r=dot_r fill=color class="transition-all hover:opacity-80" />
                    </g>
                }.into_any());
            }

            paths.push(view! {
                <path d=d fill="none" stroke=color stroke-width="2.5" stroke-linejoin="round" />
            }.into_any());
        }

        let step = (max_ts - min_ts) / x_ticks;
        let mut x_axis_labels = Vec::new();
        if step > 0 {
            for i in 0..=x_ticks {
                let ts = min_ts + i * step;
                let x = get_x(ts);
                x_axis_labels.push(view! {
                    <text x=x y={height - 15.0} text-anchor="middle" font-size="10" fill="#94a3b8">
                        {format_time(ts)}
                    </text>
                });
            }
        } else {
            x_axis_labels.push(view! {
                <text x={get_x(min_ts)} y={height - 15.0} text-anchor="middle" font-size="10" fill="#94a3b8">
                    {format_time(min_ts)}
                </text>
            });
        }

        // JWST orbits at 1.5 million km, where a decimal place is only noise and
        // would push the label past the left padding.
        let fmt_axis = |v: f64| if v.abs() >= 1000.0 {
            format!("{:.0}", v)
        } else {
            format!("{:.1}", v)
        };

        let y_axis_labels = vec![
            view! { <text x={padding - 5.0} y={get_y(padded_max_val) + 4.0} text-anchor="end" font-size="10" fill="#94a3b8">{fmt_axis(padded_max_val)}</text> },
            view! { <text x={padding - 5.0} y={get_y(padded_min_val + padded_range / 2.0) + 4.0} text-anchor="end" font-size="10" fill="#94a3b8">{fmt_axis(padded_min_val + padded_range / 2.0)}</text> },
            view! { <text x={padding - 5.0} y={get_y(padded_min_val) + 4.0} text-anchor="end" font-size="10" fill="#94a3b8">{fmt_axis(padded_min_val)}</text> },
        ];

        let unit_suffix = match metric.as_str() {
            "temperature" => "K",
            "pressure" => "Bar",
            _ => "km",
        };

        let tooltip = move || {
            hovered_point.get().map(|(x, y, val_str, sensor_name, time_str)| {
                let x_pos = if x > width - 100.0 { x - 100.0 } else if x < 60.0 { x + 10.0 } else { x - 45.0 };
                let y_pos = if y < 50.0 { y + 20.0 } else { y - 45.0 };

                view! {
                    <g transform=format!("translate({}, {})", x_pos, y_pos)>
                        <rect x="0" y="0" width="90" height="38" fill="#334155" stroke="#64748b" stroke-width="0.5" rx="4" opacity="0.97" filter="drop-shadow(0 4px 6px rgb(0 0 0 / 0.4))" />
                        <text x="45" y="14" text-anchor="middle" font-size="11" fill="#f8fafc" font-weight="bold">{format!("{} {}", val_str, unit_suffix)}</text>
                        <text x="45" y="24" text-anchor="middle" font-size="9" fill="#cbd5e1">{sensor_name}</text>
                        <text x="45" y="33" text-anchor="middle" font-size="8" fill="#94a3b8">{time_str}</text>
                    </g>
                }
            })
        };

        view! {
            <div class="flex flex-col w-full h-full">
                <div class="flex flex-wrap gap-3 mb-2 px-2">
                    {legend}
                </div>
                <div class="relative w-full flex-1 min-h-0">
                    <svg class="w-full h-full overflow-visible" viewBox=format!("0 0 {} {}", width, height) preserveAspectRatio="none">
                        <line x1={padding} y1={get_y(padded_max_val)} x2={width - padding} y2={get_y(padded_max_val)} stroke="#1e293b" stroke-width="1" />
                        <line x1={padding} y1={get_y(padded_min_val + padded_range / 2.0)} x2={width - padding} y2={get_y(padded_min_val + padded_range / 2.0)} stroke="#1e293b" stroke-width="1" />
                        <line x1={padding} y1={get_y(padded_min_val)} x2={width - padding} y2={get_y(padded_min_val)} stroke="#334155" stroke-width="1" />

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
        <div class=move || if expanded.get() {
            "p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-sm flex flex-col relative overflow-hidden transition-all col-span-full h-[620px]"
        } else {
            "p-5 rounded-2xl bg-slate-900 border border-slate-800 shadow-sm flex flex-col relative overflow-hidden transition-all h-[340px]"
        }>


            // Wraps rather than overflowing: the card is only about a third of the
            // grid wide and clips its content, which would swallow the last
            // control in the row.
            <div class="flex flex-wrap items-start justify-between gap-x-3 gap-y-2 mb-4">
                <div class="space-y-1 min-w-0">
                    <h2 class="text-sm font-bold text-slate-100 truncate">{format!("Satellit: {}", name)}</h2>
                    <p class="text-xs text-slate-400 truncate">
                        {move || chart_logs.get()
                            .iter()
                            .max_by_key(|e| e.timestamp)
                            .map(|e| format!("Über {} · {:.1} km", e.position.city, e.position.height))
                            .unwrap_or_else(|| "Sensordaten".to_string())}
                    </p>
                </div>
                <div class="flex items-center gap-2 shrink-0 ml-auto">
                    <select
                        on:change=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                set_viewport_size.set(val);
                            }
                        }
                        title="Anzahl der angezeigten Messpunkte"
                        class="px-2 py-1.5 rounded-lg bg-slate-800 text-slate-200 text-xs font-medium border-none cursor-pointer focus:ring-0"
                    >
                        <option value="10">"10"</option>
                        <option value="25" selected=true>"25"</option>
                        <option value="50">"50"</option>
                        <option value="100">"100"</option>
                    </select>
                    <div class="flex rounded-lg bg-slate-800 p-1 text-xs font-medium">
                    <button
                        on:click=move |_| set_selected_metric.set("temperature".to_string())
                        title="Temperatur"
                        class=move || if selected_metric.get() == "temperature" {
                            "px-2 py-1.5 rounded-md bg-slate-600 text-white shadow-sm font-semibold"
                        } else {
                            "px-2 py-1.5 text-slate-400 hover:text-slate-100 transition"
                        }
                    >
                        "Temp."
                    </button>
                    <button
                        on:click=move |_| set_selected_metric.set("pressure".to_string())
                        title="Druck"
                        class=move || if selected_metric.get() == "pressure" {
                            "px-2 py-1.5 rounded-md bg-slate-600 text-white shadow-sm font-semibold"
                        } else {
                            "px-2 py-1.5 text-slate-400 hover:text-slate-100 transition"
                        }
                    >
                        "Druck"
                    </button>
                    <button
                        on:click=move |_| set_selected_metric.set("height".to_string())
                        title="Bahnhöhe"
                        class=move || if selected_metric.get() == "height" {
                            "px-2 py-1.5 rounded-md bg-slate-600 text-white shadow-sm font-semibold"
                        } else {
                            "px-2 py-1.5 text-slate-400 hover:text-slate-100 transition"
                        }
                    >
                        "Höhe"
                    </button>
                </div>
                    <button
                        on:click=move |_| set_expanded.update(|e| *e = !*e)
                        class="px-2.5 py-1.5 rounded-lg bg-slate-800 text-slate-300 text-sm leading-none font-medium hover:bg-slate-700 transition cursor-pointer shrink-0"
                        title=move || if expanded.get() { "Verkleinern" } else { "Vergrößern" }
                    >
                        {move || if expanded.get() { "⤡" } else { "⤢" }}
                    </button>
                </div>
            </div>
            <div class="flex-1 w-full h-full min-h-0 rounded-xl relative">
                <Suspense fallback=move || view!{ <div class="text-xs text-slate-500 flex items-center justify-center h-full">"Lade Sensoren..."</div> }>
                    {move || {
                        let sens = sensors.get();
                        match sens {
                            None => view! { <div class="text-xs text-slate-500 flex items-center justify-center h-full">"Warte auf Daten..."</div> }.into_any(),
                            Some(sens) if sens.is_empty() => view! { <div class="text-xs text-slate-500 flex items-center justify-center h-full">"Keine Sensoren gefunden."</div> }.into_any(),
                            Some(_) => chart_svg()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let (logs, set_logs) = signal(None::<SatelliteLogResponse>);
    let (anzahl_empfangen, set_anzahl_empfangen) = signal(0usize);
    let (satellites, set_satellites) = signal(Vec::<String>::new());
    let (sat_loaded, set_sat_loaded) = signal(false);

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

            if let Ok(names) = fetch_satellites().await {
                if !alive_sats.load(Ordering::Relaxed) {
                    break;
                }
                set_sat_loaded.set(true);
                // Only publish real changes, otherwise every poll would churn
                // the chart grid.
                if names != satellites.get_untracked() {
                    set_satellites.set(names);
                }
            }

            TimeoutFuture::new(5000).await;
        }
    });

    let alive = alive.clone();
    spawn_local(async move {
        // Consecutive polls overlap heavily, so count distinct measurements
        // instead of adding the response size every time.
        let mut seen: HashSet<(String, String, u64)> = HashSet::new();

        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(data) = fetch_logs(15).await {
                if !alive.load(Ordering::Relaxed) {
                    break;
                }
                for entry in &data.data {
                    seen.insert((entry.specs.name.clone(), entry.sensor_name.clone(), entry.timestamp));
                }
                set_anzahl_empfangen.set(seen.len());
                set_logs.set(Some(data));
            }

            TimeoutFuture::new(3000).await;
        }
    });

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-b border-slate-800 pb-6 pt-2">
                <div>
                    <div class="flex items-center align-center">
                        <h1 class="text-3xl font-bold text-slate-100 tracking-tight">"Satelliten Dashboard"</h1>
                        <span class="w-2 h-2 ml-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    </div>
                </div>
            </div>

            <div class="flex flex-col gap-6 my-8">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                    <Suspense fallback=move || view!{ <div class="col-span-full text-slate-500">"Lade Satelliten..."</div> }>
                        {move || {
                            let sats = satellites.get();
                            if sats.is_empty() {
                                // Deckt beide Faelle ab: noch nicht geladen und
                                // wirklich keine Satelliten vorhanden.
                                view! { <div class="col-span-full text-slate-500">"Noch keine Satelliten empfangen."</div> }.into_any()
                            } else {
                                sats.into_iter().map(|s| {
                                    view! { <SatelliteChart name=s /> }
                                }).collect_view().into_any()
                            }
                        }}
                    </Suspense>
                </div>

                <div class="w-full p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-sm min-h-64 overflow-x-auto">
                    <div class="mt-2 min-w-[600px]">
                        {move || match logs.get() {
                            None => view! { <p class="text-slate-400">"Lade Daten..."</p> }.into_any(),
                            Some(log_data) => view! {
                                <div class="space-y-4">
                                    <p class="font-semibold text-slate-300 text-center text-sm">"Anzahl Empfangen: " {anzahl_empfangen}</p>
                                    <table class="w-full text-left text-sm text-slate-400">
                                        <thead class="bg-slate-800 text-slate-300 uppercase text-xs">
                                            <tr>
                                                <th class="py-2.5 px-3 rounded-l-lg">"Zeitstempel"</th>
                                                <th class="py-2.5 px-3">"Satellit"</th>
                                                <th class="py-2.5 px-3">"Sensor"</th>
                                                <th class="py-2.5 px-3">"Temperatur"</th>
                                                <th class="py-2.5 px-3">"Druck"</th>
                                                <th class="py-2.5 px-3">"Position"</th>
                                                <th class="py-2.5 px-3 rounded-r-lg">"Höhe"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-slate-800">
                                            {log_data.data.into_iter().map(|entry| view! {
                                                <tr class="hover:bg-slate-800/60 transition">
                                                    <td class="py-2.5 px-3 font-mono text-xs text-slate-400 whitespace-nowrap">{format_date(entry.timestamp)}</td>
                                                    <td class="py-2.5 px-3 font-medium text-slate-100">{entry.specs.name}</td>
                                                    <td class="py-2.5 px-3 text-xs">{entry.sensor_name}</td>
                                                    <td class="py-2.5 px-3 whitespace-nowrap">
                                                        {entry.temperature.map(|t| format!("{:.2} K", t)).unwrap_or_else(|| "—".to_string())}
                                                    </td>
                                                    <td class="py-2.5 px-3 whitespace-nowrap">
                                                        {entry.pressure.map(|p| format!("{:.2} Bar", p)).unwrap_or_else(|| "—".to_string())}
                                                    </td>
                                                    <td class="py-2.5 px-3 text-xs whitespace-nowrap">{entry.position.city}</td>
                                                    <td class="py-2.5 px-3 tabular-nums whitespace-nowrap">
                                                        {format!("{:.1} km", entry.position.height)}
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