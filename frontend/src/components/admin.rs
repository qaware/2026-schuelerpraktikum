use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveTaskInfo {
    pub name: String,
    pub remaining_seconds: u64,
    pub intensity: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatagenStatusResponse {
    pub status: String,
    pub height_offsets: HashMap<String, f64>,
    pub anomalies: HashMap<String, serde_json::Value>,
    pub active_tasks: HashMap<String, ActiveTaskInfo>,
    pub satellites: Vec<String>,
}

#[derive(Serialize)]
struct OrbitRequest {
    satellite: String,
    height_delta: f64,
}

#[derive(Serialize)]
struct AnomalyRequest {
    satellite: String,
    sensor: String,
    r#type: String,
    value: f64,
}

#[derive(Serialize)]
struct TaskRequest {
    satellite: String,
    task_name: String,
    duration: f64,
    intensity: String,
}

async fn fetch_admin_status() -> Result<DatagenStatusResponse, gloo_net::Error> {
    let url = format!("{}/admin/status", API_BASE);
    Request::get(&url)
        .send()
        .await?
        .json::<DatagenStatusResponse>()
        .await
}

#[component]
pub fn Admin() -> impl IntoView {
    let status_data = RwSignal::new(Option::<DatagenStatusResponse>::None);
    let feedback_msg = RwSignal::new(Option::<String>::None);

    let selected_sat = RwSignal::new(String::from("ISS"));
    let height_input = RwSignal::new(50.0f64);

    let selected_sensor = RwSignal::new(String::from("thruster_1.a"));
    let anomaly_type = RwSignal::new(String::from("overheat"));
    let anomaly_value = RwSignal::new(150.0f64);

    let task_name_input = RwSignal::new(String::from("Orbit Trajectory Matrix"));
    let task_duration = RwSignal::new(15.0f64);

    let refresh_status = move || {
        spawn_local(async move {
            if let Ok(data) = fetch_admin_status().await {
                status_data.set(Some(data));
            }
        });
    };

    refresh_status();

    let execute_orbit = move |_| {
        let sat = selected_sat.get();
        let delta = height_input.get();
        spawn_local(async move {
            let url = format!("{}/admin/orbit", API_BASE);
            let req = OrbitRequest { satellite: sat.clone(), height_delta: delta };
            if let Ok(res) = Request::post(&url).json(&req).unwrap().send().await {
                if res.ok() {
                    feedback_msg.set(Some(format!("Höhenanpassung für {} ausgeführt ({:+.0} km).", sat, delta)));
                    refresh_status();
                }
            }
        });
    };

    let execute_anomaly = move |_| {
        let sat = selected_sat.get();
        let sensor = selected_sensor.get();
        let atype = anomaly_type.get();
        let val = anomaly_value.get();
        spawn_local(async move {
            let url = format!("{}/admin/anomaly", API_BASE);
            let req = AnomalyRequest { satellite: sat.clone(), sensor: sensor.clone(), r#type: atype.clone(), value: val };
            if let Ok(res) = Request::post(&url).json(&req).unwrap().send().await {
                if res.ok() {
                    feedback_msg.set(Some(format!("Anomalie '{}' für {}/{} gesetzt.", atype, sat, sensor)));
                    refresh_status();
                }
            }
        });
    };

    let execute_task = move |_| {
        let sat = selected_sat.get();
        let tname = task_name_input.get();
        let dur = task_duration.get();
        spawn_local(async move {
            let url = format!("{}/admin/task", API_BASE);
            let req = TaskRequest { satellite: sat.clone(), task_name: tname.clone(), duration: dur, intensity: "high".into() };
            if let Ok(res) = Request::post(&url).json(&req).unwrap().send().await {
                if res.ok() {
                    feedback_msg.set(Some(format!("Berechnungsaufgabe '{}' gestartet (Dauer: {:.0} s).", tname, dur)));
                    refresh_status();
                }
            }
        });
    };

    let execute_reset = move |_| {
        spawn_local(async move {
            let url = format!("{}/admin/reset", API_BASE);
            if let Ok(res) = Request::post(&url).send().await {
                if res.ok() {
                    feedback_msg.set(Some("Alle Einstellungen wurden auf Standardwerte zurückgesetzt.".into()));
                    refresh_status();
                }
            }
        });
    };

    view! {
        <div class="max-w-6xl mx-auto p-6 space-y-6 text-slate-100">
            <div class="flex justify-between items-center bg-slate-900 p-6 rounded-2xl border border-slate-800 shadow-sm">
                <div>
                    <h1 class="text-2xl font-semibold text-slate-200">
                        "Satelliten-Steuerung"
                    </h1>
                    <p class="text-slate-400 text-sm mt-1">
                        "Manuelle Anpassung von Bahnhöhen, Fehlersimulation und Rechenlasten im Datengenerator."
                    </p>
                </div>
                <button
                    on:click=execute_reset
                    class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium rounded-xl border border-slate-700 text-sm transition cursor-pointer"
                >
                    "Zurücksetzen"
                </button>
            </div>

            {move || feedback_msg.get().map(|msg| view! {
                <div class="bg-blue-950/40 border border-blue-800 text-blue-200 p-4 rounded-xl text-sm font-medium">
                    {msg}
                </div>
            })}

            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-5 shadow-sm space-y-4">
                <h2 class="text-base font-semibold text-slate-200">
                    "Aktuelle Steuerungs-Parameter"
                </h2>
                {move || match status_data.get() {
                    Some(st) => view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
                            <div class="bg-slate-950 p-4 rounded-xl border border-slate-800 space-y-1">
                                <p class="font-medium text-slate-300 mb-2">"Aktive Höhenversätze:"</p>
                                {st.height_offsets.into_iter().map(|(sat, off)| view! {
                                    <div class="flex justify-between font-mono">
                                        <span class="text-slate-400">{sat}</span>
                                        <span class="text-slate-200 font-semibold">{if off >= 0.0 { format!("+{} km", off) } else { format!("{} km", off) }}</span>
                                    </div>
                                }).collect::<Vec<_>>()}
                            </div>

                            <div class="bg-slate-950 p-4 rounded-xl border border-slate-800 space-y-1">
                                <p class="font-medium text-slate-300 mb-2">"Aktive Anomalien:"</p>
                                {if st.anomalies.is_empty() {
                                    view! { <p class="text-slate-500 font-mono">"Keine"</p> }.into_any()
                                } else {
                                    st.anomalies.into_iter().map(|(k, v)| view! {
                                        <p class="text-amber-400 font-mono">{k} " => " {v.to_string()}</p>
                                    }).collect::<Vec<_>>().into_any()
                                }}
                            </div>

                            <div class="bg-slate-950 p-4 rounded-xl border border-slate-800 space-y-1">
                                <p class="font-medium text-slate-300 mb-2">"Laufende Rechenaufgaben:"</p>
                                {if st.active_tasks.is_empty() {
                                    view! { <p class="text-slate-500 font-mono">"Inaktiv"</p> }.into_any()
                                } else {
                                    st.active_tasks.into_iter().map(|(sat, task)| view! {
                                        <div class="p-2 rounded bg-slate-900 border border-slate-800 text-slate-300 font-mono">
                                            <p class="font-semibold text-slate-200">{sat} ": " {task.name}</p>
                                            <p class="text-slate-400">{task.remaining_seconds} " s verbleibend"</p>
                                        </div>
                                    }).collect::<Vec<_>>().into_any()
                                }}
                            </div>
                        </div>
                    }.into_any(),
                    None => view! {
                        <p class="text-slate-500 text-xs">"Lade Status..."</p>
                    }.into_any()
                }}
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">

                <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-sm space-y-4">
                    <h2 class="text-base font-semibold text-slate-200">
                        "Bahnhöhe anpassen"
                    </h2>
                    <p class="text-xs text-slate-400">
                        "Ändert die Basishöhe des Satelliten. Wirkt sich direkt auf die Telemetriedaten aus."
                    </p>

                    <div class="space-y-3 text-sm">
                        <div>
                            <label class="block text-xs text-slate-400 mb-1">"Satellit auswählen"</label>
                            <select
                                on:change=move |ev| selected_sat.set(event_target_value(&ev))
                                class="w-full bg-slate-950 border border-slate-700 rounded-xl p-2.5 text-slate-200 text-sm focus:outline-none focus:border-blue-500"
                            >
                                <option value="ISS">"ISS"</option>
                                <option value="Hubble">"Hubble"</option>
                                <option value="JWST">"JWST"</option>
                            </select>
                        </div>

                        <div>
                            <label class="block text-xs text-slate-400 mb-1">
                                "Höhenänderung: " <span class="font-semibold text-slate-200">{move || height_input.get()} " km"</span>
                            </label>
                            <input
                                type="range" min="-100" max="200" step="5"
                                value=move || height_input.get().to_string()
                                on:input=move |ev| height_input.set(event_target_value(&ev).parse().unwrap_or(0.0))
                                class="w-full h-2 bg-slate-950 rounded-lg appearance-none cursor-pointer accent-blue-500"
                            />
                        </div>

                        <button
                            on:click=execute_orbit
                            class="w-full py-2.5 bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 font-medium text-sm rounded-xl transition cursor-pointer"
                        >
                            "Höhenanpassung senden"
                        </button>
                    </div>
                </div>

                <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-sm space-y-4">
                    <h2 class="text-base font-semibold text-slate-200">
                        "Anomalie simulieren"
                    </h2>
                    <p class="text-xs text-slate-400">
                        "Erzeugt Temperaturspitzen oder Druckabfälle auf bestimmten Sensoren."
                    </p>

                    <div class="space-y-3 text-sm">
                        <div>
                            <label class="block text-xs text-slate-400 mb-1">"Sensor auswählen"</label>
                            <select
                                on:change=move |ev| selected_sensor.set(event_target_value(&ev))
                                class="w-full bg-slate-950 border border-slate-700 rounded-xl p-2.5 text-slate-200 text-sm focus:outline-none focus:border-blue-500"
                            >
                                <option value="thruster_1.a">"thruster_1.a"</option>
                                <option value="thruster_1.b">"thruster_1.b"</option>
                                <option value="thruster_2.a">"thruster_2.a"</option>
                                <option value="oxygen_tank_1">"oxygen_tank_1"</option>
                                <option value="hydrogen_tank_1">"hydrogen_tank_1"</option>
                            </select>
                        </div>

                        <div>
                            <label class="block text-xs text-slate-400 mb-1">"Anomalie-Typ"</label>
                            <select
                                on:change=move |ev| anomaly_type.set(event_target_value(&ev))
                                class="w-full bg-slate-950 border border-slate-700 rounded-xl p-2.5 text-slate-700 text-sm focus:outline-none focus:border-blue-500 text-slate-200"
                            >
                                <option value="overheat">"Temperaturspitze (+150 °C)"</option>
                                <option value="pressure_drop">"Druckabfall (-3.0 bar)"</option>
                                <option value="clear">"Anomalie zurücksetzen"</option>
                            </select>
                        </div>

                        <button
                            on:click=execute_anomaly
                            class="w-full py-2.5 bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 font-medium text-sm rounded-xl transition cursor-pointer"
                        >
                            "Anomalie senden"
                        </button>
                    </div>
                </div>

                <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-sm space-y-4">
                    <h2 class="text-base font-semibold text-slate-200">
                        "Rechenaufgabe starten"
                    </h2>
                    <p class="text-xs text-slate-400">
                        "Führt intensiv arbeitenden Hintergrundcode im Generator aus (CPU-Last)."
                    </p>

                    <div class="space-y-3 text-sm">
                        <div>
                            <label class="block text-xs text-slate-400 mb-1">"Aufgabentyp"</label>
                            <select
                                on:change=move |ev| task_name_input.set(event_target_value(&ev))
                                class="w-full bg-slate-950 border border-slate-700 rounded-xl p-2.5 text-slate-200 text-sm focus:outline-none focus:border-blue-500"
                            >
                                <option value="Orbit Trajectory Matrix">"Bahnberechnung"</option>
                                <option value="Deep Space Key Rotation">"Schlüssel-Rotation"</option>
                                <option value="Diagnostic Full Sweep">"System-Selbsttest"</option>
                            </select>
                        </div>

                        <div>
                            <label class="block text-xs text-slate-400 mb-1">
                                "Dauer: " <span class="font-semibold text-slate-200">{move || task_duration.get()} " s"</span>
                            </label>
                            <input
                                type="range" min="5" max="60" step="5"
                                value=move || task_duration.get().to_string()
                                on:input=move |ev| task_duration.set(event_target_value(&ev).parse().unwrap_or(15.0))
                                class="w-full h-2 bg-slate-950 rounded-lg appearance-none cursor-pointer accent-blue-500"
                            />
                        </div>

                        <button
                            on:click=execute_task
                            class="w-full py-2.5 bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 font-medium text-sm rounded-xl transition cursor-pointer"
                        >
                            "Aufgabe starten"
                        </button>
                    </div>
                </div>

            </div>
        </div>
    }
}
