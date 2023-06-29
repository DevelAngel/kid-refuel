use crate::types::AppRefuelStation;

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
        <div>
            <Suspense fallback=move || view! { cx, <p>"Loading Current Price List..."</p> }>
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
        <Outlet/>
    }
}

#[server(GetCurrentPrices, "/api", "GetCbor")]
pub async fn get_current_prices() -> Result<Vec<AppRefuelStation>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::*;

    // simulate some time to acquire the informations
    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = RefuelStationPriceChange::load_current(conn);
    let list = list.into_iter()
        .map(|rs| AppRefuelStation::from(rs))
        .collect();
    Ok(list)
}
