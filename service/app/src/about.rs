use leptos::prelude::*;

#[component]
pub fn About() -> impl IntoView {
    view! {
        <section class="mx-auto max-w-3xl space-y-6">
            <header class="space-y-2">
                <p class="text-sm font-semibold uppercase tracking-widest text-cyan-400">"About the project"</p>
                <h1 class="text-3xl font-bold tracking-tight text-white sm:text-4xl">"Refuel"</h1>
                <p class="text-lg leading-7 text-slate-300">"A clear view of fuel prices and their changes over time."</p>
            </header>
            <div class="rounded-xl border border-slate-800 bg-slate-900 p-6 leading-7 text-slate-400 shadow-sm">
                <p>"Refuel brings current fuel prices and station price history together in one place."</p>
                <p class="mt-3">"Use the current overview to compare the latest observations, or explore station history to see how prices have changed."</p>
            </div>
        </section>
    }
}
