use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::prelude::*;
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart, IntoInner, Legend, Line, Series, Step, TickLabels, Tooltip,
    XGridLine, YGridLine,
};

#[derive(Clone)]
struct PricePoint {
    station_id: i32,
    station_name: String,
    timestamp: DateTime<Utc>,
    price: f64,
}

#[component]
pub(crate) fn PriceChart(
    data: Vec<StationPriceChange>,
    title: String,
) -> impl IntoView {
    let mut points = data
        .into_iter()
        .map(|item| PricePoint {
            station_name: item.name,
            station_id: item.station_id,
            timestamp: item.updated,
            price: f64::from(item.price[0])
                + f64::from(item.price[1]) / 100.0
                + f64::from(item.price[2]) / 1000.0,
        })
        .collect::<Vec<_>>();
    points.sort_by_key(|point| point.timestamp);
    let has_data = !points.is_empty();
    let stations = points
        .iter()
        .map(|point| point.station_id)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter_map(|station_id| {
            points
                .iter()
                .find(|point| point.station_id == station_id)
                .map(|point| (station_id, point.station_name.clone()))
        })
        .collect::<Vec<_>>();

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
    let series = Series::new(|point: &PricePoint| point.timestamp).lines(
        stations.into_iter().map(|(station_id, station_name)| {
            Line::new(move |point: &PricePoint| {
                if point.station_id == station_id {
                    point.price
                } else {
                    f64::NAN
                }
            })
            .with_name(station_name)
            .with_interpolation(Step::Horizontal)
        }),
    );

    view! {
        <section class="price-history-chart rounded-xl border border-slate-800 bg-slate-900 p-4 shadow-sm">
            <h2 class="mb-4 text-lg font-semibold text-white">{title}</h2>
            {if !has_data {
                view! { <p class="text-sm text-slate-400">"No price history available."</p> }.into_any()
            } else {
                view! {
                    <div class="w-full overflow-x-auto">
                        <Chart
                            right=Legend::end()
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
                            series=series
                            data=data
                        />
                    </div>
                }.into_any()
            }}
        </section>
    }
}
