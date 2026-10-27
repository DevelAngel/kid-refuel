use crate::fuel_price::FuelPrice;
use crate::price_list::LocalDateTime;
use crate::types::{Station, StationPriceChange};

use chrono::prelude::*;
use leptos::prelude::*;
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart, IntoInner, Line, Series, Step, TickLabels, Tooltip, XGridLine,
    YGridLine,
};
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::use_params_map;

#[component]
pub fn PriceHistory() -> impl IntoView {
    view! {
        <section class="space-y-6">
            <header class="space-y-2">
                <p class="text-sm font-semibold uppercase tracking-widest text-cyan-400">
                    "Station explorer"
                </p>
                <h1 class="text-3xl font-bold tracking-tight text-white sm:text-4xl">
                    "Price history"
                </h1>
                <p class="max-w-2xl text-sm leading-6 text-slate-400 sm:text-base">
                    "Choose a station to review its recorded prices and updates."
                </p>
            </header>
            <StationList />
            <Outlet />
        </section>
    }
}

#[component]
pub fn StationList() -> impl IntoView {
    let list = Resource::new(|| (), |_| async move { get_stations().await });

    view! {
        <div class="overflow-x-auto rounded-xl border border-slate-800 bg-slate-900 shadow-sm">
            <Suspense fallback=move || view! { <p>"Loading Station List..."</p> }>
                <table class="min-w-full divide-y divide-slate-800 text-left text-sm">
                    <thead>
                        <tr>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "ID"
                            </th>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "Refuel Station"
                            </th>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "Address"
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            list.get()
                                .map(|result| match result {
                                    Ok(list) => {
                                        list.into_iter()
                                            .map(|station| {
                                                view! {
                                                    <tr>
                                                        <td class="border-t border-slate-800 px-5 py-4 text-cyan-300">
                                                            <A href=station.id.to_string()>{station.id}</A>
                                                        </td>
                                                        <td class="border-t border-slate-800 px-5 py-4 font-medium text-white">
                                                            <div>{station.name}</div>
                                                        </td>
                                                        <td class="border-t border-slate-800 px-5 py-4 text-slate-400">
                                                            <address class="not-italic">{station.addr}</address>
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                    Err(error) => {
                                        view! {
                                            <p>{format!("Failed to load stations: {error}")}</p>
                                        }
                                            .into_any()
                                    }
                                })
                        }}
                    </tbody>
                </table>
            </Suspense>
        </div>
    }
}

#[component]
pub fn StationPriceHistory() -> impl IntoView {
    let params = use_params_map();
    let list = Resource::new(
        move || params.read().get("id").unwrap_or_default(),
        move |id| async move {
            match id.parse::<i32>() {
                Ok(station_id) => get_price_history(station_id).await,
                Err(error) => Err(ServerFnError::new(format!("Invalid station ID: {error}"))),
            }
        },
    );
    view! {
        <Suspense fallback=move || {
            view! { <p class="text-slate-400">"Loading price chart..."</p> }
        }>
            {move || {
                list.get()
                    .map(|result| match result {
                        Ok(data) => view! { <PriceChart data=data /> }.into_any(),
                        Err(_) => ().into_any(),
                    })
            }}
        </Suspense>

        <div class="overflow-x-auto rounded-xl border border-slate-800 bg-slate-900 shadow-sm">
            <Suspense fallback=move || view! { <p>"Loading Station Price History..."</p> }>
                <table class="min-w-full divide-y divide-slate-800 text-left text-sm">
                    <thead>
                        <tr>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "Refuel Station"
                            </th>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "Address"
                            </th>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "Price"
                            </th>
                            <th class="whitespace-nowrap px-5 py-3 font-semibold text-slate-400">
                                "Updated"
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            list.get()
                                .map(|result| match result {
                                    Ok(list) => {
                                        list.into_iter()
                                            .map(|item| {
                                                view! {
                                                    <tr>
                                                        <td class="border-t border-slate-800 px-5 py-4 font-medium text-white">
                                                            <div>{item.name}</div>
                                                        </td>
                                                        <td class="border-t border-slate-800 px-5 py-4 text-slate-400">
                                                            <address class="not-italic">{item.addr}</address>
                                                        </td>
                                                        <td class="border-t border-slate-800 px-5 py-4 text-2xl font-bold text-cyan-300">
                                                            <FuelPrice price=item.price />
                                                        </td>
                                                        <td class="border-t border-slate-800 px-5 py-4 whitespace-nowrap text-slate-400">
                                                            <LocalDateTime
                                                                timestamp=item.updated
                                                                format="%Y-%m-%d %H:%M"
                                                            />
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                    Err(error) => {
                                        view! {
                                            <p>{format!("Failed to load station history: {error}")}</p>
                                        }
                                            .into_any()
                                    }
                                })
                        }}
                    </tbody>
                </table>
            </Suspense>
        </div>
    }
}
#[derive(Clone)]
struct PricePoint {
    timestamp: DateTime<Utc>,
    price: f64,
}

#[component]
fn PriceChart(data: Vec<StationPriceChange>) -> impl IntoView {
    let mut points = data
        .into_iter()
        .map(|item| PricePoint {
            timestamp: item.updated,
            price: f64::from(item.price[0])
                + f64::from(item.price[1]) / 100.0
                + f64::from(item.price[2]) / 1000.0,
        })
        .collect::<Vec<_>>();
    points.sort_by_key(|point| point.timestamp);
    let has_data = !points.is_empty();

    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let time_labels = TickLabels::timestamps().with_format(move |value, formatter| {
        if hydrated.get() {
            value.with_timezone(&Local).format("%a %H:%M").to_string()
        } else {
            formatter.format(value)
        }
    });
    let mut tooltip = Tooltip::left_cursor();
    tooltip.x_ticks = time_labels.clone();
    let data = Signal::derive(move || points.clone());
    view! {
        <section class="price-history-chart rounded-xl border border-slate-800 bg-slate-900 p-4 shadow-sm">
            <h2 class="mb-4 text-lg font-semibold text-white">"Price trend"</h2>
            {if !has_data {
                view! { <p class="text-sm text-slate-400">"No price history available."</p> }
                    .into_any()
            } else {
                view! {
                    <div class="w-full overflow-x-auto">
                        <Chart
                            aspect_ratio=AspectRatio::from_outer_ratio(900.0, 360.0)
                            bottom=time_labels
                            left=TickLabels::aligned_floats()
                            inner=[
                                AxisMarker::left_edge().into_inner(),
                                AxisMarker::bottom_edge().into_inner(),
                                XGridLine::default().into_inner(),
                                YGridLine::default().into_inner(),
                            ]
                            tooltip=tooltip
                            series=Series::new(|point: &PricePoint| point.timestamp)
                                .line(
                                    Line::new(|point: &PricePoint| point.price)
                                        .with_name("Price (EUR/L)")
                                        .with_interpolation(Step::Horizontal),
                                )
                            data=data
                        />
                    </div>
                }
                    .into_any()
            }}
        </section>
    }
}

#[server(endpoint = "get_stations")]
pub async fn get_stations() -> Result<Vec<Station>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::Station as DBStation;

    let mut conn =
        establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list =
        DBStation::load_all(&mut conn).map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter().map(Station::from).collect();
    Ok(list)
}

#[server(endpoint = "get_price_history")]
pub async fn get_price_history(station_id: i32) -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn =
        establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_station(station_id, &mut conn)
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter().map(StationPriceChange::from).collect();
    Ok(list)
}
