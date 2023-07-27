use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::*;

#[component]
pub(crate) fn PriceListItem(cx: Scope, item: StationPriceChange) -> impl IntoView {
    let name = &item.name;
    let addr = &item.addr;
    let price = &item.price;
    let updated = item.updated.with_timezone(&Local);
    view! {
        cx,
        <div class="flex flex-col bg-white text-gray-700 py-2 px-4">
            <div class="flex flex-row justify-between text-2xl">
                // station name
                <div class="flex-none px-1">{name}</div>
                // price
                <div class="flex-none px-1">{price[0]}","{price[1]}<sup>{price[2]}</sup></div>
            </div>
            <div class="flex flex-row justify-between flex-wrap text-base">
                // station address
                <address class="flex-1 px-1">{addr}</address>
                // updated date and time
                <div class="flex-none px-1">{format!("{}", updated.format("%Y-%m-%d"))}</div>
                <div class="flex-none px-1">{format!("{}", updated.format("%H:%M"))}</div>
            </div>
        </div>
    }
}
