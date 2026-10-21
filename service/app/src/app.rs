use crate::about::About;
use crate::all_prices::AllPrices;
use crate::current_prices::CurrentPrices;
use crate::price_history::{PriceHistory, StationPriceHistory};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{ParentRoute, Route, Router, Routes};
use leptos_router::hooks::use_location;
use leptos_router::path;

const VIEWS: [(&str, &str); 4] = [
    ("Current prices", "/"),
    ("Price history", "/stations"),
    ("All prices", "/all"),
    ("About", "/about"),
];

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Refuel WebApp"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Stylesheet id="refuel" href="/pkg/refuel.css"/>
        <Router>
            <NavBar/>
            <main class="min-h-screen bg-slate-950 px-4 py-6 text-slate-100 sm:px-6 sm:py-8 lg:px-8">
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
            <Footer/>
        </Router>
    }
}

#[component(transparent)]
pub fn NavBar() -> impl IntoView {
    let pathname = use_location().pathname;

    let current_index = move || {
        let path = pathname.get();
        if path.starts_with("/stations") {
            1
        } else if path == "/all" {
            2
        } else if path == "/about" {
            3
        } else {
            0
        }
    };

    let previous_href = move || {
        let index = current_index();
        VIEWS[(index + VIEWS.len() - 1) % VIEWS.len()].1
    };
    let next_href = move || {
        let index = current_index();
        VIEWS[(index + 1) % VIEWS.len()].1
    };

    view! {
        <header class="border-b border-slate-800 bg-slate-900 text-slate-100 shadow-lg">
            <div class="mx-auto max-w-7xl px-3 py-3 sm:px-6 sm:py-4 lg:px-8">
                <div class="flex items-center justify-between gap-3">
                    <a class="flex min-h-11 min-w-0 items-center gap-2 rounded-lg pr-2 text-lg font-semibold tracking-tight text-white focus:outline-none focus:ring-2 focus:ring-cyan-400 sm:gap-3 sm:text-xl" href="/">
                        <img class="h-8 w-8 shrink-0 rounded-lg" alt="Refuel" src="/favicon-1.png"/>
                        <span>"Refuel"</span>
                    </a>
                    <div class="flex items-center gap-2 sm:gap-3">
                        <a
                            class="inline-flex h-11 w-11 items-center justify-center rounded-xl border border-slate-700 text-xl text-slate-200 transition hover:bg-slate-800 hover:text-cyan-300 focus:outline-none focus:ring-2 focus:ring-cyan-400"
                            href=previous_href
                            aria-label=move || format!("Previous view: {}", VIEWS[(current_index() + VIEWS.len() - 1) % VIEWS.len()].0)
                            title="Previous view"
                        >
                            <span aria-hidden="true">"←"</span>
                        </a>
                        <div class="min-w-0 text-center">
                            <p class="text-xs font-semibold uppercase tracking-widest text-cyan-400">"View"</p>
                            <h1 class="max-w-36 truncate text-sm font-bold text-white sm:max-w-none sm:text-base">
                                {move || VIEWS[current_index()].0}
                            </h1>
                        </div>
                        <a
                            class="inline-flex h-11 w-11 items-center justify-center rounded-xl border border-slate-700 text-xl text-slate-200 transition hover:bg-slate-800 hover:text-cyan-300 focus:outline-none focus:ring-2 focus:ring-cyan-400"
                            href=next_href
                            aria-label=move || format!("Next view: {}", VIEWS[(current_index() + 1) % VIEWS.len()].0)
                            title="Next view"
                        >
                            <span aria-hidden="true">"→"</span>
                        </a>
                    </div>
                </div>
                <nav class="mt-3" aria-label="Main navigation">
                    <ul class="grid grid-cols-4 gap-2">
                        {VIEWS.iter().enumerate().map(|(index, (title, href))| {
                            let title = *title;
                            let href = *href;
                            view! {
                                <li>
                                    <a
                                        class="flex min-h-11 items-center justify-center rounded-xl border px-2 py-2 text-center text-xs font-semibold transition focus:outline-none focus:ring-2 focus:ring-cyan-400 sm:text-sm"
                                        class:border-cyan-400=move || current_index() == index
                                        class:bg-cyan-950=move || current_index() == index
                                        class:text-cyan-200=move || current_index() == index
                                        class:border-slate-800=move || current_index() != index
                                        class:text-slate-400=move || current_index() != index
                                        href=href
                                        aria-current=move || if current_index() == index { "page" } else { "false" }
                                    >
                                        {title}
                                    </a>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                </nav>
            </div>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-slate-900 bg-slate-950 px-4 py-4 text-center text-xs text-slate-600">
            {match option_env!("KID_GIT_COMMIT") { Some(commit) => format!("kid refuel {} ({commit})", env!("CARGO_PKG_VERSION")), None => concat!("kid refuel ", env!("CARGO_PKG_VERSION")).to_owned() }}
        </footer>
    }
}
