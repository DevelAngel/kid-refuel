use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::*;
use leptos_router::*;

#[component]
pub fn CurrentPrices(cx: Scope) -> impl IntoView {
    let list = create_resource(
        cx,
        || (), //< run once
        |_| async move {
            get_current_prices().await.unwrap()
        },
    );

    view! {
        cx,
        <Suspense fallback=move || view! { cx, <p>"Loading Current Price List..."</p> }>
            {move || { list.read(cx).map(|list| list.into_iter()
                .map(|n| {
                    let name = &n.name;
                    let addr = &n.addr;
                    let price = &n.price;
                    let updated = n.updated.with_timezone(&Local);
                    view! {
                        cx,
                        <div class="flex flex-col bg-white text-gray-700 py-2 px-4">
                            <div class="flex flex-row justify-between text-2xl">
                                // station name
                                <div class="flex-none px-1">{name}</div>
                                // price
                                <div class="flex-none px-1">{price[0]}","{price[1]}<sup>{price[2]}</sup></div>
                            </div>
                            <div class="flex flex-row justify-between flex-wrap text-base">
                                // station address
                                <address class="flex-1 px-1">{addr}</address>
                                // updated date and time
                                <div class="flex-none px-1">{format!("{}", updated.format("%Y-%m-%d"))}</div>
                                <div class="flex-none px-1">{format!("{}", updated.format("%H:%M"))}</div>
                            </div>
                        </div>
                    }
                })
                .collect_view(cx)
            )}}
        </Suspense>
    }
}

#[server(GetCurrentPrices, "/api", "GetCbor")]
pub async fn get_current_prices() -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    // simulate some time to acquire the informations
    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = DBStationPriceChange::load_current(conn);
    let list = list.into_iter()
        .map(|rs| StationPriceChange::from(rs))
        .collect();
    Ok(list)
}
