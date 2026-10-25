use crate::distance::format_distance;
use crate::types::StationPriceChange;

use chrono::prelude::*;
use leptos::prelude::*;

const MAX_STATIONS: usize = 6;
const WINDOW_WEEKS: i64 = 2;
const COLOURS: [(u8, u8, u8); 6] = [
    (34, 211, 238),
    (251, 191, 36),
    (251, 113, 133),
    (163, 230, 53),
    (167, 139, 250),
    (251, 146, 60),
];
const PRICE_STEP: f64 = 0.05;
const MAX_PRICE_TICKS: i64 = 10;
const CHART_WIDTH: f64 = 900.0;
const CHART_HEIGHT: f64 = 560.0;
const MARGIN_LEFT: f64 = 64.0;
const MARGIN_RIGHT: f64 = 16.0;
const MARGIN_TOP: f64 = 12.0;
const MARGIN_BOTTOM: f64 = 36.0;

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

/// Ticks on multiples of five cents covering `min` to `max`. The step grows in
/// multiples of five cents if the range needs more than `MAX_PRICE_TICKS` ticks.
fn price_ticks(min: f64, max: f64) -> Vec<f64> {
    let mut low = (min / PRICE_STEP + 1e-9).floor() as i64;
    let mut high = (max / PRICE_STEP - 1e-9).ceil() as i64;
    if high <= low {
        high = low + 1;
    }
    let mut step = 1;
    while (high - low) / step + 1 > MAX_PRICE_TICKS {
        step *= 2;
    }
    low = low.div_euclid(step) * step;
    high = (high + step - 1).div_euclid(step) * step;

    (low..=high)
        .step_by(step as usize)
        .map(|units| units as f64 * PRICE_STEP)
        .collect()
}

fn format_price(value: f64) -> String {
    format!("{value:.3}").replace('.', ",")
}

fn format_time(timestamp: DateTime<Utc>, local: bool, format: &str) -> String {
    if local {
        timestamp.with_timezone(&Local).format(format).to_string()
    } else {
        timestamp.format(format).to_string()
    }
}

/// Midnights of the time zone within `start` to `end`.
fn day_ticks<Tz: TimeZone>(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    tz: &Tz,
) -> Vec<DateTime<Utc>> {
    let mut ticks = Vec::new();
    let mut date = start.with_timezone(tz).date_naive();
    loop {
        let midnight = date
            .and_hms_opt(0, 0, 0)
            .and_then(|local| tz.from_local_datetime(&local).earliest())
            .map(|local| local.with_timezone(&Utc));
        match midnight {
            Some(midnight) if midnight > end => break,
            Some(midnight) if midnight >= start => ticks.push(midnight),
            _ => {}
        }
        match date.succ_opt() {
            Some(next) => date = next,
            None => break,
        }
    }
    ticks
}

