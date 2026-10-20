use leptos::prelude::*;
use serde::{Deserialize, Serialize};

pub const MAX_COLLECTION_RUNS: usize = 20;
pub const MAX_RESPONSE_EXCERPT_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CollectionDiagnostic {
    pub started_at: String,
    pub source_url: url::Url,
    #[serde(with = "status_code_serde")]
    pub http_status: Option<http::StatusCode>,
    pub http_duration: Option<std::time::Duration>,
    pub duration: std::time::Duration,
    pub result: String,
    pub prices_found: usize,
    pub rejected_entries: usize,
    pub error: Option<String>,
    pub response_excerpt: Option<String>,
}
#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
mod status_code_serde {
    use http::StatusCode;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(status: &Option<StatusCode>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        status.as_ref().map(StatusCode::as_u16).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<StatusCode>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<u16>::deserialize(deserializer)?
            .map(StatusCode::from_u16)
            .transpose()
            .map_err(serde::de::Error::custom)
    }
}

impl CollectionDiagnostic {
    pub fn response_excerpt(body: &str) -> Option<String> {
        if body.is_empty() {
            return None;
        }

        let mut end = body.len().min(MAX_RESPONSE_EXCERPT_BYTES);
        while !body.is_char_boundary(end) {
            end -= 1;
        }

        Some(body[..end].to_owned())
    }
}

#[cfg(feature = "ssr")]
mod server {
    use super::{CollectionDiagnostic, MAX_COLLECTION_RUNS};
    use std::sync::{Mutex, OnceLock};

    static COLLECTION_HISTORY: OnceLock<Mutex<Vec<CollectionDiagnostic>>> = OnceLock::new();

    fn history() -> &'static Mutex<Vec<CollectionDiagnostic>> {
        COLLECTION_HISTORY.get_or_init(|| Mutex::new(Vec::new()))
    }

    pub fn record(diagnostic: CollectionDiagnostic) {
        if let Ok(mut history) = history().lock() {
            history.push(diagnostic);
            let excess = history.len().saturating_sub(MAX_COLLECTION_RUNS);
            if excess > 0 {
                history.drain(..excess);
            }
        }
    }

    pub fn all() -> Vec<CollectionDiagnostic> {
        history()
            .lock()
            .map(|history| history.iter().rev().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(feature = "ssr")]
pub use server::record as record_collection_diagnostic;

#[server(endpoint = "get_collection_diagnostics")]
pub async fn get_collection_diagnostics() -> Result<Vec<CollectionDiagnostic>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        Ok(server::all())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{CollectionDiagnostic, MAX_RESPONSE_EXCERPT_BYTES};

    #[test]
    fn response_excerpt_is_limited_without_splitting_utf8() {
        let body = format!("{}🧙", "a".repeat(MAX_RESPONSE_EXCERPT_BYTES - 2));
        let excerpt = CollectionDiagnostic::response_excerpt(&body)
            .expect("non-empty body should produce an excerpt");

        assert_eq!(excerpt.len(), MAX_RESPONSE_EXCERPT_BYTES - 2);
        assert!(excerpt.is_char_boundary(excerpt.len()));
    }

    #[test]
    fn empty_response_has_no_excerpt() {
        assert_eq!(CollectionDiagnostic::response_excerpt(""), None);
    }
}
