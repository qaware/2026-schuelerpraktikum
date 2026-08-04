use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-gray-800 p-3 text-white flex gap-6 items-center justify-center">
            <a href="/"><img src="/public/pizza.png" alt="Logo" class="h-10"/></a>
            <a href="/"><button class="cursor-pointer p-2 rounded-3xl text-white hover:text-blue-300">"Home"</button></a>
            <a href="/dashboard"><button class="cursor-pointer p-2 rounded-3xl text-white hover:text-blue-300">"Dashboard"</button></a>
            <a href="/Satellite"><button class="cursor-pointer p-2 rounded-3xl text-white hover:text-blue-300">"Satellite"</button></a>
        </nav>
    }
}