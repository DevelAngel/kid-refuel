use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("fuel type mismatch: expected {expected}, found {actual}")]
    FuelTypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("fuel type could not be identified from page heading: {heading}")]
    MissingFuelType { heading: String },
    #[error("price list not found in source HTML")]
    MissingPriceList,
    #[error("unsupported price source: {url}")]
    UnsupportedSource { url: String },
    #[error("missing or empty {field} in source HTML")]
    MissingField { field: &'static str },
    #[error("invalid price value: {value}")]
    InvalidPrice { value: String },
    #[error("invalid timestamp: {value}")]
    InvalidUpdated { value: String },
    #[error("invalid selector {selector}: {reason}")]
    InvalidSelector {
        selector: &'static str,
        reason: String,
    },
    #[error("invalid {pattern} pattern: {reason}")]
    InvalidPattern {
        pattern: &'static str,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::ParseError;

    #[test]
    fn fuel_type_mismatch_has_a_specific_user_facing_message() {
        let error = ParseError::FuelTypeMismatch {
            expected: "E10",
            actual: "E5",
        };

        assert_eq!(
            error.to_string(),
            "fuel type mismatch: expected E10, found E5"
        );
    }

    #[test]
    fn missing_fuel_type_includes_the_page_heading() {
        let error = ParseError::MissingFuelType {
            heading: "Aktuelle Kraftstoff Preisübersicht".to_owned(),
        };

        assert_eq!(
            error.to_string(),
            "fuel type could not be identified from page heading: Aktuelle Kraftstoff Preisübersicht"
        );
    }
}
