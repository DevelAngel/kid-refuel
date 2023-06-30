use crate::types::AppRefuelStation;

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
        <div>
            <Suspense fallback=move || view! { cx, <p>"Loading Price List with History..."</p> }>
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

#[server(GetAllPrices, "/api", "GetCbor")]
pub async fn get_all_prices() -> Result<Vec<AppRefuelStation>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::*;

    // simulate some time to acquire the informations
    //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = StationPriceChange::load_all(conn);
    let list = list.into_iter()
        .map(|rs| AppRefuelStation::from(rs))
        .collect();
    Ok(list)
}
