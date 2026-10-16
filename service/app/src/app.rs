use crate::about::About;
use crate::all_prices::AllPrices;
use crate::current_prices::CurrentPrices;
use crate::price_history::{PriceHistory, StationPriceHistory};

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Refuel WebApp"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="refuel" href="/tailwind.css"/>
        <Router>
            <NavBar/>
            <main>
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=CurrentPrices/>
                    <ParentRoute path=path!("/stations") view=PriceHistory>
                        <Route path=path!(":id") view=StationPriceHistory/>
                        <Route path=path!("") view=|| view! { <p>"Select a station"</p> }/>
                    </ParentRoute>
                    <Route path=path!("/all") view=AllPrices/>
                    <Route path=path!("/about") view=About/>
                </Routes>
            </main>
        </Router>
    }
}

#[component(transparent)]
pub fn NavBar() -> impl IntoView {
    use std::ops::Not;

    let (menu_hidden, toggle_menu) = signal(true);

    view! {
        <nav class="flex flex-wrap items-center justify-between w-full py-4 md:py-0 px-4 text-lg text-gray-700 bg-white">
            <a class="flex px-4 justify-between cursor-pointer" href="">
                <img class="w-6 h-6" alt="Refuel" src="/favicon-1.png"/>
                <span>"Refuel"</span>
            </a>
            <svg xmlns="http://www.w3.org/2000/svg"
                 id="menu-button"
                 class="md:hidden block h-6 w-6 cursor-pointer"
                 fill="none" viewBox="0 0 24 24"
                 stroke="currentColor"
                 on:click=move |_| { toggle_menu.update(|hidden| *hidden = hidden.not()); }
            >
                <path stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 6h16M4 12h16M4 18h16"
                />
            </svg>
            <div id="menu" class:hidden=move || menu_hidden.get() class="w-full md:flex md:w-auto md:items-center md:justify-between">
                <ul class="text-base text-gray-700 pt-4 md:flex md:pt-0 md:justify-between">
                    <li><a href="stations" class="py-2 block md:p-4 hover:text-purple-400">"Price History"</a></li>
                    <li><a href="all" class="py-2 block md:p-4 hover:text-purple-400">"All Prices"</a></li>
                    <li><a href="about" class="py-2 block md:p-4 hover:text-purple-400">"About"</a></li>
                </ul>
            </div>
        </nav>
    }
}
