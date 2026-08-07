use leptos::prelude::*;

use crate::anim;

#[component]
pub fn Home() -> impl IntoView {
    Effect::new(move |_| {
        anim::reveal_once("[data-anim=\"reveal\"]", 0.1);
    });

    view! {
        <div class="container mx-auto max-w-screen-xl px-4">
            <div data-anim="reveal" class="flex flex-col gap-4 border-b border-gray-200 pt-2 pb-6 md:flex-row md:items-center md:justify-between">
                <div>
                    <div class="flex items-center gap-2">
                        <h1 class="text-sheen text-3xl font-bold tracking-tight">"Home"</h1>
                        <span class="relative flex h-2 w-2" aria-hidden="true">
                            <span class="animate-live-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400"></span>
                            <span class="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
                        </span>
                    </div>
                </div>
            </div>

            <div
                data-anim="reveal" class="mt-8 space-y-4 rounded-2xl border border-gray-200 bg-white p-8 shadow-sm transition-all duration-300 hover:shadow-md"
            >
                <h2 class="text-2xl font-semibold text-gray-800">"Willkommen"</h2>
                <p class="leading-relaxed text-gray-600">
                    "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
                </p>
                <p class="leading-relaxed text-gray-600">
                    "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."
                </p>
            </div>

            <div class="mt-8 grid grid-cols-1 gap-6 md:grid-cols-2">
                <div
                    data-anim="reveal" class="group rounded-2xl border border-gray-200 bg-white p-6 shadow-sm transition-all duration-300 hover:-translate-y-1 hover:border-blue-200 hover:shadow-lg"
                >
                    <h3 class="mb-3 text-lg font-semibold text-gray-800 transition-colors duration-300 group-hover:text-blue-600">"Dummy Sektion 1"</h3>
                    <p class="text-sm text-gray-600">"Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat, vel illum dolore eu feugiat nulla facilisis at vero eros et accumsan et iusto odio dignissim qui blandit praesent luptatum zzril delenit augue duis dolore te feugait nulla facilisi."</p>
                </div>
                <div
                    data-anim="reveal" class="group rounded-2xl border border-gray-200 bg-white p-6 shadow-sm transition-all duration-300 hover:-translate-y-1 hover:border-violet-200 hover:shadow-lg"
                >
                    <h3 class="mb-3 text-lg font-semibold text-gray-800 transition-colors duration-300 group-hover:text-violet-600">"Dummy Sektion 2"</h3>
                    <p class="text-sm text-gray-600">"Nam liber tempor cum soluta nobis eleifend option congue nihil imperdiet doming id quod mazim placerat facer possim assum. Lorem ipsum dolor sit amet, consectetuer adipiscing elit, sed diam nonummy nibh euismod tincidunt ut laoreet dolore magna aliquam erat volutpat."</p>
                </div>
            </div>
        </div>
    }
}
