use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="container mx-auto max-w-screen-xl">
            <h1 class="text-3xl font-bold text-blue-600 text-center ">"Dashboard"</h1>
            <div class="grid grid-flow-col grid-rows-3 gap-6 my-8">
                <div class="row-span-3 shadow-xl/20 border border-black/10 rounded-3xl p-2 min-h-48">01</div>
                <div class="col-span-1 shadow-xl/20 border border-black/10 rounded-3xl p-2 min-h-48">02</div>
                <div class="col-span-2 row-span-2 shadow-xl/20 border border-black/10 rounded-3xl p-2 min-h-48">03</div>
                <div class="col-span-1 shadow-xl/20 border border-black/10 rounded-3xl p-2 min-h-48">02</div>
            </div>
        </div>
    }
}