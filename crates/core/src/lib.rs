//! Pure logic for Timelapse Studio. This crate never depends on `tauri`: logic in a
//! Tauri command is a smell. Errors cross IPC as `{ kind, args }` and are
//! turned into text by the UI through Fluent, so nothing here is user-facing prose.

use serde::Serialize;

/// Everything that can go wrong in this crate. `kind()` is the stable id the UI
/// looks up as `error-<kind>` in the `.ftl` files.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("name is empty")]
    NameEmpty,
    #[error("name is longer than {max} characters")]
    NameTooLong { max: usize },
}

impl Error {
    /// Stable machine id, matching a message id in `locales/*/errors.ftl`.
    pub fn kind(&self) -> &'static str {
        match self {
            Error::NameEmpty => "validation-required",
            Error::NameTooLong { .. } => "validation-too-long",
        }
    }

    /// Fluent arguments for the message (`{ $max }`).
    pub fn args(&self) -> serde_json::Value {
        match self {
            Error::NameEmpty => serde_json::json!({}),
            Error::NameTooLong { max } => serde_json::json!({ "max": max }),
        }
    }
}

pub const NAME_MAX: usize = 40;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Greeting {
    pub name: String,
}

/// The sample feature that proves the bridge end to end. Replace it.
pub fn greet(name: &str) -> Result<Greeting, Error> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::NameEmpty);
    }
    if name.chars().count() > NAME_MAX {
        return Err(Error::NameTooLong { max: NAME_MAX });
    }
    Ok(Greeting { name: name.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_trims_the_name() {
        assert_eq!(greet("  Ada ").unwrap().name, "Ada");
    }

    #[test]
    fn an_empty_name_is_a_validation_error() {
        assert_eq!(greet("   ").unwrap_err().kind(), "validation-required");
    }

    #[test]
    fn a_long_name_reports_the_limit() {
        let err = greet(&"x".repeat(NAME_MAX + 1)).unwrap_err();
        assert_eq!(err.kind(), "validation-too-long");
        assert_eq!(err.args()["max"], NAME_MAX);
    }
}
