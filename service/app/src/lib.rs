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
            <p>"Hello World!"</p>
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
