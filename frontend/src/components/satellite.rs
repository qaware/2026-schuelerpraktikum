use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::anim;
use crate::components::dashboard::{
    fetch_satellite_detail, fetch_satellite_logs, fetch_satellites, format_date, LogEntry,
    SatelliteDetail,
};

/// Vier Bilder, beliebig viele Satelliten -- daher zyklisch zugeordnet.
const IMAGES: [&str; 4] = ["sat1", "sat2", "sat3", "sat4"];

/// A measurement older than this counts as loss of signal.
const STALE_AFTER_SECONDS: u64 = 30;

fn now_seconds() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

/// Ein Datenfeld in der Spec-Tabelle.
#[component]
fn SpecRow(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex justify-between items-baseline gap-4 py-1.5 border-b border-slate-800 last:border-b-0">
            <dt class="text-sm text-slate-400 shrink-0">{label}</dt>
            <dd class="text-sm font-medium text-slate-100 text-right break-all">{value}</dd>
        </div>
    }
}

/// How many of the newest measurements the height trend is derived from.
const TREND_WINDOW: usize = 30;

/// Describes how the orbit height moved across the polled window.
fn height_trend(entries: &[LogEntry]) -> Option<(&'static str, String, &'static str)> {
    let first = entries.first()?;
    let last = entries.last()?;
    let delta = last.position.height - first.position.height;

    // Below this the change is indistinguishable from the sensor noise the
    // generator adds to every reading.
    if delta.abs() < 0.5 {
        return Some(("→", "stabil".to_string(), "text-slate-500"));
    }
    if delta > 0.0 {
        Some(("↑", format!("+{:.2} km", delta), "text-emerald-400"))
    } else {
        Some(("↓", format!("{:.2} km", delta), "text-amber-400"))
    }
}

