use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4 border-b border-slate-800 pb-6 pt-2">
                <div>
                    <div class="flex items-center align-center">
                        <h1 class="text-3xl font-bold text-slate-100 tracking-tight">"Home"</h1>
                        <span class="w-2 h-2 ml-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                    </div>
                </div>
            </div>

            <div class="bg-slate-900 p-8 rounded-2xl border border-slate-800 shadow-sm space-y-4 mt-8">
                <h2 class="text-2xl font-semibold text-slate-200">"Willkommen"</h2>
                <p class="text-slate-400 leading-relaxed">
                    "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
                </p>
                <p class="text-slate-400 leading-relaxed">
                    "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
                <div class="bg-slate-900 p-6 rounded-2xl border border-slate-800 shadow-sm">
                    <h3 class="text-lg font-semibold text-slate-200 mb-3">"Dummy Sektion 1"</h3>
                    <p class="text-slate-400 text-sm">"Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat, vel illum dolore eu feugiat nulla facilisis at vero eros et accumsan et iusto odio dignissim qui blandit praesent luptatum zzril delenit augue duis dolore te feugait nulla facilisi."</p>
                </div>
                <div class="bg-slate-900 p-6 rounded-2xl border border-slate-800 shadow-sm">
                    <h3 class="text-lg font-semibold text-slate-200 mb-3">"Dummy Sektion 2"</h3>
                    <p class="text-slate-400 text-sm">"Nam liber tempor cum soluta nobis eleifend option congue nihil imperdiet doming id quod mazim placerat facer possim assum. Lorem ipsum dolor sit amet, consectetuer adipiscing elit, sed diam nonummy nibh euismod tincidunt ut laoreet dolore magna aliquam erat volutpat."</p>
                </div>
            </div>
        </div>
    }
}
