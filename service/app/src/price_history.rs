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
        |_| async move { get_stations().await },
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
                            list.get().map(|result| match result {
                                Ok(list) => list.into_iter()
                                    .map(|station| view! {
                                        <tr>
                                            <td><div><A href=station.id.to_string()>{station.id}</A></div></td>
                                            <td><div>{station.name}</div></td>
                                            <td><address>{station.addr}</address></td>
                                        </tr>
                                    })
                                    .collect_view().into_any(),
                                Err(error) => view! { <p>{format!("Failed to load stations: {error}")}</p> }.into_any(), })
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
            match id.parse::<i32>() { Ok(station_id) => get_price_history(station_id).await, Err(error) => Err(ServerFnError::new(format!("Invalid station ID: {error}"))) }
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
                            list.get().map(|result| match result {
                                Ok(list) => list.into_iter()
                                    .map(|item| view! {
                                        <tr>
                                            <td><div>{item.name}</div></td>
                                            <td><address>{item.addr}</address></td>
                                            <td><span>{item.price[0]}","{item.price[1]}<sup>{item.price[2]}</sup></span></td>
                                            <td><div>{item.updated.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()}</div></td>
                                        </tr>
                                    })
                                    .collect_view().into_any(),
                                Err(error) => view! { <p>{format!("Failed to load station history: {error}")}</p> }.into_any(), })
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

    let mut conn = establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStation::load_all(&mut conn).map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter()
        .map(Station::from)
        .collect();
    Ok(list)
}

#[server(GetPriceHistory, "/api", "GetCbor")]
pub async fn get_price_history(station_id: i32) -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn = establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_station(station_id, &mut conn).map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter()
        .map(StationPriceChange::from)
        .collect();
    Ok(list)
}
