use crate::types::StationPriceChange;
use crate::price_list::PriceListItem;

use leptos::*;

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
                .map(|n| view! { cx,
                    <PriceListItem item=n/>
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
