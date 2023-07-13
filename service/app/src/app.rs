use crate::about::About;
use crate::all_prices::AllPrices;
use crate::current_prices::CurrentPrices;
use crate::price_history::PriceHistory;
use crate::price_history::StationPriceHistory;

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
        <Stylesheet id="refuel" href="/tailwind.css"/>
        <Router>
            <NavBar/>
            <main>
                <Routes>
                    <Route
                        path=""
                        view=move |cx| view! { cx,  <CurrentPrices/> }
                    >
                    </Route>
                    <Route
                        path="stations"
                        view=move |cx| view! { cx,  <PriceHistory/> }
                    >
                        <Route
                            path=":id"
                            view=move |cx| view! { cx,  <StationPriceHistory/> }
                        />
                        // fallback if :id is missing from URL
                        <Route
                            path=""
                            view=move |cx| view! { cx,  <p>"Select a station"</p> }
                        />
                    </Route>
                    <Route
                        path="all"
                        view=move |cx| view! { cx,  <AllPrices/> }
                    />
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
pub fn NavBar(cx: Scope) -> impl IntoView {
    use std::ops::Not;

    let (menu_hidden, toggle_menu) = create_signal(cx, true);

    view! {
        cx,
        <nav class="flex flex-wrap items-center justify-between w-full py-4 md:py-0 px-4 text-lg text-gray-700 bg-white">
            <A class="flex px-4 justify-between cursor-pointer" href="">
                <img class="w-6 h-6" alt="Refuel" src="/favicon-1.png"/>
                <span>"Refuel"</span>
            </A>
            // Hamburger Icon
            <svg xmlns="<http://www.w3.org/2000/svg>"
                 id="menu-button"
                 class="md:hidden block h-6 w-6 cursor-pointer"
                 fill="none" viewBox="0 0 24 24"
                 stroke="currentColor"
                 on:click=move |_| { toggle_menu.update(|m| *m = m.not()); }
            >
                <path stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 6h16M4 12h16M4 18h16"
                />
            </svg>
            // menu
            <div id="menu" class:hidden=move || menu_hidden.get() class="w-full md:flex md:w-auto md:items-center md:justify-between">
                <ul class="text-base text-gray-700 pt-4 md:flex md:pt-0 md:justify-between">
                    <li><A href="stations" class="py-2 block md:p-4 hover:text-purple-400">"Price History"</A></li>
                    <li><A href="all" class="py-2 block md:p-4 hover:text-purple-400">"All Prices"</A></li>
                    <li><A href="about" class="py-2 block md:p-4 hover:text-purple-400">"About"</A></li>
                </ul>
            </div>
        </nav>
    }
}
