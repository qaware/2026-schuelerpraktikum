use leptos::prelude::*;

#[component]
pub fn Satellite() -> impl IntoView {
    view! {
        <div class="container mx-auto max-w-screen-xl">
            <h1 class="text-3xl font-bold text-blue-600 text-center">"Satelliten"</h1>

            <div class="grid grid-flow-col grid-rows-3 my-8 border border-black/30 rounded-3xl overflow-hidden divide-x divide-gray-300">
                <div class="row-span-3 p-6 min-h-48 w-[600px]">
                    <img src="/public/sat1.png" alt="Space" class="h-48 w-full object-cover rounded-xl"/>
                </div>
                <div class="col-span-3 row-span-3 p-6 min-h-48"></div>
            </div>
            <div class="grid grid-flow-col grid-rows-3 my-8 border border-black/30 rounded-3xl overflow-hidden divide-x divide-gray-300">
                <div class="row-span-3 p-6 min-h-48 w-[600px]">
                    <img src="/public/sat2.png" alt="Space" class="h-48 w-full object-cover rounded-xl"/>
                </div>
                <div class="col-span-3 row-span-3 p-6 min-h-48"></div>
            </div>

            <div class="grid grid-flow-col grid-rows-3 my-8 border border-black/30 rounded-3xl overflow-hidden divide-x divide-gray-300">
                <div class="row-span-3 p-6 min-h-48 w-[600px]">
                    <img src="/public/sat3.png" alt="Space" class="h-48 w-full object-cover rounded-xl"/>
                </div>
                <div class="col-span-3 row-span-3 p-6 min-h-48"></div>
            </div>
        </div>
    }
}