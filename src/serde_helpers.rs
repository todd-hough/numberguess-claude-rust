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
