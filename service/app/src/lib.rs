use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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
                    <tr>
                        <td>"MyESSO"</td>
                        <td>"Marienfelder Chaussee 171, 12349 Berlin"</td>
                        <td>1.78<sup>9</sup></td>
                        <td>"2023-06-04 15:00"</td>
                    </tr>
                    <tr>
                        <td>"MyJET"</td>
                        <td>"Rhinstr. 240, 13055 Berlin"</td>
                        <td>1.79<sup>8</sup></td>
                        <td>"2023-06-04 14:00"</td>
                    </tr>
                    <tr>
                        <td>"MyTotalEnergies"</td>
                        <td>"Landsberger Allee 376, 12681 Berlin"</td>
                        <td>1.81<sup>9</sup></td>
                        <td>"2023-06-04 14:30"</td>
                    </tr>
                    <tr>
                        <td>"MyAGIP ENI"</td>
                        <td>"Dietzgenstr. 127, 13158 Berlin"</td>
                        <td>1.80<sup>9</sup></td>
                        <td>"2023-06-04 17:15"</td>
                    </tr>
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
