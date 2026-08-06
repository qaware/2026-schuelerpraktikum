use leptos::prelude::*;

#[component]
pub fn Satellite() -> impl IntoView {
    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-b border-slate-800 pb-6 pt-2">
                <div>
                    <div class="flex items-center align-center">
                        <h1 class="text-3xl font-bold text-slate-100 tracking-tight">"Satelliten"</h1>
                        <span class="w-2 h-2 ml-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    </div>
                </div>
            </div>

            <div class="my-8 w-full p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-sm overflow-hidden">
                <div class="grid grid-cols-2 divide-x divide-slate-800">
                    <div class="col-span-1 p-4 min-h-48 flex justify-center items-center">
                        <img src="/public/sat1.png" alt="Space" class="h-48 object-cover rounded-xl"/>
                    </div>
                    <div class="col-span-1 p-4 min-h-48"></div>
                </div>
            </div>

            <div class="my-8 w-full p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-sm overflow-hidden">
                <div class="grid grid-cols-2 divide-x divide-slate-800">
                    <div class="col-span-1 p-4 min-h-48 flex justify-center items-center">
                        <img src="/public/sat2.png" alt="Space" class="h-48 object-cover rounded-xl"/>
                    </div>
                    <div class="col-span-1 p-4 min-h-48"></div>
                </div>
            </div>

            <div class="my-8 w-full p-6 rounded-3xl bg-slate-900 border border-slate-800 shadow-sm overflow-hidden">
                <div class="grid grid-cols-2 divide-x divide-slate-800">
                    <div class="col-span-1 p-4 min-h-48 flex justify-center items-center">
                        <img src="/public/sat3.png" alt="Space" class="h-48 object-cover rounded-xl"/>
                    </div>
                    <div class="col-span-1 p-4 min-h-48"></div>
                </div>
            </div>
        </div>
    }
}
