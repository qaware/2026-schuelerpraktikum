use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::anim;
use crate::components::dashboard::{fetch_satellites, fetch_specs, SatelliteSpecs};

/// Vier Bilder, beliebig viele Satelliten -- daher zyklisch zugeordnet.
const IMAGES: [&str; 4] = ["sat1", "sat2", "sat3", "sat4"];

/// Ein Datenfeld in der Spec-Tabelle.
#[component]
fn SpecRow(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex items-baseline justify-between gap-4 border-b border-gray-100 py-2 last:border-b-0">
            <dt class="text-xs font-medium tracking-wide text-gray-500 uppercase">{label}</dt>
            <dd class="text-right text-sm font-semibold text-gray-900">{value}</dd>
        </div>
    }
}

#[component]
fn SpecCard(specs: SatelliteSpecs, index: usize) -> impl IntoView {
    let image = IMAGES[index % IMAGES.len()];
    let sensor_count = specs.sensors.len();

    view! {
        <div
            data-anim="reveal"
            class="group my-8 w-full overflow-hidden rounded-3xl border border-gray-200 bg-white p-6 shadow-sm transition-all duration-300 hover:-translate-y-1 hover:border-blue-200 hover:shadow-lg"
        >
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:divide-x md:divide-gray-200">
                <div class="col-span-1 flex min-h-48 items-center justify-center p-4">
                    <img
                        src=format!("/public/{}.png", image)
                        alt=format!("Satellit {}", specs.name)
                        class="animate-float h-48 rounded-xl object-cover transition-transform duration-500 ease-out group-hover:scale-105"
                        style=format!("animation-delay: {}ms", index * 900)
                    />
                </div>

                // Diese Haelfte war bisher ein leeres <div>.
                <div class="col-span-1 min-h-48 p-4">
                    <div class="mb-3 flex items-center gap-2">
                        <h2 class="text-xl font-bold text-gray-900">{specs.name.clone()}</h2>
                        <span class="rounded-full bg-blue-50 px-2 py-0.5 text-[10px] font-semibold tracking-wide text-blue-700 uppercase">
                            {specs.nation.clone()}
                        </span>
                    </div>

                    <dl class="mb-4">
                        <SpecRow label="Modell" value=specs.model.clone() />
                        <SpecRow label="Startdatum" value=specs.launchdate.clone() />
                        <SpecRow label="Sensoren" value=sensor_count.to_string() />
                    </dl>

                    <p class="mb-2 text-xs font-medium tracking-wide text-gray-500 uppercase">"Sensorik"</p>
                    <div class="flex flex-wrap gap-1.5">
                        {specs
                            .sensors
                            .iter()
                            .map(|sensor| {
                                // Tanks und Thruster optisch trennen, damit die
                                // Liste bei einem Dutzend Sensoren lesbar bleibt.
                                let is_tank = sensor.contains("tank");
                                let chip = if is_tank {
                                    "rounded-md bg-amber-50 px-2 py-1 font-mono text-[11px] text-amber-800 transition-transform duration-200 hover:scale-105"
                                } else {
                                    "rounded-md bg-gray-100 px-2 py-1 font-mono text-[11px] text-gray-700 transition-transform duration-200 hover:scale-105"
                                };
                                view! { <span class=chip>{sensor.clone()}</span> }
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Satellite() -> impl IntoView {
    let (specs, set_specs) = signal(Vec::<SatelliteSpecs>::new());
    let (loaded, set_loaded) = signal(false);
    let (failed, set_failed) = signal(false);

    let alive = Arc::new(AtomicBool::new(true));
    let cleanup_flag = alive.clone();
    on_cleanup(move || cleanup_flag.store(false, Ordering::Relaxed));

    spawn_local(async move {
        loop {
            if !alive.load(Ordering::Relaxed) {
                break;
            }

            match fetch_satellites().await {
                Ok(names) => {
                    let mut collected = Vec::with_capacity(names.len());
                    for name in &names {
                        if !alive.load(Ordering::Relaxed) {
                            return;
                        }
                        if let Ok(spec) = fetch_specs(name).await {
                            collected.push(spec);
                        }
                    }

                    if !alive.load(Ordering::Relaxed) {
                        break;
                    }
                    // Nur bei echter Aenderung schreiben, sonst baut jeder
                    // Durchlauf die Karten neu auf.
                    if collected != specs.get_untracked() {
                        set_specs.set(collected);
                    }
                    if !loaded.get_untracked() {
                        set_loaded.set(true);
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

            // Specs selbst aendern sich nie -- gepollt wird nur, um neu
            // hinzugekommene Satelliten aufzunehmen.
            TimeoutFuture::new(10000).await;
        }
    });

    Effect::new(move |_| {
        let _ = specs.get();
        anim::reveal_once("[data-anim=\"reveal\"]", 0.12);
    });

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div data-anim="reveal" class="flex flex-col gap-4 border-b border-gray-200 pt-2 pb-6 md:flex-row md:items-center md:justify-between">
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
                    <span class="animate-pop-in rounded-full bg-amber-100 px-3 py-1 text-xs font-semibold text-amber-700">
                        "Backend nicht erreichbar"
                    </span>
                })}
            </div>

            {move || (!loaded.get()).then(|| view! {
                <div class="my-8 space-y-6">
                    {(0..2).map(|_| view! {
                        <div class="w-full rounded-3xl border border-gray-200 bg-white p-6 shadow-sm">
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

            {move || (loaded.get() && specs.get().is_empty()).then(|| view! {
                <div class="animate-fade-in my-8 rounded-2xl border border-dashed border-gray-300 p-10 text-center text-gray-400">
                    "Noch keine Satelliten empfangen."
                </div>
            })}

            {move || specs
                .get()
                .into_iter()
                .enumerate()
                .map(|(i, spec)| view! { <SpecCard specs=spec index=i /> })
                .collect_view()}
        </div>
    }
}
