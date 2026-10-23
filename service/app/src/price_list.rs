use crate::fuel_price::FuelPrice;
use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::prelude::*;


#[component]
pub(crate) fn LocalDateTime(timestamp: DateTime<Utc>, format: &'static str) -> impl IntoView {
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));

    view! {
        <time datetime=timestamp.to_rfc3339()>
            {move || {
                if hydrated.get() {
                    timestamp.with_timezone(&Local).format(format).to_string()
                } else {
                    timestamp.format(format).to_string()
                }
            }}
        </time>
    }
}

#[component]
pub(crate) fn PriceListItem(item: StationPriceChange) -> impl IntoView {
    let name = item.name;
    let updated = item.updated;
    let addr = item.addr;
    let price = item.price;

    view! {
        <article class="flex min-w-0 flex-col gap-4 rounded-xl border border-slate-800 bg-slate-900 p-5 shadow-sm transition hover:border-slate-700 hover:bg-slate-900/80">
            <div class="flex min-w-0 flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <h2 class="min-w-0 break-words text-lg font-semibold leading-6 text-white">
                    {name}
                </h2>
                <p class="shrink-0 text-3xl font-bold tracking-tight text-cyan-300">
                    <FuelPrice price=price />
                </p>
            </div>
            <address class="break-words text-sm not-italic leading-6 text-slate-400">
                {addr}
            </address>
            <div class="flex flex-wrap items-center justify-between gap-2 border-t border-slate-800 pt-3 text-xs font-medium text-slate-500">
                <span><LocalDateTime timestamp=updated format="%Y-%m-%d" /></span>
                <span><LocalDateTime timestamp=updated format="%H:%M" /></span>
            </div>
        </article>
    }
}
