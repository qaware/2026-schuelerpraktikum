use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="app-container">
                <header>
                    <div class="logo">"frontend"</div>
                    <nav>
                        <A href="/">"Home"</A>
                        <A href="/details">"Details"</A>
                    </nav>
                </header>
                <main>
                    <Routes fallback=|| view! { <h1>"404 - Seite nicht gefunden"</h1> }>
                        <Route path=path!("/") view=HomePage />
                        <Route path=path!("/details") view=AppDetailsPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section>
            <h1>"Hallo Welt"</h1>
        </section>
    }
}

#[component]
fn AppDetailsPage() -> impl IntoView {
    view! {
        <section>
            <h1>"Hallo Welt im detail"</h1>
        </section>
    }
}
