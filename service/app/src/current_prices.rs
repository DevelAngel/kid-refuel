use crate::price_list::PriceListItem;
use crate::types::StationPriceChange;

use leptos::prelude::*;

#[component]
pub fn CurrentPrices() -> impl IntoView {
    let list = Resource::new(
        || (),
        |_| async move { get_current_prices().await.unwrap() },
    );

    view! {
        <Suspense fallback=move || view! { <p>"Loading Current Price List..."</p> }>
            {move || {
                list.get().map(|list| {
                    list.into_iter()
                        .map(|item| view! { <PriceListItem item=item/> })
                        .collect_view()
                })
            }}
        </Suspense>
    }
}

#[server(GetCurrentPrices, "/api", "GetCbor")]
pub async fn get_current_prices() -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let conn = &mut establish_connection_sqlite();
    let list = DBStationPriceChange::load_current(conn);
    let list = list.into_iter()
        .map(StationPriceChange::from)
        .collect();
    Ok(list)
}
