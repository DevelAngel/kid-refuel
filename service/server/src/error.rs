use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("price list not found in source HTML")]
    MissingPriceList,
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
