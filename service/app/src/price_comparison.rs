use crate::distance::format_distance;
use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::prelude::*;
use leptos_chartistry::{
    AspectRatio, AxisMarker, Chart, IntoInner, Legend, Line, Series, Step, TickLabels, Tooltip,
    XGridLine, YGridLine,
};

const MAX_STATIONS: usize = 6;
const WINDOW_WEEKS: i64 = 2;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NearbyStation {
    id: i32,
    label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Comparison {
    labels: Vec<String>,
    rows: Vec<ChartRow>,
}

#[derive(Clone, Debug, PartialEq)]
struct ChartRow {
    timestamp: DateTime<Utc>,
    prices: Vec<f64>,
}

pub(crate) fn nearest_stations(
    sorted: &[(StationPriceChange, Option<f64>)],
) -> Vec<NearbyStation> {
    sorted
        .iter()
        .filter_map(|(item, distance)| {
            distance.map(|distance| NearbyStation {
                id: item.station_id,
                label: format!("{}, {}", item.name, format_distance(distance)),
            })
        })
        .take(MAX_STATIONS)
        .collect()
}

/// The window ends at the newest change, so server and browser render the same chart.
/// A station without a change before its first one in the window is drawn flat from
/// its first known price.
fn build_comparison(stations: &[NearbyStation], changes: &[StationPriceChange]) -> Comparison {
    let Some(end) = changes.iter().map(|change| change.updated).max() else {
        return Comparison::default();
    };
    let since = end - chrono::Duration::weeks(WINDOW_WEEKS);

    let series = stations
        .iter()
        .filter_map(|station| {
            let mut points = changes
                .iter()
                .filter(|change| change.station_id == station.id)
                .map(|change| (change.updated.max(since), price_eur(change.price)))
                .collect::<Vec<_>>();
            points.sort_by_key(|(timestamp, _)| *timestamp);
            (!points.is_empty()).then(|| (station.label.clone(), points))
        })
        .collect::<Vec<_>>();

    let mut timestamps = series
        .iter()
        .flat_map(|(_, points)| points.iter().map(|(timestamp, _)| *timestamp))
        .collect::<Vec<_>>();
    timestamps.sort_unstable();
    timestamps.dedup();

    let rows = timestamps
        .into_iter()
        .map(|timestamp| ChartRow {
            timestamp,
            prices: series
                .iter()
                .map(|(_, points)| {
                    points
                        .iter()
                        .rev()
                        .find(|(changed, _)| *changed <= timestamp)
                        .or(points.first())
                        .map_or(0.0, |&(_, price)| price) // points is never empty
                })
                .collect(),
        })
        .collect();

    Comparison {
        labels: series.into_iter().map(|(label, _)| label).collect(),
        rows,
    }
}

fn price_eur(price: [u8; 3]) -> f64 {
    f64::from(price[0]) + f64::from(price[1]) / 100.0 + f64::from(price[2]) / 1000.0
}

#[component]
pub(crate) fn PriceComparison(stations: Memo<Option<Vec<NearbyStation>>>) -> impl IntoView {
    let changes = Resource::new(
        move || {
            stations
                .get()
                .map(|stations| stations.iter().map(|station| station.id).collect::<Vec<_>>())
        },
        |ids| async move {
            match ids {
                Some(ids) if !ids.is_empty() => get_price_comparison(ids).await,
                _ => Ok(Vec::new()),
            }
        },
    );

    view! {
        <section class="price-history-chart rounded-xl border border-slate-800 bg-slate-900 p-4 shadow-sm">
            <h2 class="mb-4 text-lg font-semibold text-white">"Price comparison"</h2>
            <Suspense fallback=move || view! { <p class="text-sm text-slate-400">"Loading price comparison..."</p> }>
                {move || {
                    changes.get().map(|result| match result {
                        Ok(changes) => {
                            let stations = stations.get().unwrap_or_default();
                            view! { <ComparisonChart comparison=build_comparison(&stations, &changes) /> }
                                .into_any()
                        }
                        Err(error) => view! {
                            <p class="text-sm text-red-200">
                                {format!("Failed to load price comparison: {error}")}
                            </p>
                        }
                            .into_any(),
                    })
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn ComparisonChart(comparison: Comparison) -> impl IntoView {
    if comparison.rows.is_empty() {
        return view! { <p class="text-sm text-slate-400">"No price history available."</p> }
            .into_any();
    }

    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let labels = move |format: &'static str| {
        TickLabels::timestamps().with_format(move |value, formatter| {
            if hydrated.get() {
                value.with_timezone(&Local).format(format).to_string()
            } else {
                formatter.format(value)
            }
        })
    };
    let time_labels = labels("%a %d.%m.");
    let mut tooltip = Tooltip::left_cursor();
    tooltip.x_ticks = labels("%a %d.%m. %H:%M");

    let series = comparison.labels.into_iter().enumerate().fold(
        Series::new(|row: &ChartRow| row.timestamp),
        |series, (index, label)| {
            series.line(
                Line::new(move |row: &ChartRow| row.prices[index])
                    .with_name(label)
                    .with_interpolation(Step::Horizontal),
            )
        },
    );
    let rows = comparison.rows;
    let data = Signal::derive(move || rows.clone());

    view! {
        <div class="w-full overflow-x-auto">
            <Chart
                aspect_ratio=AspectRatio::from_outer_ratio(900.0, 600.0)
                bottom=time_labels
                left=TickLabels::aligned_floats()
                right=Legend::middle()
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
    }
    .into_any()
}

#[server(endpoint = "get_price_comparison")]
pub async fn get_price_comparison(
    station_ids: Vec<i32>,
) -> Result<Vec<StationPriceChange>, ServerFnError> {
    use refuel_db::establish_connection_sqlite;
    use refuel_db::prelude::StationPriceChange as DBStationPriceChange;

    let mut conn =
        establish_connection_sqlite().map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = DBStationPriceChange::load_window(&station_ids, chrono::Utc::now(), &mut conn)
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let list = list.into_iter().map(StationPriceChange::from).collect();
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::{NearbyStation, build_comparison, nearest_stations};
    use crate::types::StationPriceChange;
    use chrono::{DateTime, Duration, TimeZone, Utc};

    fn end() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).unwrap()
    }

    fn change(station_id: i32, days_ago: i64, price: [u8; 3]) -> StationPriceChange {
        StationPriceChange {
            station_id,
            name: format!("Station {station_id}"),
            addr: String::new(),
            price,
            updated: end() - Duration::days(days_ago),
            coordinates: None,
        }
    }

    fn station(id: i32, distance: Option<f64>) -> (StationPriceChange, Option<f64>) {
        (change(id, 0, [1, 70, 0]), distance)
    }

    fn nearby(id: i32) -> NearbyStation {
        NearbyStation {
            id,
            label: format!("Station {id}"),
        }
    }

    #[test]
    fn selects_the_six_nearest_stations_in_order() {
        let sorted = (1..=10)
            .map(|id| station(id, Some(f64::from(id))))
            .collect::<Vec<_>>();

        let ids = nearest_stations(&sorted)
            .iter()
            .map(|station| station.id)
            .collect::<Vec<_>>();

        assert_eq!(ids, (1..=6).collect::<Vec<_>>());
    }

    #[test]
    fn selects_every_station_when_fewer_than_six_exist() {
        let sorted = vec![station(1, Some(1.0)), station(2, Some(2.0))];

        assert_eq!(nearest_stations(&sorted).len(), 2);
    }

    #[test]
    fn skips_stations_without_distance() {
        let sorted = vec![station(1, Some(1.0)), station(2, None)];

        let selected = nearest_stations(&sorted);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, 1);
    }

    #[test]
    fn labels_stations_with_name_and_distance() {
        let selected = nearest_stations(&[station(1, Some(4.25))]);

        assert_eq!(selected[0].label, "Station 1, 4.2 km");
    }

    #[test]
    fn keeps_a_price_until_the_next_change() {
        let changes = vec![
            change(1, 10, [1, 80, 0]),
            change(1, 2, [1, 70, 0]),
            change(2, 5, [1, 90, 0]),
        ];

        let comparison = build_comparison(&[nearby(1), nearby(2)], &changes);

        let prices = comparison
            .rows
            .iter()
            .map(|row| row.prices.clone())
            .collect::<Vec<_>>();
        assert_eq!(prices, vec![vec![1.8, 1.9], vec![1.8, 1.9], vec![1.7, 1.9]]);
        assert_eq!(comparison.labels, vec!["Station 1", "Station 2"]);
    }

    #[test]
    fn starts_a_price_from_before_the_window_at_the_window_start() {
        let changes = vec![change(1, 20, [1, 80, 0]), change(1, 0, [1, 70, 0])];

        let comparison = build_comparison(&[nearby(1)], &changes);

        assert_eq!(comparison.rows[0].timestamp, end() - Duration::weeks(2));
        assert_eq!(comparison.rows[0].prices, vec![1.8]);
    }

    #[test]
    fn draws_a_late_station_flat_from_its_first_price() {
        let changes = vec![change(1, 10, [1, 80, 0]), change(2, 2, [1, 90, 0])];

        let comparison = build_comparison(&[nearby(1), nearby(2)], &changes);

        assert_eq!(comparison.rows[0].prices, vec![1.8, 1.9]);
    }

    #[test]
    fn omits_stations_without_changes() {
        let comparison = build_comparison(&[nearby(1), nearby(2)], &[change(1, 3, [1, 80, 0])]);

        assert_eq!(comparison.labels, vec!["Station 1"]);
        assert!(comparison.rows.iter().all(|row| row.prices.len() == 1));
    }
}
