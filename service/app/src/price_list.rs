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
        <div class="flex flex-col bg-white text-gray-700 py-2 px-4">
            <div class="flex flex-row justify-between text-2xl">
                <div class="flex-none px-1">{name}</div>
                <div class="flex-none px-1">{price[0]}","{price[1]}<sup>{price[2]}</sup></div>
            </div>
            <div class="flex flex-row justify-between flex-wrap text-base">
                <address class="flex-1 px-1">{addr}</address>
                <div class="flex-none px-1">{updated.format("%Y-%m-%d").to_string()}</div>
                <div class="flex-none px-1">{updated.format("%H:%M").to_string()}</div>
            </div>
        </div>
    }
}
