use crate::collection_diagnostics::get_collection_diagnostics;
use leptos::prelude::*;

#[component]
pub fn About() -> impl IntoView {
    let diagnostics = Resource::new(|| (), |_| async move { get_collection_diagnostics().await });
    let coordinate_coverage = Resource::new(
        || (),
        |_| async move { get_station_coordinate_coverage().await },
    );
    let fallback_position = Resource::new(
        || (),
        |_| async move { crate::fallback_position::get_fallback_position().await },
    );

    view! {
        <section class="mx-auto max-w-3xl space-y-6">
            <header class="space-y-2">
                <p class="text-sm font-semibold uppercase tracking-widest text-cyan-400">
                    "About the project"
                </p>
                <h1 class="text-3xl font-bold tracking-tight text-white sm:text-4xl">"Refuel"</h1>
                <p class="text-lg leading-7 text-slate-300">
                    "A clear view of fuel prices and their changes over time."
                </p>
            </header>
            <div class="rounded-xl border border-slate-800 bg-slate-900 p-6 leading-7 text-slate-400 shadow-sm">
                <p>
                    "Refuel brings current fuel prices and station price history together in one place."
                </p>
                <p class="mt-3">
                    "Use the current overview to compare the latest observations, or explore station history to see how prices have changed."
                </p>
            </div>
            <section class="space-y-3">
                <h2 class="text-2xl font-bold text-white">"Configured fallback position"</h2>
                <Suspense fallback=move || {
                    view! { <p class="text-slate-400">"Loading fallback position..."</p> }
                }>
                    {move || {
                        fallback_position
                            .get()
                            .map(|result| match result {
                                Ok((latitude, longitude)) => {
                                    view! {
                                        <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-200">
                                            {format!(
                                                "Latitude {latitude:.5}, longitude {longitude:.5}",
                                            )}
                                        </p>
                                    }
                                        .into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <p class="rounded-xl border border-red-900 bg-red-950/50 p-5 text-red-200">
                                            {format!("Failed to load fallback position: {error}")}
                                        </p>
                                    }
                                        .into_any()
                                }
                            })
                    }}
                </Suspense>
            </section>
            <BrowserPosition />
            <section class="space-y-3">
                <h2 class="text-2xl font-bold text-white">"Station coordinate coverage"</h2>
                <Suspense fallback=move || {
                    view! { <p class="text-slate-400">"Loading station coordinates..."</p> }
                }>
                    {move || {
                        coordinate_coverage
                            .get()
                            .map(|result| match result {
                                Ok((0, _complete)) => {
                                    view! {
                                        <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-400">
                                            "No stations are available yet. Coordinate coverage cannot be assessed."
                                        </p>
                                    }
                                        .into_any()
                                }
                                Ok((total, complete)) if total == complete => {
                                    view! {
                                        <p class="rounded-xl border border-emerald-900 bg-emerald-950/30 p-5 text-emerald-200">
                                            {format!("All {total} stations have coordinates.")}
                                        </p>
                                    }
                                        .into_any()
                                }
                                Ok((total, complete)) => {
                                    view! {
                                        <p class="rounded-xl border border-amber-900 bg-amber-950/30 p-5 text-amber-200">
                                            {format!(
                                                "{complete} of {total} stations have coordinates. Some stations are missing coordinates.",
                                            )}
                                        </p>
                                    }
                                        .into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <p class="rounded-xl border border-red-900 bg-red-950/50 p-5 text-red-200">
                                            {format!(
                                                "Failed to load station coordinate coverage: {error}",
                                            )}
                                        </p>
                                    }
                                        .into_any()
                                }
                            })
                    }}
                </Suspense>
            </section>
            <section class="space-y-4">
                <header class="space-y-1">
                    <h2 class="text-2xl font-bold text-white">"Price collection diagnostics"</h2>
                    <p class="text-sm leading-6 text-slate-400">
                        "The latest 20 collection runs are kept in memory and disappear when the server restarts."
                    </p>
                </header>
                <Suspense fallback=move || {
                    view! {
                        <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-400">
                            "Loading collection history..."
                        </p>
                    }
                }>
                    {move || {
                        diagnostics
                            .get()
                            .map(|result| match result {
                                Ok(runs) if runs.is_empty() => {
                                    view! {
                                        <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-400">
                                            "No collection runs have been recorded yet."
                                        </p>
                                    }
                                        .into_any()
                                }
                                Ok(runs) => {
                                    runs.into_iter()
                                        .map(|run| {
                                            let http_status = run
                                                .http_status
                                                .map(|status| status.to_string())
                                                .unwrap_or_else(|| "Not available".to_owned());
                                            let http_duration = run
                                                .http_duration
                                                .map(|duration| format!("{} ms", duration.as_millis()))
                                                .unwrap_or_else(|| "Not available".to_owned());
                                            let excerpt = run
                                                .response_excerpt
                                                .map(|body| {
                                                    view! {
                                                        <details class="mt-4 rounded-lg border border-slate-700 bg-slate-950">
                                                            <summary class="cursor-pointer px-3 py-2 text-sm font-semibold text-cyan-300">
                                                                "Response excerpt"
                                                            </summary>
                                                            <pre class="max-h-96 overflow-auto whitespace-pre-wrap break-words p-3 text-xs leading-5 text-slate-300">
                                                                {body}
                                                            </pre>
                                                        </details>
                                                    }
                                                        .into_any()
                                                })
                                                .unwrap_or_else(|| ().into_any());
                                            let error = run
                                                .error
                                                .map(|error| {
                                                    view! {
                                                        <p class="mt-3 rounded-lg border border-red-900 bg-red-950/50 p-3 text-sm text-red-200">
                                                            {error}
                                                        </p>
                                                    }
                                                        .into_any()
                                                })
                                                .unwrap_or_else(|| ().into_any());

                                            view! {
                                                <article class="rounded-xl border border-slate-800 bg-slate-900 p-4 shadow-sm sm:p-5">
                                                    <div class="flex flex-wrap items-start justify-between gap-2">
                                                        <div class="min-w-0 space-y-1">
                                                            <h3 class="break-all text-base font-semibold text-white">
                                                                {run.source_url.to_string()}
                                                            </h3>
                                                            <p class="text-sm text-slate-400">
                                                                {run.started_at.clone()}
                                                            </p>
                                                        </div>
                                                        <span class="rounded-full border border-slate-700 px-3 py-1 text-xs font-semibold text-cyan-200">
                                                            {run.result.clone()}
                                                        </span>
                                                    </div>
                                                    <div class="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3">
                                                        <div class="rounded-lg bg-slate-950 p-3">
                                                            <p class="text-xs text-slate-500">"HTTP status"</p>
                                                            <p class="mt-1 font-semibold text-slate-200">
                                                                {http_status}
                                                            </p>
                                                        </div>
                                                        <div class="rounded-lg bg-slate-950 p-3">
                                                            <p class="text-xs text-slate-500">"HTTP duration"</p>
                                                            <p class="mt-1 font-semibold text-slate-200">
                                                                {http_duration}
                                                            </p>
                                                        </div>
                                                        <div class="rounded-lg bg-slate-950 p-3">
                                                            <p class="text-xs text-slate-500">"Total duration"</p>
                                                            <p class="mt-1 font-semibold text-slate-200">
                                                                {format!("{} ms", run.duration.as_millis())}
                                                            </p>
                                                        </div>
                                                        <div class="rounded-lg bg-slate-950 p-3">
                                                            <p class="text-xs text-slate-500">"Extracted prices"</p>
                                                            <p class="mt-1 font-semibold text-slate-200">
                                                                {run.prices_found}
                                                            </p>
                                                        </div>
                                                        <div class="rounded-lg bg-slate-950 p-3">
                                                            <p class="text-xs text-slate-500">"Rejected entries"</p>
                                                            <p class="mt-1 font-semibold text-slate-200">
                                                                {run.rejected_entries}
                                                            </p>
                                                        </div>
                                                    </div>
                                                    {error}
                                                    {excerpt}
                                                </article>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <p class="rounded-xl border border-red-900 bg-red-950/50 p-5 text-red-200">
                                            {format!("Failed to load collection history: {error}")}
                                        </p>
                                    }
                                        .into_any()
                                }
                            })
                    }}
                </Suspense>
            </section>
        </section>
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
enum PositionState {
    Pending,
    Granted { latitude: f64, longitude: f64 },
    Denied,
    Failed(String),
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
impl PositionState {
    const PERMISSION_DENIED: u16 = 1;

    fn from_error(code: u16, message: String) -> Self {
        if code == Self::PERMISSION_DENIED {
            Self::Denied
        } else {
            Self::Failed(message)
        }
    }
}

#[cfg(feature = "hydrate")]
fn request_position(state: RwSignal<PositionState>) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::{Position, PositionError};

    let Ok(geolocation) = window().navigator().geolocation() else {
        let _ = state.try_set(PositionState::Failed(
            "Geolocation is not supported by this browser.".to_owned(),
        ));
        return;
    };
    let on_success = Closure::once_into_js(move |position: Position| {
        let coords = position.coords();
        let _ = state.try_set(PositionState::Granted {
            latitude: coords.latitude(),
            longitude: coords.longitude(),
        });
    });
    let on_error = Closure::once_into_js(move |error: PositionError| {
        let _ = state.try_set(PositionState::from_error(error.code(), error.message()));
    });

    if let Err(error) = geolocation.get_current_position_with_error_callback(
        on_success.unchecked_ref(),
        Some(on_error.unchecked_ref()),
    ) {
        let _ = state.try_set(PositionState::Failed(format!("{error:?}")));
    }
}

#[cfg(not(feature = "hydrate"))]
fn request_position(_state: RwSignal<PositionState>) {}

#[component]
fn BrowserPosition() -> impl IntoView {
    let state = RwSignal::new(PositionState::Pending);
    Effect::new(move |_| request_position(state));

    view! {
        <section class="space-y-3">
            <h2 class="text-2xl font-bold text-white">"Browser position"</h2>
            {move || match state.get() {
                PositionState::Pending => {
                    view! { <p class="text-slate-400">"Determining position..."</p> }.into_any()
                }
                PositionState::Granted { latitude, longitude } => {
                    view! {
                        <p class="rounded-xl border border-slate-800 bg-slate-900 p-5 text-slate-200">
                            {format!("Latitude {latitude:.5}, longitude {longitude:.5}")}
                        </p>
                    }
                        .into_any()
                }
                PositionState::Denied => {
                    view! {
                        <p class="rounded-xl border border-amber-900 bg-amber-950/30 p-5 text-amber-200">
                            "Location permission was not granted."
                        </p>
                    }
                        .into_any()
                }
                PositionState::Failed(reason) => {
                    view! {
                        <p class="rounded-xl border border-red-900 bg-red-950/50 p-5 text-red-200">
                            {format!("Position could not be determined: {reason}")}
                        </p>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

#[server(endpoint = "get_station_coordinate_coverage")]
pub async fn get_station_coordinate_coverage() -> Result<(i64, i64), ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::CoordinateCoverage as DBCoordinateCoverage;

    let mut conn =
        establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let coverage = DBCoordinateCoverage::load(&mut conn)
        .map_err(|error| ServerFnError::new(error.to_string()))?;

    Ok((coverage.total, coverage.complete))
}

#[cfg(test)]
mod tests {
    use super::PositionState;
    use std::assert_matches;

    #[test]
    fn permission_denied_maps_to_denied() {
        let state = PositionState::from_error(1, "User denied Geolocation".to_owned());

        assert_matches!(state, PositionState::Denied);
    }

    #[test]
    fn other_errors_keep_the_browser_message() {
        for code in [2, 3] {
            let state = PositionState::from_error(code, "reason".to_owned());

            assert_matches!(state, PositionState::Failed(reason) if reason == "reason");
        }
    }
}
