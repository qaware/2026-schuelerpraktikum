use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path; 

use crate::components::nav::Nav;
use crate::components::home::Home;
use crate::components::dashboard::Dashboard;
use crate::components::satellite::Satellite;




#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Nav />
            
            <main class="p-4">
                <Routes fallback=|| view! { <div class="p-4 text-red-500">"Seite nicht gefunden (404)"</div> }>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/dashboard") view=Dashboard />
                    <Route path=path!("/Satellite") view=Satellite />
                </Routes>
            </main>
        </Router>
    }
}