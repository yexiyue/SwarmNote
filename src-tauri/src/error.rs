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
    Identity(String),
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
}

/// 结构化序列化：为前端提供 `{ kind: "...", message: "..." }` 格式。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;

        let (kind, message) = match self {
            AppError::Database(e) => ("Database", e.to_string()),
            AppError::Io(e) => ("Io", e.to_string()),
            AppError::Identity(msg) => ("Identity", msg.clone()),
            AppError::NoWorkspaceDb => ("NoWorkspaceDb", self.to_string()),
            AppError::NoAppDataDir => ("NoAppDataDir", self.to_string()),
            AppError::FolderNotEmpty(msg) => ("FolderNotEmpty", msg.clone()),
            AppError::InvalidPath(msg) => ("InvalidPath", msg.clone()),
            AppError::PathTraversal(msg) => ("PathTraversal", msg.clone()),
            AppError::NameConflict(msg) => ("NameConflict", msg.clone()),
            AppError::NoWorkspaceOpen => ("NoWorkspaceOpen", self.to_string()),
            AppError::Network(msg) => ("Network", msg.clone()),
        };

        state.serialize_field("kind", kind)?;
        state.serialize_field("message", &message)?;
        state.end()
    }
}

impl From<crate::identity::IdentityError> for AppError {
    fn from(e: crate::identity::IdentityError) -> Self {
        AppError::Identity(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
