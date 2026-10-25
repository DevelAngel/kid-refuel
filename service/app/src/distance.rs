use crate::types::StationPriceChange;
use leptos::prelude::*;

pub fn use_browser_position() -> RwSignal<Option<(f64, f64)>> {
    let position = RwSignal::new(None);
    Effect::new(move |_| request_position(position));
    position
}

pub fn sort_by_distance(
    items: Vec<StationPriceChange>,
    position: Option<(f64, f64)>,
) -> Vec<(StationPriceChange, Option<f64>)> {
    let mut items = items
        .into_iter()
        .map(|item| {
            let distance = position.and_then(|position| {
                item.coordinates
                    .map(|coordinates| distance_km(position, coordinates))
            });
            (item, distance)
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| match (left.1, right.1) {
        (Some(left), Some(right)) => left.total_cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    items
}

pub fn format_distance(distance_km: f64) -> String {
    if distance_km < 10.0 {
        format!("{:.1} km", distance_km)
    } else {
        format!("{distance_km:.0} km")
    }
}

pub(crate) fn distance_km(from: (f64, f64), to: (f64, f64)) -> f64 {
    let (latitude, longitude) = (from.0.to_radians(), from.1.to_radians());
    let (other_latitude, other_longitude) = (to.0.to_radians(), to.1.to_radians());
    let latitude_delta = other_latitude - latitude;
    let longitude_delta = other_longitude - longitude;
    let haversine = (latitude_delta / 2.0).sin().powi(2)
        + latitude.cos() * other_latitude.cos() * (longitude_delta / 2.0).sin().powi(2);
    6_371.0 * 2.0 * haversine.sqrt().asin()
}

#[cfg(feature = "hydrate")]
fn request_position(position: RwSignal<Option<(f64, f64)>>) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::Position;

    let Ok(geolocation) = window().navigator().geolocation() else {
        return;
    };
    let on_success = Closure::once_into_js(move |value: Position| {
        let coordinates = value.coords();
        let _ = position.try_set(Some((coordinates.latitude(), coordinates.longitude())));
    });

    let _ = geolocation.get_current_position_with_error_callback(
        on_success.unchecked_ref(),
        None,
    );
}

#[cfg(not(feature = "hydrate"))]
fn request_position(_position: RwSignal<Option<(f64, f64)>>) {}

#[cfg(test)]
mod tests {
    use super::{distance_km, format_distance, sort_by_distance};
    use crate::types::StationPriceChange;
    use chrono::Utc;

    fn station(name: &str, coordinates: Option<(f64, f64)>) -> StationPriceChange {
        StationPriceChange {
            name: name.to_owned(),
            addr: String::new(),
            price: [1, 2, 3],
            updated: Utc::now(),
            coordinates,
        }
    }

    #[test]
    fn calculates_distance_and_formats_compactly() {
        assert!((distance_km((51.05, 13.74), (51.05, 13.74))).abs() < 0.001);
        assert_eq!(format_distance(4.25), "4.2 km");
        assert_eq!(format_distance(12.6), "13 km");
    }

    #[test]
    fn sorts_nearest_first_and_unknown_coordinates_last() {
        let sorted = sort_by_distance(
            vec![
                station("far", Some((52.0, 13.0))),
                station("unknown", None),
                station("near", Some((51.051, 13.74))),
            ],
            Some((51.05, 13.74)),
        );

        assert_eq!(sorted[0].0.name, "near");
        assert_eq!(sorted[1].0.name, "far");
        assert_eq!(sorted[2].0.name, "unknown");
        assert!(sorted[2].1.is_none());
    }

    #[test]
    fn preserves_input_order_without_a_position() {
        let sorted = sort_by_distance(
            vec![station("first", Some((1.0, 1.0))), station("second", None)],
            None,
        );

        assert_eq!(sorted[0].0.name, "first");
        assert_eq!(sorted[1].0.name, "second");
        assert!(sorted.iter().all(|item| item.1.is_none()));
    }
}
