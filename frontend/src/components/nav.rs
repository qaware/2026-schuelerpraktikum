use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-slate-900 border-b border-slate-800 p-3 text-slate-100 flex gap-6 items-center justify-center">
            <a href="/"><img src="/public/pizza.png" alt="Logo" class="h-10"/></a>
            <a href="/"><button class="cursor-pointer p-2 rounded-3xl text-slate-300 hover:text-blue-400">"Home"</button></a>
            <a href="/dashboard"><button class="cursor-pointer p-2 rounded-3xl text-slate-300 hover:text-blue-400">"Dashboard"</button></a>
            <a href="/Satellite"><button class="cursor-pointer p-2 rounded-3xl text-slate-300 hover:text-blue-400">"Satellite"</button></a>
        </nav>
    }
}