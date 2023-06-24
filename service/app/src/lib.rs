use chrono::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppRefuelStation {
    name: String,
    addr: String,
    price: [u8; 3],
    updated: DateTime<Utc>,
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);

    view! {
        cx,
        <Title text="Refuel WebApp"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="picnic" href="/picnic.min.css"/>
        <Stylesheet id="leptos" href="/pkg/refuel.css"/>
        <Router>
            <nav>
                <A class="brand" href="">
                    <img class="logo" src="/favicon-1.png" />
                    <span>"Refuel"</span>
                </A>
                // responsive
                <input id="bmenub" type="checkbox" class="show"/>
                <label for="bmenub" class="burger pseudo button">"MENU"</label>
                // menu
                <div class="menu">
                    <A href="about" class="pseudo button icon-picture">"About"</A>
                </div>
            </nav>
            <main>
                <Routes>
                    <Route
                        path=""
                        view=move |cx| view! { cx,  <Home/> }
                    >
                    </Route>
                    <Route
                        path="about"
                        view=move |cx| view! { cx,  <About/> }
                    />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Home(cx: Scope) -> impl IntoView {
    let list_current = create_resource(
        cx,
        || (), //< run once
        |_| async move {
            get_current_prices().await.unwrap()
        },
    );

    let list_all = create_resource(
        cx,
        || (), //< run once
        |_| async move {
            get_all_prices().await.unwrap()
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
                        {move || { list_current.read(cx).map(|list| list.into_iter()
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
                        {move || { list_all.read(cx).map(|list| list.into_iter()
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

#[component]
fn About(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div>
            <p>"It's me!"</p>
        </div>
    }
}

#[cfg(feature = "ssr")]
use refuel_db::prelude::*;

#[cfg(feature = "ssr")]
impl From<RefuelStationPriceChange> for AppRefuelStation {
    fn from(src: RefuelStationPriceChange) -> Self {
        Self {
            name: src.name,
            addr: src.addr,
            price: src.price,
            updated: src.updated,
        }
    }
}

#[server(GetCurrentPrices, "/api", "GetCbor")]
pub async fn get_current_prices() -> Result<Vec<AppRefuelStation>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;

    // simulate some time to acquire the informations
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let conn = &mut establish_connection_sqlite();
    let list = RefuelStationPriceChange::load_current(conn);
    let list = list.into_iter()
        .map(|rs| AppRefuelStation::from(rs))
        .collect();
    Ok(list)
}

#[server(GetAllPrices, "/api", "GetCbor")]
pub async fn get_all_prices() -> Result<Vec<AppRefuelStation>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::*;

    // simulate some time to acquire the informations
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let conn = &mut establish_connection_sqlite();
    let list = RefuelStationPriceChange::load_all(conn);
    let list = list.into_iter()
        .map(|rs| AppRefuelStation::from(rs))
        .collect();
    Ok(list)
}
