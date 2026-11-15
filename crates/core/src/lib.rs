//! Timelapse Studio — core crate.
//!
//! Pure Rust domain logic, free of any `tauri` dependency (CLAUDE.md: "Keep
//! `crates/core` independent of Tauri"). The UI and the Tauri shell are thin
//! adapters over this module's API.

pub mod adjust;
pub mod clean;
pub mod consolidate;
pub mod contact_sheet;
pub mod erase;
pub mod export;
pub mod files;
pub mod import;
pub mod layout;
pub mod sampling;

/// Errors that can cross the IPC boundary into the UI. Every variant maps to a stable
/// `kind` string used by the frontend to look up a Fluent key (`error-<kind>`).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("import root is not readable: {path}")]
    ImportRootUnreadable { path: String },

    #[error("image could not be read: {path}")]
    ImageUnreadable { path: String },

    #[error("database error at {path}")]
    DbMessage { path: String },

    #[error("import transaction failed")]
    StoreTransaction,

    #[error("image could not be decoded: {path}")]
    ImageUndecodable { path: String },

    #[error("no image with id {id}")]
    UnknownImage { id: i64 },

    #[error("invalid sampling configuration: {reason}")]
    InvalidSampling { reason: &'static str },

    #[error("no sampling run with id {id}")]
    UnknownRun { id: i64 },

    #[error("video export failed: {reason}")]
    Export { reason: &'static str },

    #[error("consolidation refused or failed: {reason}")]
    Consolidation { reason: &'static str },

    #[error("contact sheet refused or failed: {reason}")]
    ContactSheet { reason: &'static str },

    #[error("deflicker or stabilization refused or failed: {reason}")]
    Adjust { reason: &'static str },

    #[error("erase refused or failed: {reason}")]
    Erase { reason: &'static str },
}

impl Error {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ImportRootUnreadable { .. } => "import-root-unreadable",
            Self::ImageUnreadable { .. } => "image-unreadable",
            Self::DbMessage { .. } => "database",
            Self::StoreTransaction => "store-transaction",
            Self::ImageUndecodable { .. } => "image-undecodable",
            Self::UnknownImage { .. } => "unknown-image",
            Self::InvalidSampling { .. } => "invalid-sampling",
            Self::UnknownRun { .. } => "unknown-run",
            Self::Export { .. } => "export",
            Self::Consolidation { .. } => "consolidation",
            Self::ContactSheet { .. } => "contact-sheet",
            Self::Adjust { .. } => "adjust",
            Self::Erase { .. } => "erase",
        }
    }

    /// Per-variant arguments the UI may want to interpolate ({path}, etc.).
    /// Kept as a JSON value so it is serializable across IPC.
    pub fn args(&self) -> serde_json::Value {
        match self {
            Self::ImportRootUnreadable { path }
            | Self::ImageUnreadable { path }
            | Self::DbMessage { path }
            | Self::ImageUndecodable { path } => serde_json::json!({ "path": path }),
            Self::UnknownImage { id } | Self::UnknownRun { id } => serde_json::json!({ "id": id }),
            Self::InvalidSampling { reason }
            | Self::Export { reason }
            | Self::Consolidation { reason }
            | Self::ContactSheet { reason }
            | Self::Adjust { reason }
            | Self::Erase { reason } => {
                serde_json::json!({ "reason": reason })
            }
            Self::StoreTransaction => serde_json::json!({}),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
