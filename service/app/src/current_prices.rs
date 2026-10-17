use crate::price_list::PriceListItem;
use crate::types::StationPriceChange;

use leptos::prelude::*;

#[component]
pub fn CurrentPrices() -> impl IntoView {
    let list = Resource::new(
        || (),
        |_| async move { get_current_prices().await },
    );

    view! {
        <section class="space-y-6">
            <header class="space-y-2">
                <p class="text-sm font-semibold uppercase tracking-widest text-cyan-400">"Live overview"</p>
                <h1 class="text-3xl font-bold tracking-tight text-white sm:text-4xl">"Current fuel prices"</h1>
                <p class="max-w-2xl text-sm leading-6 text-slate-400 sm:text-base">"Compare the latest recorded prices at nearby stations."</p>
            </header>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
                <Suspense fallback=move || view! { <p class="rounded-xl border border-slate-800 bg-slate-900 p-6 text-slate-400">"Loading current prices..."</p> }>
                    {move || {
                        list.get().map(|result| match result {
                            Ok(list) if list.is_empty() => view! {
                                <p class="col-span-full rounded-xl border border-slate-800 bg-slate-900 p-6 text-slate-400">"No current prices are available yet."</p>
                            }.into_any(),
                            Ok(list) => list.into_iter()
                                .map(|item| view! { <PriceListItem item=item/> })
                                .collect_view().into_any(),
                            Err(error) => view! {
                                <p class="col-span-full rounded-xl border border-red-900 bg-red-950/50 p-6 text-red-200">{format!("Failed to load current prices: {error}")}</p>
                            }.into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[server(GetCurrentPrices, "/api", "GetCbor")]
pub async fn get_current_prices() -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn = establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_current(&mut conn).map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter()
        .map(StationPriceChange::from)
        .collect();
    Ok(list)
}
