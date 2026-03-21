use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::error::AppError;

pub struct DbState {
    #[allow(dead_code)]
    pub devices_db: DatabaseConnection,
    pub workspace_db: RwLock<Option<DatabaseConnection>>,
}

impl DbState {
    pub async fn workspace_db(&self) -> Result<WorkspaceDbGuard<'_>, AppError> {
        let guard = self.workspace_db.read().await;
        if guard.is_none() {
            return Err(AppError::NoWorkspaceDb);
        }
        Ok(WorkspaceDbGuard(guard))
    }
}

/// Wrapper that guarantees the inner `Option<DatabaseConnection>` is `Some`.
pub struct WorkspaceDbGuard<'a>(tokio::sync::RwLockReadGuard<'a, Option<DatabaseConnection>>);

impl WorkspaceDbGuard<'_> {
    pub fn conn(&self) -> &DatabaseConnection {
        // SAFETY: DbState::workspace_db checks is_none before constructing this guard
        self.0.as_ref().unwrap()
    }
}
