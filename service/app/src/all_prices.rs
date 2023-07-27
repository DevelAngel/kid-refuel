use crate::types::StationPriceChange;
use crate::price_list::PriceListItem;

use chrono::prelude::*;
use leptos::*;

#[component]
pub fn AllPrices(cx: Scope) -> impl IntoView {
    let list = create_resource(
        cx,
        || (), //< run once
        |_| async move {
            get_all_prices().await.expect("get_all_prices resource")
        },
    );

    view! {
        cx,
        <Suspense fallback=move || view! { cx, <p>"Loading Price List with History..."</p> }>
            {move || { list.read(cx).map(|list| list.into_iter()
                .map(|n| view! { cx,
                    <PriceListItem item=n/>
                })
                .collect_view(cx)
            )}}
        </Suspense>
    }
}

#[server(GetAllPrices, "/api", "GetCbor")]
pub async fn get_all_prices() -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    // simulate some time to acquire the informations
    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = DBStationPriceChange::load_all(conn);
    let list = list.into_iter()
        .map(|rs| StationPriceChange::from(rs))
        .collect();
    Ok(list)
}
