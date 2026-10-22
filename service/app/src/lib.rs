pub mod prelude;

mod about;
mod all_prices;
mod app;
mod collection_diagnostics;
mod current_prices;
mod fuel_price;
mod price_history;
mod price_list;
mod types;

#[cfg(feature = "ssr")]
use crate::prelude::App;
#[cfg(feature = "ssr")]
use leptos::config::LeptosOptions;
#[cfg(feature = "ssr")]
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_meta::MetaTags;

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="manifest" href="manifest.json" />
                <meta name="theme-color" content="#0891b2" />
                <meta name="apple-mobile-web-app-capable" content="yes" />
                <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
                <link rel="apple-touch-icon" href="favicon-192x192.png" />
                <link rel="icon" href="favicon.ico" />
                <link rel="icon" type="image/png" href="favicon-16x16.png" />
                <link rel="icon" type="image/png" href="favicon-32x32.png" />
                <link rel="icon" type="image/png" href="favicon-64x46.png" />
                <link rel="icon" type="image/png" href="favicon-128x128.png" />
                <script>
                    "if ('serviceWorker' in navigator) { navigator.serviceWorker.register('/sw.js'); }"
                </script>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
