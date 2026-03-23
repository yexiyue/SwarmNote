use serde::ser::SerializeStruct;
use serde::Serialize;

/// Unified application error type for all Tauri commands.
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
}

/// Structured serialization: `{ kind: "...", message: "..." }` for frontend.
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
