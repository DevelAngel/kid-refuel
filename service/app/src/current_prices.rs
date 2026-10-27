use crate::price_comparison::{PriceComparison, nearest_stations};
use crate::price_list::PriceListItem;
use crate::types::StationPriceChange;

use leptos::prelude::*;

#[component]
pub fn CurrentPrices() -> impl IntoView {
    let list = Resource::new(|| (), |_| async move { get_current_prices().await });
    let fallback_position = Resource::new(
        || (),
        |_| async move { crate::prelude::get_fallback_position().await },
    );
    let browser_position = crate::distance::use_browser_position();
    let nearby = Memo::new(move |_| {
        let list = list.get()?.ok()?;
        let fallback = fallback_position.get()?.ok()?;
        let position = browser_position.get().or(Some(fallback));
        Some(nearest_stations(&crate::distance::sort_by_distance(
            list, position,
        )))
    });

    view! {
        <section class="space-y-6">
            <header class="space-y-2">
                <p class="text-sm font-semibold uppercase tracking-widest text-cyan-400">
                    "Live overview"
                </p>
                <h1 class="text-3xl font-bold tracking-tight text-white sm:text-4xl">
                    "Current fuel prices"
                </h1>
                <p class="max-w-2xl text-sm leading-6 text-slate-400 sm:text-base">
                    "Compare the latest recorded prices at nearby stations."
                </p>
            </header>
            <PriceComparison stations=nearby />
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                <Suspense fallback=move || {
                    view! {
                        <p class="rounded-xl border border-slate-800 bg-slate-900 p-6 text-slate-400">
                            "Loading current prices..."
                        </p>
                    }
                }>
                    {move || {
                        let current_browser_position = browser_position.get();
                        list.get()
                            .zip(fallback_position.get())
                            .map(|(result, fallback)| {
                                match (result, fallback) {
                                    (Ok(list), Ok(_fallback)) if list.is_empty() => {
                                        view! {
                                            <p class="col-span-full rounded-xl border border-slate-800 bg-slate-900 p-6 text-slate-400">
                                                "No current prices are available yet."
                                            </p>
                                        }
                                            .into_any()
                                    }
                                    (Ok(list), Ok(fallback)) => {
                                        let position = current_browser_position.or(Some(fallback));
                                        let items = crate::distance::sort_by_distance(
                                                list,
                                                position,
                                            )
                                            .into_iter()
                                            .enumerate()
                                            .map(|(index, (item, distance))| (index, item, distance))
                                            .collect::<Vec<_>>();
                                        view! {
                                            <Show when=move || current_browser_position.is_none()>
                                                <p class="col-span-full rounded-xl border border-amber-900 bg-amber-950/30 p-4 text-sm text-amber-200">
                                                    "Using the configured fallback position."
                                                </p>
                                            </Show>
                                            <For
                                                each=move || items.clone()
                                                key=|(index, item, _)| (
                                                    item.name.clone(),
                                                    item.addr.clone(),
                                                    item.updated.timestamp_millis(),
                                                    *index,
                                                )
                                                children=move |(_index, item, _distance)| {
                                                    let coordinates = item.coordinates;
                                                    let distance = Signal::derive(move || {
                                                        let position = browser_position
                                                            .get()
                                                            .or_else(|| fallback_position.get().and_then(Result::ok));
                                                        position
                                                            .and_then(|position| {
                                                                coordinates
                                                                    .map(|coordinates| crate::distance::distance_km(
                                                                        position,
                                                                        coordinates,
                                                                    ))
                                                            })
                                                    });
                                                    view! { <PriceListItem item=item distance=distance /> }
                                                }
                                            />
                                        }
                                            .into_any()
                                    }
                                    (Err(error), _) => {
                                        view! {
                                            <p class="col-span-full rounded-xl border border-red-900 bg-red-950/50 p-6 text-red-200">
                                                {format!("Failed to load current prices: {error}")}
                                            </p>
                                        }
                                            .into_any()
                                    }
                                    (_, Err(error)) => {
                                        view! {
                                            <p class="col-span-full rounded-xl border border-red-900 bg-red-950/50 p-6 text-red-200">
                                                {format!("Failed to load fallback position: {error}")}
                                            </p>
                                        }
                                            .into_any()
                                    }
                                }
                            })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[server(endpoint = "get_current_prices")]
pub async fn get_current_prices() -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn =
        establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_current(&mut conn)
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter().map(StationPriceChange::from).collect();
    Ok(list)
}
