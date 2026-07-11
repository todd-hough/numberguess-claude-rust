//! Shared serde helpers used by both the JSON API and Web UI request types.

use serde::{Deserialize, Deserializer};

/// Custom deserializer for optional u32 that treats empty strings as None.
///
/// This handles inputs where an empty field comes through as an empty string
/// rather than being omitted entirely (HTML forms always, and some API
/// clients). Also accepts numeric strings.
pub fn deserialize_option_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(ref s) if s.is_empty() => Ok(None),
        Some(s) => s.parse::<u32>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Lenient deserializer for optional i32 that treats empty or unparseable
/// input as None instead of failing the whole request.
///
/// Used for display-only state (e.g. the tracker bounds that round-trip
/// through hidden form fields): garbage input must degrade gracefully to
/// "no state" rather than reject an otherwise valid guess.
pub fn deserialize_lenient_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.and_then(|s| s.parse::<i32>().ok()))
}
