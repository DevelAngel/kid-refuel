use crate::types::Station;
use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::*;
use leptos_router::*;

#[component]
pub fn PriceHistory(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <StationList/>
        <Outlet/>
    }
}

#[component]
pub fn StationList(cx: Scope) -> impl IntoView {
    let list = create_resource(
        cx,
        || (), //< run once
        |_| async move {
            get_stations().await.unwrap()
        },
    );

    view! {
        cx,
        <div>
            <Suspense fallback=move || view! { cx, <p>"Loading Station List..."</p> }>
                <table class="primary">
                    <thead>
                        <tr>
                            <th>"Refuel Station"</th>
                            <th>"Address"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || { list.read(cx).map(|list| list.into_iter()
                            .map(|n| view! { cx,
                                <tr>
                                    <td><div>{n.name}</div></td>
                                    <td><address>{n.addr}</address></td>
                                </tr>
                            })
                            .collect_view(cx)
                        )}}
                    </tbody>
                </table>
            </Suspense>
        </div>
    }
}

#[component]
pub fn StationPriceHistory(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);

    let list = create_resource(
        cx,
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| async move {
            get_price_history(id.parse::<i32>().expect("station id is no number")).await.unwrap()
        },
    );

    view! {
        cx,
        <div>
            <Suspense fallback=move || view! { cx, <p>"Loading Station Price History..."</p> }>
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
                        {move || { list.read(cx).map(|list| list.into_iter()
                            .map(|n| view! { cx,
                                <tr>
                                    <td><div>{n.name}</div></td>
                                    <td><address>{n.addr}</address></td>
                                    <td><span>{n.price[0]}","{n.price[1]}<sup>{n.price[2]}</sup></span></td>
                                    <td><div>{format!("{}", n.updated.with_timezone(&Local).format("%Y-%m-%d %H:%M"))}</div></td>
                                </tr>
                            })
                            .collect_view(cx)
                        )}}
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

    // simulate some time to acquire the informations
    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = DBStation::load_all(conn);
    let list = list.into_iter()
        .map(|rs| Station::from(rs))
        .collect();
    Ok(list)
}

#[server(GetPriceHistory, "/api", "GetCbor")]
pub async fn get_price_history(station_id: i32) -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    // simulate some time to acquire the informations
    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = DBStationPriceChange::load_station(station_id, conn);
    let list = list.into_iter()
        .map(|rs| StationPriceChange::from(rs))
        .collect();
    Ok(list)
}
