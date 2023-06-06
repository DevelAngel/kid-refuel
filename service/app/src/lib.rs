use chrono::prelude::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

struct RefuelStation {
    name: String,
    addr: String,
    price: [u8; 3],
    updated: DateTime<Local>,
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
                    <img class="logo" src="/favicon.ico" />
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
    let list = vec![
        RefuelStation {
            name: "MyESSO".to_owned(),
            addr: "Marienfelder Chaussee 171, 12349 Berlin".to_owned(),
            price: [1, 78, 9],
            updated: Local.with_ymd_and_hms(2023, 6, 4, 15, 0, 0).unwrap(),
        },
        RefuelStation {
            name: "MyJET".to_owned(),
            addr: "Rhinstr. 240, 13055 Berlin".to_owned(),
            price: [1, 79, 8],
            updated: Local.with_ymd_and_hms(2023, 6, 4, 14, 0, 0).unwrap(),
        },
        RefuelStation {
            name: "MyTotalEnergies".to_owned(),
            addr: "Landsberger Allee 376, 12681 Berlin".to_owned(),
            price: [1, 81, 9],
            updated: Local.with_ymd_and_hms(2023, 6, 4, 14, 30, 0).unwrap(),
        },
        RefuelStation {
            name: "MyAGIP ENI".to_owned(),
            addr: "Dietzgenstr. 127, 13158 Berlin".to_owned(),
            price: [1, 80, 9],
            updated: Local.with_ymd_and_hms(2023, 6, 4, 17, 15, 0).unwrap(),
        },
    ];
    view! {
        cx,
        <div>
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
                    {
                        list.into_iter()
                            .map(|n| view! { cx,
                                <tr>
                                    <td>{n.name}</td>
                                    <td>{n.addr}</td>
                                    <td>{n.price[0]}","{n.price[1]}<sup>{n.price[2]}</sup></td>
                                    <td>{format!("{}", n.updated.format("%Y-%m-%d %H:%M"))}</td>
                                </tr>
                            })
                            .collect_view(cx)
                    }
                </tbody>
            </table>
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
