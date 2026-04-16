//! Unified error type for swarmnote-core.
//!
//! `AppError` is serialized as `{ kind, message }` when it crosses the Tauri
//! IPC boundary (the `Serialize` impl lives in the desktop shell to avoid a
//! `serde` hard dependency here — but the `kind` discriminants are stable
//! across the layer).

use std::borrow::Cow;

use serde::ser::SerializeStruct;
use serde::Serialize;

/// Core-layer error type. Every `AppResult<T>` returns this.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Identity error: {0}")]
    Identity(String),
    #[error("Keychain error: {0}")]
    Keychain(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("No workspace database open")]
    NoWorkspaceDb,
    #[error("App data directory not found")]
    NoAppDataDir,
    #[error("Folder is not empty: {0}")]
    FolderNotEmpty(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Path traversal detected: {0}")]
    PathTraversal(String),
    #[error("Name conflict: {0}")]
    NameConflict(String),
    #[error("No workspace open")]
    NoWorkspaceOpen,
    #[error("Network error: {0}")]
    Network(String),
    #[error("Pairing error: {0}")]
    Pairing(String),
    #[error("Window error: {0}")]
    Window(String),
    #[error("Yjs error: {0}")]
    Yjs(String),
    #[error("Document not open: {0}")]
    DocNotOpen(String),
}

impl AppError {
    pub fn node_not_running() -> Self {
        Self::Network("P2P node is not running".to_string())
    }
}

/// Structured serialization for frontend consumption: `{ kind, message }`.
///
/// `Cow` avoids redundant cloning of `String` variant payloads.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;

        let (kind, message): (&str, Cow<'_, str>) = match self {
            AppError::Database(e) => ("Database", e.to_string().into()),
            AppError::Io(e) => ("Io", e.to_string().into()),
            AppError::Identity(msg) => ("Identity", Cow::Borrowed(msg)),
            AppError::Keychain(msg) => ("Keychain", Cow::Borrowed(msg)),
            AppError::Config(msg) => ("Config", Cow::Borrowed(msg)),
            AppError::NoWorkspaceDb => {
                ("NoWorkspaceDb", Cow::Borrowed("No workspace database open"))
            }
            AppError::NoAppDataDir => (
                "NoAppDataDir",
                Cow::Borrowed("App data directory not found"),
            ),
            AppError::FolderNotEmpty(msg) => ("FolderNotEmpty", Cow::Borrowed(msg)),
            AppError::InvalidPath(msg) => ("InvalidPath", Cow::Borrowed(msg)),
            AppError::PathTraversal(msg) => ("PathTraversal", Cow::Borrowed(msg)),
            AppError::NameConflict(msg) => ("NameConflict", Cow::Borrowed(msg)),
            AppError::NoWorkspaceOpen => ("NoWorkspaceOpen", Cow::Borrowed("No workspace open")),
            AppError::Network(msg) => ("Network", Cow::Borrowed(msg)),
            AppError::Pairing(msg) => ("Pairing", Cow::Borrowed(msg)),
            AppError::Window(msg) => ("Window", Cow::Borrowed(msg)),
            AppError::Yjs(msg) => ("Yjs", Cow::Borrowed(msg)),
            AppError::DocNotOpen(msg) => ("DocNotOpen", Cow::Borrowed(msg)),
        };

        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &message)?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