#[component]
fn SatelliteCard(name: String, index: usize) -> impl IntoView {
    let (detail, set_detail) = signal(None::<SatelliteDetail>);
    // A short window instead of a single row, so the height trend has something
    // to compare the newest value against.
    let (recent, set_recent) = signal(Vec::<LogEntry>::new());
    let latest = Memo::new(move |_| recent.get().last().cloned());

    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    let poll_name = name.clone();
    spawn_local(async move {
        // The specs never change, so they are fetched once and then only the
        // newest measurement is refreshed.
        let mut have_detail = false;

        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            if !have_detail {
                if let Ok(d) = fetch_satellite_detail(&poll_name).await {
                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }
                    set_detail.set(Some(d));
                    have_detail = true;
                }
            }

            if let Ok(res) = fetch_satellite_logs(&poll_name, TREND_WINDOW).await {
                if !alive.load(Ordering::Relaxed) {
                    break;
                }
                set_recent.set(res.data);
            }

            TimeoutFuture::new(3000).await;
        }
    });

    let heading = name.clone();
    let alt_text = name.clone();
    let image = IMAGES[index % IMAGES.len()];

    view! {
        <div
            data-anim="reveal"
            class="group my-8 w-full overflow-hidden rounded-3xl border border-slate-800 bg-slate-900 p-6 shadow-sm transition-all duration-300 hover:-translate-y-1 hover:border-blue-500/40 hover:shadow-lg"
        >
            <div class="grid grid-cols-1 md:grid-cols-2 md:divide-x divide-slate-800">
                <div class="p-4 min-h-48 flex justify-center items-center">
                    <img
                        src=format!("/public/{}.png", image)
                        alt=alt_text
                        class="animate-float h-48 rounded-xl object-cover transition-transform duration-500 ease-out group-hover:scale-105"
                        // Versetzt, damit nicht alle Karten im Gleichtakt schweben.
                        style=format!("animation-delay: {}ms", index * 900)
                    />
                </div>

                <div class="p-4 min-h-48">
                    <div class="flex items-center justify-between gap-3 mb-4">
                        <div class="flex items-center gap-2 min-w-0">
                            <h2 class="text-2xl font-semibold text-slate-200 truncate">{heading}</h2>
                            {move || detail.get().map(|d| view! {
                                <span class="shrink-0 rounded-full bg-blue-500/15 px-2 py-0.5 text-[10px] font-semibold tracking-wide text-blue-300 uppercase">
                                    {d.nation}
                                </span>
                            })}
                        </div>
                        {move || {
                            let entry = latest.get();
                            let fresh = entry
                                .as_ref()
                                .map(|e| now_seconds().saturating_sub(e.timestamp) < STALE_AFTER_SECONDS)
                                .unwrap_or(false);

                            if fresh {
                                view! {
                                    <span class="flex shrink-0 items-center gap-1.5 text-xs font-medium text-emerald-400">
                                        <span class="relative flex h-2 w-2" aria-hidden="true">
                                            <span class="animate-live-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400"></span>
                                            <span class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
                                        </span>
                                        "Signal"
                                    </span>
                                }.into_any()
                            } else {
                                view! {
                                    <span class="flex shrink-0 items-center gap-1.5 text-xs font-medium text-amber-400">
                                        <span class="w-2 h-2 rounded-full bg-amber-500"></span>
                                        "Kein Signal"
                                    </span>
                                }.into_any()
                            }
                        }}
                    </div>

                    {move || match detail.get() {
                        None => view! {
                            <div class="space-y-3">
                                {(0..5).map(|_| view! {
                                    <div class="skeleton h-4 w-full rounded-full"></div>
                                }).collect_view()}
                            </div>
                        }.into_any(),
                        Some(d) => {
                            let entry = latest.get();
                            let sensor_count = d.sensors.len();
                            let trend = height_trend(&recent.get());
                            let height_value = entry.as_ref()
                                .map(|e| format!("{:.2} km", e.position.height))
                                .unwrap_or_else(|| "—".to_string());

                            view! {
                                <dl class="mb-4">
                                    <SpecRow label="Modell" value=d.model.clone() />
                                    <SpecRow label="Nation" value=d.nation.clone() />
                                    <SpecRow label="Startdatum" value=d.launchdate.clone() />
                                    <SpecRow label="Sensoren" value=sensor_count.to_string() />
                                    <SpecRow
                                        label="Überflug"
                                        value=entry.as_ref()
                                            .map(|e| e.position.city.clone())
                                            .unwrap_or_else(|| "—".to_string())
                                    />
                                    // Spelled out instead of reusing SpecRow, because the
                                    // trend badge needs its own colour per direction.
                                    <div class="flex justify-between items-baseline gap-4 py-1.5 border-b border-slate-800 last:border-b-0">
                                        <dt class="text-sm text-slate-400 shrink-0">"Höhe"</dt>
                                        <dd class="flex items-baseline justify-end gap-2 text-right">
                                            {trend.map(|(arrow, label, color)| view! {
                                                <span class=format!("text-xs font-medium shrink-0 {}", color)>
                                                    {format!("{} {}", arrow, label)}
                                                </span>
                                            })}
                                            <span class="text-sm font-medium text-slate-100 tabular-nums">
                                                {height_value}
                                            </span>
                                        </dd>
                                    </div>
                                    <SpecRow
                                        label="Letzter Kontakt"
                                        value=entry.as_ref()
                                            .map(|e| format_date(e.timestamp))
                                            .unwrap_or_else(|| "—".to_string())
                                    />
                                    <SpecRow
                                        label="Letzte Messung"
                                        value=entry.as_ref()
                                            .map(|e| {
                                                // A tank reports only one of the two probes.
                                                let temp = e.temperature
                                                    .map(|t| format!("{:.2} K", t))
                                                    .unwrap_or_else(|| "—".to_string());
                                                let pres = e.pressure
                                                    .map(|p| format!("{:.2} Bar", p))
                                                    .unwrap_or_else(|| "—".to_string());
                                                format!("{} · {} · {}", e.sensor_name, temp, pres)
                                            })
                                            .unwrap_or_else(|| "—".to_string())
                                    />
                                </dl>

                                <div>
                                    <p class="text-xs uppercase tracking-wide text-slate-400 mb-2">"Sensoren"</p>
                                    <div class="flex flex-wrap gap-1.5">
                                        {d.sensors.iter().map(|sensor| {
                                            // Tanks und Thruster optisch trennen, damit die
                                            // Liste bei einem Dutzend Sensoren lesbar bleibt.
                                            let chip = if sensor.contains("tank") {
                                                "rounded-md bg-amber-500/15 px-2 py-1 font-mono text-[11px] text-amber-300 transition-transform duration-200 hover:scale-105"
                                            } else {
                                                "rounded-md bg-slate-800 px-2 py-1 font-mono text-[11px] text-slate-300 transition-transform duration-200 hover:scale-105"
                                            };
                                            view! { <span class=chip>{sensor.clone()}</span> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Satellite() -> impl IntoView {
    let (satellites, set_satellites) = signal(Vec::<String>::new());
    let (loaded, set_loaded) = signal(false);
    let (failed, set_failed) = signal(false);

    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    // Same reason as on the dashboard: a satellite only shows up in /satellites
    // once its first measurement has been stored, so the list has to be polled.
    spawn_local(async move {
        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            match fetch_satellites().await {
                Ok(names) => {
                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }
                    set_loaded.set(true);
                    if names != satellites.get_untracked() {
                        set_satellites.set(names);
                    }
                    if failed.get_untracked() {
                        set_failed.set(false);
                    }
                }
                Err(_) => {
                    if !failed.get_untracked() {
                        set_failed.set(true);
                    }
                }
            }

            TimeoutFuture::new(5000).await;
        }
    });

    Effect::new(move |_| {
        let _ = satellites.get();
        anim::reveal_once("[data-anim=\"reveal\"]", 0.12);
    });

    // Both closures live outside the view! macro: a turbofish like
    // ::<Vec<_>> inside an attribute is parsed as an opening tag.
    let indexed = move || -> Vec<(usize, String)> {
        satellites.get().into_iter().enumerate().collect()
    };
    let render_card = move |(i, name): (usize, String)| {
        view! { <SatelliteCard name=name index=i /> }
    };

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div data-anim="reveal" class="flex flex-col gap-4 border-b border-slate-800 pt-2 pb-6 md:flex-row md:items-center md:justify-between">
                <div>
                    <div class="flex items-center gap-2">
                        <h1 class="text-sheen text-3xl font-bold tracking-tight">"Satelliten"</h1>
                        <span class="relative flex h-2 w-2" aria-hidden="true">
                            <span class="animate-live-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400"></span>
                            <span class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
                        </span>
                    </div>
                </div>
                {move || failed.get().then(|| view! {
                    <span class="animate-pop-in rounded-full bg-amber-500/15 px-3 py-1 text-xs font-semibold text-amber-400">
                        "Backend nicht erreichbar"
                    </span>
                })}
            </div>

            {move || (!loaded.get()).then(|| view! {
                <div class="my-8 space-y-6">
                    {(0..2).map(|_| view! {
                        <div class="w-full rounded-3xl border border-slate-800 bg-slate-900 p-6 shadow-sm">
                            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                                <div class="skeleton h-48 rounded-xl"></div>
                                <div class="space-y-3 p-4">
                                    <div class="skeleton h-5 w-32 rounded-full"></div>
                                    <div class="skeleton h-4 w-full rounded-full"></div>
                                    <div class="skeleton h-4 w-4/5 rounded-full"></div>
                                    <div class="skeleton h-4 w-2/3 rounded-full"></div>
                                </div>
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            })}

            {move || (loaded.get() && satellites.get().is_empty()).then(|| view! {
                <div class="animate-fade-in my-8 rounded-2xl border border-dashed border-slate-700 p-10 text-center text-slate-500">
                    "Noch keine Satelliten empfangen."
                </div>
            })}

            // Keyed, so a newly appearing satellite adds one card instead of
            // rebuilding the ones already polling.
            <For
                each=indexed
                key=|(_, name): &(usize, String)| name.clone()
                children=render_card
            />
        </div>
    }
}