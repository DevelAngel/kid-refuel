use crate::collection_diagnostics::get_collection_diagnostics;
use leptos::prelude::*;

#[component]
pub fn About() -> impl IntoView {
    let diagnostics = Resource::new(|| (), |_| async move { get_collection_diagnostics().await });

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
            <section class="space-y-4">
                <header class="space-y-1">
                    <h2 class="text-2xl font-bold text-white">"Price collection diagnostics"</h2>
                    <p class="text-sm leading-6 text-slate-400">"The latest 20 collection runs are kept in memory and disappear when the server restarts."</p>
                </header>
                <Suspense fallback=move || view! {
                    <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-400">"Loading collection history..."</p>
                }>
                    {move || {
                        diagnostics.get().map(|result| match result {
                            Ok(runs) if runs.is_empty() => view! {
                                <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-400">"No collection runs have been recorded yet."</p>
                            }.into_any(),
                            Ok(runs) => runs.into_iter().map(|run| {
                                let http_status = run.http_status
                                    .map(|status| status.to_string())
                                    .unwrap_or_else(|| "Not available".to_owned());
                                let http_duration = run.http_duration
                                    .map(|duration| format!("{} ms", duration.as_millis()))
                                    .unwrap_or_else(|| "Not available".to_owned());
                                let excerpt = run.response_excerpt.map(|body| view! {
                                    <details class="mt-4 rounded-lg border border-slate-700 bg-slate-950">
                                        <summary class="cursor-pointer px-3 py-2 text-sm font-semibold text-cyan-300">"Response excerpt"</summary>
                                        <pre class="max-h-96 overflow-auto whitespace-pre-wrap break-words p-3 text-xs leading-5 text-slate-300">{body}</pre>
                                    </details>
                                }.into_any()).unwrap_or_else(|| ().into_any());
                                let error = run.error.map(|error| view! {
                                    <p class="mt-3 rounded-lg border border-red-900 bg-red-950/50 p-3 text-sm text-red-200">{error}</p>
                                }.into_any()).unwrap_or_else(|| ().into_any());

                                view! {
                                    <article class="rounded-xl border border-slate-800 bg-slate-900 p-4 shadow-sm sm:p-5">
                                        <div class="flex flex-wrap items-start justify-between gap-2">
                                            <div class="min-w-0 space-y-1">
                                                <h3 class="break-all text-base font-semibold text-white">{run.source_url.to_string()}</h3>
                                                <p class="text-sm text-slate-400">{run.started_at.clone()}</p>
                                            </div>
                                            <span class="rounded-full border border-slate-700 px-3 py-1 text-xs font-semibold text-cyan-200">{run.result.clone()}</span>
                                        </div>
                                        <div class="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3">
                                            <div class="rounded-lg bg-slate-950 p-3">
                                                <p class="text-xs text-slate-500">"HTTP status"</p>
                                                <p class="mt-1 font-semibold text-slate-200">{http_status}</p>
                                            </div>
                                            <div class="rounded-lg bg-slate-950 p-3">
                                                <p class="text-xs text-slate-500">"HTTP duration"</p>
                                                <p class="mt-1 font-semibold text-slate-200">{http_duration}</p>
                                            </div>
                                            <div class="rounded-lg bg-slate-950 p-3">
                                                <p class="text-xs text-slate-500">"Total duration"</p>
                                                <p class="mt-1 font-semibold text-slate-200">{format!("{} ms", run.duration.as_millis())}</p>
                                            </div>
                                            <div class="rounded-lg bg-slate-950 p-3">
                                                <p class="text-xs text-slate-500">"Extracted prices"</p>
                                                <p class="mt-1 font-semibold text-slate-200">{run.prices_found}</p>
                                            </div>
                                            <div class="rounded-lg bg-slate-950 p-3">
                                                <p class="text-xs text-slate-500">"Rejected entries"</p>
                                                <p class="mt-1 font-semibold text-slate-200">{run.rejected_entries}</p>
                                            </div>
                                        </div>
                                        {error}
                                        {excerpt}
                                    </article>
                                }
                            }).collect_view().into_any(),
                            Err(error) => view! {
                                <p class="rounded-xl border border-red-900 bg-red-950/50 p-5 text-red-200">{format!("Failed to load collection history: {error}")}</p>
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </section>
        </section>
    }
}
