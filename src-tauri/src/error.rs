use std::borrow::Cow;

use serde::ser::SerializeStruct;
use serde::Serialize;

/// 所有 Tauri 命令的统一应用错误类型。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Identity error: {0}")]
    Identity(#[from] crate::identity::IdentityError),
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

/// 结构化序列化：为前端提供 `{ kind: "...", message: "..." }` 格式。
/// 使用 `Cow` 避免对 String 变体的冗余 clone。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;

        let (kind, message): (&str, Cow<'_, str>) = match self {
            AppError::Database(e) => ("Database", e.to_string().into()),
            AppError::Io(e) => ("Io", e.to_string().into()),
            AppError::Identity(e) => ("Identity", e.to_string().into()),
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
