use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_BASE: &str = "";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub latency_ms: i64,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemHealthResponse {
    pub status: String,
    pub uptime_sec: i64,
    pub timestamp: i64,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestStepResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: i64,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub total: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub duration_ms: i64,
    pub steps: Vec<TestStepResult>,
}

async fn fetch_system_health() -> Result<SystemHealthResponse, gloo_net::Error> {
    let url = format!("{}/health", API_BASE);
    Request::get(&url)
        .send()
        .await?
        .json::<SystemHealthResponse>()
        .await
}

async fn run_automated_tests() -> Result<TestSuiteResult, gloo_net::Error> {
    let url = format!("{}/health/tests/run", API_BASE);
    Request::post(&url)
        .send()
        .await?
        .json::<TestSuiteResult>()
        .await
}

#[component]
pub fn Health() -> impl IntoView {
    let health_data = RwSignal::new(Option::<SystemHealthResponse>::None);
    let test_results = RwSignal::new(Option::<TestSuiteResult>::None);
    let loading_tests = RwSignal::new(false);
    let error_msg = RwSignal::new(Option::<String>::None);

    let load_health = move || {
        spawn_local(async move {
            match fetch_system_health().await {
                Ok(data) => health_data.set(Some(data)),
                Err(err) => error_msg.set(Some(format!("Abfrage fehlgeschlagen: {}", err))),
            }
        });
    };

    load_health();

    let trigger_tests = move |_| {
        loading_tests.set(true);
        spawn_local(async move {
            match run_automated_tests().await {
                Ok(res) => {
                    test_results.set(Some(res));
                    loading_tests.set(false);
                    load_health();
                }
                Err(err) => {
                    error_msg.set(Some(format!("Testausführung fehlgeschlagen: {}", err)));
                    loading_tests.set(false);
                }
            }
        });
    };

    view! {
        <div class="max-w-6xl mx-auto p-6 space-y-6 text-slate-100">
            <div class="flex justify-between items-center bg-slate-900 p-6 rounded-2xl border border-slate-800 shadow-sm">
                <div>
                    <h1 class="text-2xl font-semibold text-slate-200">
                        "System-Diagnose & Tests"
                    </h1>
                    <p class="text-slate-400 text-sm mt-1">
                        "Status der Infrastruktur-Komponenten und automatische Integrationstests."
                    </p>
                </div>
                <button
                    on:click=move |_| load_health()
                    class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 font-medium rounded-xl border border-slate-700 text-sm transition cursor-pointer"
                >
                    "Aktualisieren"
                </button>
            </div>

            {move || error_msg.get().map(|msg| view! {
                <div class="bg-red-950/50 border border-red-800 text-red-300 p-4 rounded-xl text-sm">
                    {msg}
                </div>
            })}

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                {move || match health_data.get() {
                    Some(sys) => {
                        view! {
                            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-5 shadow-sm space-y-3">
                                <div class="flex justify-between items-center">
                                    <span class="font-medium text-slate-200">"Go-Backend"</span>
                                    <div class="flex items-center gap-2">
                                        <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                                        <span class="text-xs text-emerald-400">"Online"</span>
                                    </div>
                                </div>
                                <div class="text-xs text-slate-400 space-y-1 font-mono">
                                    <div>"Laufzeit: " <span class="text-slate-200">{sys.uptime_sec} " s"</span></div>
                                    {sys.components.get("backend").and_then(|c| c.details.clone()).map(|d| view! {
                                        <div>
                                            <div>"Goroutines: " <span class="text-slate-200">{d.get("goroutines").and_then(|v| v.as_i64()).unwrap_or(0)}</span></div>
                                            <div>"RAM: " <span class="text-slate-200">{d.get("alloc_mb").and_then(|v| v.as_i64()).unwrap_or(0)} " MB"</span></div>
                                        </div>
                                    })}
                                </div>
                            </div>

                            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-5 shadow-sm space-y-3">
                                <div class="flex justify-between items-center">
                                    <span class="font-medium text-slate-200">"MongoDB"</span>
                                    {match sys.components.get("mongodb") {
                                        Some(c) if c.status == "ONLINE" => view! {
                                            <div class="flex items-center gap-2">
                                                <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                                                <span class="text-xs text-emerald-400">"Online"</span>
                                            </div>
                                        },
                                        _ => view! {
                                            <div class="flex items-center gap-2">
                                                <span class="w-2 h-2 rounded-full bg-red-500"></span>
                                                <span class="text-xs text-red-400">"Fehler"</span>
                                            </div>
                                        }
                                    }}
                                </div>
                                <div class="text-xs text-slate-400 space-y-1 font-mono">
                                    {sys.components.get("mongodb").map(|c| view! {
                                        <div>"Latenz: " <span class="text-slate-200">{c.latency_ms} " ms"</span></div>
                                    })}
                                    {sys.components.get("mongodb").and_then(|c| c.details.clone()).map(|d| view! {
                                        <div>"Einträge: " <span class="text-slate-200">{d.get("documents_count").and_then(|v| v.as_i64()).unwrap_or(0)}</span></div>
                                    })}
                                </div>
                            </div>

                            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-5 shadow-sm space-y-3">
                                <div class="flex justify-between items-center">
                                    <span class="font-medium text-slate-200">"Datengenerator"</span>
                                    {match sys.components.get("datagen") {
                                        Some(c) if c.status == "ONLINE" => view! {
                                            <div class="flex items-center gap-2">
                                                <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                                                <span class="text-xs text-emerald-400">"Online"</span>
                                            </div>
                                        },
                                        _ => view! {
                                            <div class="flex items-center gap-2">
                                                <span class="w-2 h-2 rounded-full bg-amber-500"></span>
                                                <span class="text-xs text-amber-400">"Nicht erreichbar"</span>
                                            </div>
                                        }
                                    }}
                                </div>
                                <div class="text-xs text-slate-400 space-y-1 font-mono">
                                    {sys.components.get("datagen").map(|c| view! {
                                        <div>"API-Latenz: " <span class="text-slate-200">{c.latency_ms} " ms"</span></div>
                                    })}
                                    <div>"Port: " <span class="text-slate-200">"8090"</span></div>
                                </div>
                            </div>
                        }.into_any()
                    },
                    None => view! {
                        <div class="col-span-3 text-center py-8 text-slate-500 text-sm">
                            "Lade Systemstatus..."
                        </div>
                    }.into_any()
                }}
            </div>

            <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-sm space-y-6">
                <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 border-b border-slate-800 pb-4">
                    <div>
                        <h2 class="text-lg font-semibold text-slate-200">
                            "Integrationstests"
                        </h2>
                        <p class="text-slate-400 text-xs mt-1">
                            "Prüft Datenbankverbindung, Daten-Ingestion, Schema und Generator-Schnittstelle."
                        </p>
                    </div>

                    <button
                        on:click=trigger_tests
                        disabled=move || loading_tests.get()
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white font-medium text-sm rounded-xl transition cursor-pointer"
                    >
                        {move || if loading_tests.get() {
                            "Führe Tests aus..."
                        } else {
                            "Tests ausführen"
                        }}
                    </button>
                </div>

                {move || match test_results.get() {
                    Some(res) => view! {
                        <div class="space-y-4">
                            <div class="flex gap-4 items-center bg-slate-950 p-3 rounded-xl border border-slate-800 text-xs text-slate-300">
                                <div>
                                    "Gesamtdauer: " <span class="font-semibold text-slate-100">{res.duration_ms} " ms"</span>
                                </div>
                                <div class="h-3 w-px bg-slate-800"></div>
                                <div>
                                    "Erfolgreich: " <span class="font-semibold text-emerald-400">{res.passed_count}</span>
                                </div>
                                <div class="h-3 w-px bg-slate-800"></div>
                                <div>
                                    "Fehler: " <span class="font-semibold text-red-400">{res.failed_count}</span>
                                </div>
                            </div>

                            <div class="divide-y divide-slate-800/60 border border-slate-800/80 rounded-xl bg-slate-950/40">
                                {res.steps.into_iter().map(|step| {
                                    let is_passed = step.passed;
                                    view! {
                                        <div class="flex items-center justify-between p-3.5">
                                            <div class="flex items-center gap-3">
                                                <span class=if is_passed {
                                                    "w-2 h-2 rounded-full bg-emerald-500 shrink-0"
                                                } else {
                                                    "w-2 h-2 rounded-full bg-red-500 shrink-0"
                                                }></span>
                                                <div>
                                                    <p class="font-medium text-sm text-slate-200">{step.name}</p>
                                                    <p class="text-xs text-slate-400">{step.message}</p>
                                                </div>
                                            </div>
                                            <span class="text-xs font-mono text-slate-500">
                                                {step.duration_ms} " ms"
                                            </span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any(),
                    None => view! {
                        <div class="text-center py-8 text-slate-500 text-xs">
                            "Klicke auf 'Tests ausführen', um die Systemprüfung zu starten."
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
