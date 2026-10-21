use crate::fuel_price::FuelPrice;
use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::prelude::*;

#[component]
pub(crate) fn PriceListItem(item: StationPriceChange) -> impl IntoView {
    let name = item.name;
    let addr = item.addr;
    let price = item.price;
    let updated = item.updated.with_timezone(&Local);

    view! {
        <article class="flex min-w-0 flex-col gap-4 rounded-xl border border-slate-800 bg-slate-900 p-5 shadow-sm transition hover:border-slate-700 hover:bg-slate-900/80">
            <div class="flex min-w-0 flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <h2 class="min-w-0 break-words text-lg font-semibold leading-6 text-white">{name}</h2>
                <p class="shrink-0 text-3xl font-bold tracking-tight text-cyan-300"><FuelPrice price=price/></p>
            </div>
            <address class="break-words text-sm not-italic leading-6 text-slate-400">{addr}</address>
            <div class="flex flex-wrap items-center justify-between gap-2 border-t border-slate-800 pt-3 text-xs font-medium text-slate-500">
                <span>{updated.format("%Y-%m-%d").to_string()}</span>
                <span>{updated.format("%H:%M").to_string()}</span>
            </div>
        </article>
    }
}
