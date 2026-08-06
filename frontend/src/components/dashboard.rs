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

fn format_date(timestamp_sec: u64) -> String {
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
            return view! { <div class="text-xs text-gray-400 flex items-center justify-center h-full">"Warte auf Daten..."</div> }.into_any();
        }

        let metric = selected_metric.get();
        let get_val = |e: &LogEntry| -> Option<f64> {
            if metric == "temperature" { e.temperature } else { e.pressure }
        };
        let deselected = deselected_sensors.get();

        let mut grouped: HashMap<String, Vec<LogEntry>> = HashMap::new();
        for entry in &logs {
            grouped.entry(entry.sensor_name.clone()).or_default().push(entry.clone());
        }

        for (_, data) in grouped.iter_mut() {
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

        let colors = ["#2563eb", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899"];

        let mut paths = Vec::new();
        let mut legend = Vec::new();

        let mut sensor_names: Vec<String> = grouped.keys().cloned().collect();
        sensor_names.sort();

        for (i, sensor_name) in sensor_names.iter().enumerate() {
            let data = &grouped[sensor_name];
            let color = colors[i % colors.len()];
            let is_active = !deselected.contains(sensor_name);
            // A tank reports only one of the two metrics, so it has nothing to
            // draw on the other tab.
            let has_data = data.iter().any(|e| get_val(e).is_some());

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
                    <span class="w-3 h-3 rounded-full inline-block" style=format!("background-color: {}", if is_active && has_data { color } else { "#9ca3af" })></span>
                    <span class="text-xs text-gray-600 hover:text-gray-900 select-none">{sensor_name.clone()}</span>
                </div>
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

                let val_str = format!("{:.2}", val);
                let sensor_c = sensor_name.clone();
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
            hovered_point.get().map(|(x, y, val_str, sensor_name, time_str)| {
                let x_pos = if x > width - 100.0 { x - 100.0 } else if x < 60.0 { x + 10.0 } else { x - 45.0 };
                let y_pos = if y < 50.0 { y + 20.0 } else { y - 45.0 };

                view! {
                    <g transform=format!("translate({}, {})", x_pos, y_pos)>
                        <rect x="0" y="0" width="90" height="38" fill="#1f2937" rx="4" opacity="0.9" filter="drop-shadow(0 4px 3px rgb(0 0 0 / 0.07))" />
                        <text x="45" y="14" text-anchor="middle" font-size="11" fill="white" font-weight="bold">{format!("{} {}", val_str, unit_suffix)}</text>
                        <text x="45" y="24" text-anchor="middle" font-size="9" fill="#d1d5db">{sensor_name}</text>
                        <text x="45" y="33" text-anchor="middle" font-size="8" fill="#9ca3af">{time_str}</text>
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
        <div class=move || if expanded.get() {
            "p-5 rounded-2xl bg-white border border-gray-200 shadow-sm flex flex-col relative overflow-hidden transition-all col-span-full h-[620px]"
        } else {
            "p-5 rounded-2xl bg-white border border-gray-200 shadow-sm flex flex-col relative overflow-hidden transition-all h-[340px]"
        }>


            <div class="flex items-center justify-between mb-4">
                <div class="space-y-1">
                    <h2 class="text-sm font-bold text-gray-900">{format!("Satellit: {}", name)}</h2>
                    <p class="text-xs text-gray-500">"Sensordaten"</p>
                </div>
                <div class="flex items-center gap-3">
                    <select
                        on:change=move |ev| {
                            if let Ok(val) = event_target_value(&ev).parse::<usize>() {
                                set_viewport_size.set(val);
                            }
                        }
                        class="px-2 py-1.5 rounded-lg bg-gray-100 text-gray-700 text-xs font-medium border-none cursor-pointer focus:ring-0"
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
                            "px-2.5 py-1.5 rounded-md bg-white text-gray-900 shadow-sm font-semibold"
                        } else {
                            "px-2.5 py-1.5 text-gray-600 hover:text-gray-900 transition"
                        }
                    >
                        "Temperatur"
                    </button>
                    <button
                        on:click=move |_| set_selected_metric.set("pressure".to_string())
                        class=move || if selected_metric.get() == "pressure" {
                            "px-2.5 py-1.5 rounded-md bg-white text-gray-900 shadow-sm font-semibold"
                        } else {
                            "px-2.5 py-1.5 text-gray-600 hover:text-gray-900 transition"
                        }
                    >
                        "Druck"
                    </button>
                </div>
                    <button
                        on:click=move |_| set_expanded.update(|e| *e = !*e)
                        class="px-2.5 py-1.5 rounded-lg bg-gray-100 text-gray-700 text-sm leading-none font-medium hover:bg-gray-200 transition cursor-pointer"
                        title=move || if expanded.get() { "Verkleinern" } else { "Vergrößern" }
                    >
                        {move || if expanded.get() { "⤡" } else { "⤢" }}
                    </button>
                </div>
            </div>
            <div class="flex-1 w-full h-full min-h-0 rounded-xl relative">
                <Suspense fallback=move || view!{ <div class="text-xs text-gray-400 flex items-center justify-center h-full">"Lade Sensoren..."</div> }>
                    {move || {
                        let sens = sensors.get();
                        match sens {
                            None => view! { <div class="text-xs text-gray-400 flex items-center justify-center h-full">"Warte auf Daten..."</div> }.into_any(),
                            Some(sens) if sens.is_empty() => view! { <div class="text-xs text-gray-400 flex items-center justify-center h-full">"Keine Sensoren gefunden."</div> }.into_any(),
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
            <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-b border-gray-200 pb-6 pt-2">
                <div>
                    <div class="flex items-center align-center">
                        <h1 class="text-3xl font-bold text-gray-900 tracking-tight">"Satelliten Dashboard"</h1>
                        <span class="w-2 h-2 ml-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    </div>
                </div>
            </div>

            <div class="flex flex-col gap-6 my-8">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                    {move || (!sat_loaded.get()).then(|| view! {
                        <div class="col-span-full text-gray-400">"Lade Satelliten..."</div>
                    })}
                    {move || (sat_loaded.get() && satellites.get().is_empty()).then(|| view! {
                        <div class="col-span-full text-gray-400">"Noch keine Satelliten empfangen."</div>
                    })}
                    // Keyed, so a newly appearing satellite mounts one extra
                    // chart instead of rebuilding the ones already running.
                    <For
                        each=move || satellites.get()
                        key=|name: &String| name.clone()
                        children=move |name: String| view! { <SatelliteChart name=name /> }
                    />
                </div>

                <div class="w-full p-6 rounded-3xl bg-white border border-gray-200 shadow-sm min-h-64 overflow-x-auto">
                    <div class="mt-2 min-w-[600px]">
                        {move || match logs.get() {
                            None => view! { <p class="text-gray-500">"Lade Daten..."</p> }.into_any(),
                            Some(log_data) => view! {
                                <div class="space-y-4">
                                    <p class="font-semibold text-gray-700 text-center text-sm">"Anzahl Empfangen: " {anzahl_empfangen}</p>
                                    <table class="w-full text-left text-sm text-gray-600">
                                        <thead class="bg-gray-50 text-gray-700 uppercase text-xs">
                                            <tr>
                                                <th class="py-2.5 px-3 rounded-l-lg">"Zeitstempel"</th>
                                                <th class="py-2.5 px-3">"Satellit"</th>
                                                <th class="py-2.5 px-3">"Sensor"</th>
                                                <th class="py-2.5 px-3">"Temperatur"</th>
                                                <th class="py-2.5 px-3 rounded-r-lg">"Druck"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-100">
                                            {log_data.data.into_iter().map(|entry| view! {
                                                <tr class="hover:bg-gray-50/80 transition">
                                                    <td class="py-2.5 px-3 font-mono text-xs text-gray-500 whitespace-nowrap">{format_date(entry.timestamp)}</td>
                                                    <td class="py-2.5 px-3 font-medium text-gray-900">{entry.specs.name}</td>
                                                    <td class="py-2.5 px-3 text-xs">{entry.sensor_name}</td>
                                                    <td class="py-2.5 px-3 whitespace-nowrap">
                                                        {entry.temperature.map(|t| format!("{:.2} K", t)).unwrap_or_else(|| "—".to_string())}
                                                    </td>
                                                    <td class="py-2.5 px-3 whitespace-nowrap">
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