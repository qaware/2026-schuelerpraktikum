use leptos::prelude::*;
use leptos_router::components::A;

/// Ein Navigationslink.
///
/// `<A>` rendert selbst ein `<a>` und setzt darauf `aria-current="page"`, sobald
/// die Route aktiv ist -- die Hervorhebung braucht daher kein eigenes Signal,
/// sondern haengt an der `aria-[current=page]`-Variante.
#[component]
fn NavLink(href: &'static str, label: &'static str, exact: bool) -> impl IntoView {
    view! {
        <A
            href=href
            exact=exact
            attr:class="group relative px-3.5 py-2 text-sm font-medium text-gray-300 no-underline transition-colors duration-200 hover:text-white aria-[current=page]:text-white"
        >
            // Hintergrund-Pille: skaliert beim Hover auf und bleibt auf der
            // aktiven Route stehen.
            <span class="absolute inset-0 rounded-xl bg-white/10 opacity-0 scale-90 transition-all duration-200 ease-out group-hover:opacity-100 group-hover:scale-100 group-aria-[current=page]:opacity-100 group-aria-[current=page]:scale-100"></span>
            // Unterstrich, der aus der Mitte herauswaechst.
            <span class="absolute bottom-0 left-1/2 h-0.5 w-0 -translate-x-1/2 rounded-full bg-linear-to-r from-blue-400 to-violet-400 transition-all duration-300 ease-out group-hover:w-2/3 group-aria-[current=page]:w-2/3"></span>
            <span class="relative">{label}</span>
        </A>
    }
}

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="animate-slide-down sticky top-0 z-50 flex items-center justify-center gap-2 border-b border-white/5 bg-gray-800/85 p-3 text-white shadow-lg backdrop-blur-md">
            <A href="/" attr:class="group mr-2 shrink-0">
                <img
                    src="/public/pizza.png"
                    alt="Zur Startseite"
                    class="h-10 transition-transform duration-300 ease-out group-hover:scale-110 group-hover:-rotate-6"
                />
            </A>
            <NavLink href="/" label="Home" exact=true />
            <NavLink href="/dashboard" label="Dashboard" exact=false />
            <NavLink href="/satellite" label="Satellite" exact=false />
            <NavLink href="/orbit" label="Orbit" exact=false />
            <NavLink href="/health" label="System-Tests" exact=false />
            <NavLink href="/admin" label="Steuerung" exact=false />
        </nav>
    }
}
