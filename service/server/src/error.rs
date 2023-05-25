use thiserror::Error;

use regex::Regex;
use scraper::Selector;
use std::num;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid price detected\nregex: {regex}\nhtml:\n{html}\n")]
    InvalidPriceError { html: String, regex: Regex },
    #[error("invalid updated timestamp detected\nregex: {regex}\nhtml:\n{html}\n")]
    InvalidUpdatedError { html: String, regex: Regex },
    #[error("html select error\nselector: {selector:?}\nhtml:\n{html}\n")]
    HtmlSelectError { html: String, selector: Selector },
    #[error("regex mismatch error\nregex: {regex}\nhtml:\n{html}\n")]
    RegexMismatchError { html: String, regex: Regex },
    #[error("string to number convertion error")]
    ParseIntError(#[from] num::ParseIntError),
}
