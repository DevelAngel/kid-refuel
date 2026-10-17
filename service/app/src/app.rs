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
            <main class="min-h-screen bg-slate-950 px-4 py-8 text-slate-100 sm:px-6 lg:px-8">
                <div class="mx-auto w-full max-w-7xl">
                    <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=CurrentPrices/>
                    <ParentRoute path=path!("/stations") view=PriceHistory>
                        <Route path=path!(":id") view=StationPriceHistory/>
                        <Route path=path!("") view=|| view! { <p>"Select a station"</p> }/>
                    </ParentRoute>
                    <Route path=path!("/all") view=AllPrices/>
                    <Route path=path!("/about") view=About/>
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

#[component(transparent)]
pub fn NavBar() -> impl IntoView {

    let (menu_hidden, toggle_menu) = signal(true);

    view! {
        <nav class="border-b border-slate-700 bg-slate-900 text-slate-100 shadow-lg">
            <div class="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-4 px-4 py-4 sm:px-6 lg:px-8">
                <a class="flex items-center gap-3 text-xl font-semibold tracking-tight text-white" href="/">
                    <img class="h-8 w-8 rounded-lg" alt="Refuel" src="/favicon-1.png"/>
                    <span>"Refuel"</span>
                </a>
                <button
                    type="button"
                    class="inline-flex items-center justify-center rounded-lg border border-slate-700 p-2 text-slate-200 hover:bg-slate-800 focus:outline-none focus:ring-2 focus:ring-cyan-400 md:hidden"
                    aria-label="Toggle navigation"
                    aria-expanded=move || (!menu_hidden.get()).to_string()
                    on:click=move |_| { toggle_menu.update(|hidden| *hidden = !*hidden); }
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                    </svg>
                </button>
                <div class="hidden w-full md:block md:w-auto" class:hidden=move || menu_hidden.get()>
                    <ul class="flex flex-col gap-1 text-sm font-medium md:flex-row md:items-center md:gap-2">
                        <li><a href="/stations" class="block rounded-lg px-4 py-3 text-slate-300 transition hover:bg-slate-800 hover:text-cyan-300 focus:outline-none focus:ring-2 focus:ring-cyan-400 md:py-2">"Price History"</a></li>
                        <li><a href="/all" class="block rounded-lg px-4 py-3 text-slate-300 transition hover:bg-slate-800 hover:text-cyan-300 focus:outline-none focus:ring-2 focus:ring-cyan-400 md:py-2">"All Prices"</a></li>
                        <li><a href="/about" class="block rounded-lg px-4 py-3 text-slate-300 transition hover:bg-slate-800 hover:text-cyan-300 focus:outline-none focus:ring-2 focus:ring-cyan-400 md:py-2">"About"</a></li>
                    </ul>
                </div>
            </div>
        </nav>
    }
}
