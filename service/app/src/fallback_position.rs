use leptos::prelude::*;
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[cfg(feature = "ssr")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FallbackPosition {
    pub latitude: f64,
    pub longitude: f64,
}

#[cfg(feature = "ssr")]
impl FromStr for FallbackPosition {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid =
            || format!("invalid fallback position '{value}'; expected LATITUDE,LONGITUDE");
        let (latitude, longitude) = value.split_once(',').ok_or_else(invalid)?;
        let latitude = latitude.parse::<f64>().map_err(|_| invalid())?;
        let longitude = longitude.parse::<f64>().map_err(|_| invalid())?;

        if !latitude.is_finite()
            || !(-90.0..=90.0).contains(&latitude)
            || !longitude.is_finite()
            || !(-180.0..=180.0).contains(&longitude)
        {
            return Err(format!(
                "invalid fallback position '{value}'; latitude must be between -90 and 90 and longitude between -180 and 180"
            ));
        }

        Ok(Self {
            latitude,
            longitude,
        })
    }
}

#[cfg(feature = "ssr")]
static FALLBACK_POSITION: std::sync::OnceLock<FallbackPosition> = std::sync::OnceLock::new();

#[cfg(feature = "ssr")]
pub fn set_fallback_position(position: FallbackPosition) -> Result<(), FallbackPosition> {
    FALLBACK_POSITION.set(position)
}

#[server(endpoint = "get_fallback_position")]
pub async fn get_fallback_position() -> Result<(f64, f64), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let position = FALLBACK_POSITION
            .get()
            .ok_or_else(|| ServerFnError::new("fallback position is not configured"))?;
        Ok((position.latitude, position.longitude))
    }

    #[cfg(not(feature = "ssr"))]
    Err(ServerFnError::new("fallback position is unavailable"))
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::FallbackPosition;
    use std::str::FromStr;

    #[test]
    fn parses_valid_coordinates() {
        assert_eq!(
            FallbackPosition::from_str("51.05,13.74"),
            Ok(FallbackPosition {
                latitude: 51.05,
                longitude: 13.74,
            })
        );
    }

    #[test]
    fn rejects_missing_or_invalid_coordinates() {
        for value in ["51.05", "91,0", "0,-181", "NaN,0", "0,inf", "1,2,3"] {
            assert!(FallbackPosition::from_str(value).is_err(), "{value}");
        }
    }
}
