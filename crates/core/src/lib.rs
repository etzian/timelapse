//! Timelapse Studio — core crate.
//!
//! Pure Rust domain logic, free of any `tauri` dependency (CLAUDE.md: "Keep
//! `crates/core` independent of Tauri"). The UI and the Tauri shell are thin
//! adapters over this module's API.

pub mod clean;
pub mod files;
pub mod import;

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
            Self::UnknownImage { id } => serde_json::json!({ "id": id }),
            Self::StoreTransaction => serde_json::json!({}),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
