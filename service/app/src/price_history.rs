use crate::types::{Station, StationPriceChange};

use chrono::prelude::*;
use leptos::prelude::*;
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::use_params_map;

#[component]
pub fn PriceHistory() -> impl IntoView {
    view! {
        <StationList/>
        <Outlet/>
    }
}

#[component]
pub fn StationList() -> impl IntoView {
    let list = Resource::new(
        || (),
        |_| async move { get_stations().await.unwrap() },
    );

    view! {
        <div>
            <Suspense fallback=move || view! { <p>"Loading Station List..."</p> }>
                <table class="primary">
                    <thead>
                        <tr>
                            <th>"ID"</th>
                            <th>"Refuel Station"</th>
                            <th>"Address"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            list.get().map(|list| {
                                list.into_iter()
                                    .map(|station| view! {
                                        <tr>
                                            <td><div><A href=station.id.to_string()>{station.id}</A></div></td>
                                            <td><div>{station.name}</div></td>
                                            <td><address>{station.addr}</address></td>
                                        </tr>
                                    })
                                    .collect_view()
                            })
                        }}
                    </tbody>
                </table>
            </Suspense>
        </div>
    }
}

#[component]
pub fn StationPriceHistory() -> impl IntoView {
    let params = use_params_map();
    let list = Resource::new(
        move || params.read().get("id").unwrap_or_default(),
        move |id| async move {
            get_price_history(id.parse::<i32>().expect("station id is no number")).await.unwrap()
        },
    );

    view! {
        <div>
            <Suspense fallback=move || view! { <p>"Loading Station Price History..."</p> }>
                <table class="primary">
                    <thead>
                        <tr>
                            <th>"Refuel Station"</th>
                            <th>"Address"</th>
                            <th>"Price"</th>
                            <th>"Updated"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            list.get().map(|list| {
                                list.into_iter()
                                    .map(|item| view! {
                                        <tr>
                                            <td><div>{item.name}</div></td>
                                            <td><address>{item.addr}</address></td>
                                            <td><span>{item.price[0]}","{item.price[1]}<sup>{item.price[2]}</sup></span></td>
                                            <td><div>{item.updated.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()}</div></td>
                                        </tr>
                                    })
                                    .collect_view()
                            })
                        }}
                    </tbody>
                </table>
            </Suspense>
        </div>
    }
}

#[server(GetStations, "/api", "GetCbor")]
pub async fn get_stations() -> Result<Vec<Station>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::Station as DBStation;

    let conn = &mut establish_connection_sqlite();
    let list = DBStation::load_all(conn);
    let list = list.into_iter()
        .map(Station::from)
        .collect();
    Ok(list)
}

#[server(GetPriceHistory, "/api", "GetCbor")]
pub async fn get_price_history(station_id: i32) -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let conn = &mut establish_connection_sqlite();
    let list = DBStationPriceChange::load_station(station_id, conn);
    let list = list.into_iter()
        .map(StationPriceChange::from)
        .collect();
    Ok(list)
}