fn step_path(points: impl Iterator<Item = (f64, f64)>) -> String {
    points
        .enumerate()
        .map(|(index, (x, y))| {
            if index == 0 {
                format!("M{x:.1},{y:.1}")
            } else {
                format!("H{x:.1} V{y:.1}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
fn ComparisonChart(comparison: Comparison) -> impl IntoView {
    let (Some(first), Some(last)) = (comparison.rows.first(), comparison.rows.last()) else {
        return view! { <p class="text-sm text-slate-400">"No price history available."</p> }
            .into_any();
    };
    let (start, end) = (first.timestamp, last.timestamp);
    let (min, max) = comparison
        .rows
        .iter()
        .flat_map(|row| row.prices.iter().copied())
        .fold((f64::MAX, f64::MIN), |(low, high), price| {
            (low.min(price), high.max(price))
        });
    let y_ticks = price_ticks(min, max);
    let low = y_ticks.first().copied().unwrap_or(min);
    let high = y_ticks.last().copied().unwrap_or(max);

    let plot_right = CHART_WIDTH - MARGIN_RIGHT;
    let plot_bottom = CHART_HEIGHT - MARGIN_BOTTOM;
    let label_x = MARGIN_LEFT - 8.0;
    let axis_label_y = plot_bottom + 20.0;
    let milliseconds = (end - start).num_milliseconds().max(1) as f64;
    let x_of = move |timestamp: DateTime<Utc>| {
        MARGIN_LEFT
            + (timestamp - start).num_milliseconds() as f64 / milliseconds
                * (plot_right - MARGIN_LEFT)
    };
    let y_of = move |price: f64| {
        plot_bottom - (price - low) / (high - low) * (plot_bottom - MARGIN_TOP)
    };

    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));

    let y_axis = y_ticks
        .iter()
        .map(|&tick| {
            let y = y_of(tick);
            view! {
                <line x1=MARGIN_LEFT x2=plot_right y1=y y2=y class="stroke-slate-800" />
                <text x=label_x y=y dy="0.35em" text-anchor="end" font-size="12" class="fill-slate-400">
                    {format_price(tick)}
                </text>
            }
        })
        .collect_view();

    let x_axis = move || {
        let local = hydrated.get();
        let ticks = if local {
            day_ticks(start, end, &Local)
        } else {
            day_ticks(start, end, &Utc)
        };
        ticks
            .into_iter()
            .map(|tick| {
                let x = x_of(tick);
                view! {
                    <line x1=x x2=x y1=MARGIN_TOP y2=plot_bottom class="stroke-slate-800" />
                    <text x=x y=axis_label_y text-anchor="middle" font-size="12" class="fill-slate-400">
                        {format_time(tick, local, "%a %d.")}
                    </text>
                }
            })
            .collect_view()
    };

    let colours = (0..comparison.labels.len())
        .map(|index| COLOURS[index % COLOURS.len()])
        .collect::<Vec<_>>();
    let lines = colours
        .iter()
        .enumerate()
        .map(|(index, &(red, green, blue))| {
            let path = step_path(comparison.rows.iter().filter_map(|row| {
                row.prices
                    .get(index)
                    .map(|&price| (x_of(row.timestamp), y_of(price)))
            }));
            view! {
                <path
                    d=path
                    fill="none"
                    stroke=format!("rgb({red}, {green}, {blue})")
                    stroke-width="2"
                    stroke-linejoin="round"
                    vector-effect="non-scaling-stroke"
                />
            }
        })
        .collect_view();

    let hovers = comparison
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let x = x_of(row.timestamp);
            let next = comparison
                .rows
                .get(index + 1)
                .map_or(plot_right, |next| x_of(next.timestamp));
            let timestamp = row.timestamp;
            let readings = comparison
                .labels
                .iter()
                .zip(&row.prices)
                .map(|(label, &price)| format!("{label}: {}", format_price(price)))
                .collect::<Vec<_>>()
                .join("\n");
            view! {
                <rect
                    x=x
                    y=MARGIN_TOP
                    width=(next - x).max(1.0)
                    height=plot_bottom - MARGIN_TOP
                    class="fill-transparent hover:fill-white/10"
                >
                    <title>
                        {move || {
                            format!(
                                "{}\n{readings}",
                                format_time(timestamp, hydrated.get(), "%a %d.%m. %H:%M"),
                            )
                        }}
                    </title>
                </rect>
            }
        })
        .collect_view();

    view! {
        <div class="w-full overflow-x-auto">
            <svg
                viewBox=format!("0 0 {CHART_WIDTH} {CHART_HEIGHT}")
                class="h-auto w-full min-w-[640px]"
                role="img"
                aria-label="Price comparison of the nearest stations"
            >
                {y_axis}
                {x_axis}
                <line x1=MARGIN_LEFT x2=MARGIN_LEFT y1=MARGIN_TOP y2=plot_bottom class="stroke-slate-600" />
                <line x1=MARGIN_LEFT x2=plot_right y1=plot_bottom y2=plot_bottom class="stroke-slate-600" />
                {lines}
                {hovers}
            </svg>
        </div>
        <ul class="mt-4 flex flex-wrap gap-x-6 gap-y-2 text-sm text-slate-300">
            {comparison
                .labels
                .into_iter()
                .zip(colours)
                .map(|(label, (red, green, blue))| {
                    view! {
                        <li class="flex items-center gap-2">
                            <span
                                class="inline-block h-1 w-5 rounded"
                                style=format!("background-color: rgb({red}, {green}, {blue})")
                            ></span>
                            {label}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
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
    use super::{NearbyStation, build_comparison, day_ticks, format_price, nearest_stations, price_ticks, step_path};
    use crate::types::StationPriceChange;
    use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};

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

    fn formatted(ticks: &[f64]) -> Vec<String> {
        ticks.iter().map(|tick| format_price(*tick)).collect()
    }

    #[test]
    fn formats_prices_with_three_decimals_and_a_comma() {
        assert_eq!(format_price(2.25), "2,250");
        assert_eq!(format_price(2.2), "2,200");
        assert_eq!(format_price(1.7999), "1,800");
    }

    #[test]
    fn places_price_ticks_on_multiples_of_five_cents() {
        assert_eq!(
            formatted(&price_ticks(2.234, 2.3)),
            vec!["2,200", "2,250", "2,300"]
        );
    }

    #[test]
    fn keeps_a_tick_that_equals_the_price() {
        assert_eq!(formatted(&price_ticks(2.25, 2.25)), vec!["2,250", "2,300"]);
    }

    #[test]
    fn widens_the_tick_step_for_a_large_price_range() {
        let ticks = price_ticks(1.0, 3.0);

        assert!(ticks.len() <= 10);
        assert!(ticks[0] <= 1.0 && ticks[ticks.len() - 1] >= 3.0);
        assert!(ticks.iter().all(|tick| {
            let units = tick / 0.05;
            (units - units.round()).abs() < 1e-6
        }));
    }

    #[test]
    fn finds_local_midnights_within_the_range() {
        let start = Utc.with_ymd_and_hms(2026, 10, 20, 10, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 10, 22, 10, 0, 0).unwrap();
        let berlin_summer = FixedOffset::east_opt(2 * 3600).unwrap();

        assert_eq!(
            day_ticks(start, end, &Utc),
            vec![
                Utc.with_ymd_and_hms(2026, 10, 21, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 10, 22, 0, 0, 0).unwrap(),
            ]
        );
        assert_eq!(
            day_ticks(start, end, &berlin_summer),
            vec![
                Utc.with_ymd_and_hms(2026, 10, 20, 22, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 10, 21, 22, 0, 0).unwrap(),
            ]
        );
    }

    #[test]
    fn draws_a_price_as_horizontal_then_vertical_steps() {
        let path = step_path([(0.0, 10.0), (5.0, 20.0), (8.0, 15.0)].into_iter());

        assert_eq!(path, "M0.0,10.0 H5.0 V20.0 H8.0 V15.0");
    }
}
