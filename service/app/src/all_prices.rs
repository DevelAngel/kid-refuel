use crate::price_list::PriceListItem;
use crate::types::StationPriceChange;

use leptos::prelude::*;

#[component]
pub fn AllPrices() -> impl IntoView {
    let list = Resource::new(
        || (),
        |_| async move { get_all_prices().await },
    );

    view! {
        <Suspense fallback=move || view! { <p>"Loading Price List with History..."</p> }>
            {move || {
                list.get().map(|result| match result {
                    Ok(list) => list.into_iter()
                        .map(|item| view! { <PriceListItem item=item/> })
                        .collect_view().into_any(),
                    Err(error) => view! {
                        <p>{format!("Failed to load all prices: {error}")}</p>
                    }.into_any(),
                })
            }}
        </Suspense>
    }
}

#[server(GetAllPrices, "/api", "GetCbor")]
pub async fn get_all_prices() -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn = establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_all(&mut conn).map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter()
        .map(StationPriceChange::from)
        .collect();
    Ok(list)
}
