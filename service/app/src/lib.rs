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
        <Stylesheet id="leptos" href="/pkg/refuel.css"/>
        <div id="root">
            <Router>
                <nav>
                    <A href="">"Home"</A>
                    <A href="about">"About"</A>
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
        </div>
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
