use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use super::commands::WorkspaceInfo;
use crate::error::AppError;

pub struct DbState {
    #[allow(dead_code)]
    pub devices_db: DatabaseConnection,
    /// Per-window workspace database connections, indexed by window label.
    pub workspace_dbs: RwLock<HashMap<String, DatabaseConnection>>,
}

/// Per-window workspace info, indexed by window label.
pub struct WorkspaceState(pub RwLock<HashMap<String, WorkspaceInfo>>);

impl DbState {
    /// Get the workspace database connection for a specific window.
    pub async fn workspace_db_for(&self, label: &str) -> Result<WorkspaceDbGuard<'_>, AppError> {
        let guard = self.workspace_dbs.read().await;
        if !guard.contains_key(label) {
            return Err(AppError::NoWorkspaceDb);
        }
        Ok(WorkspaceDbGuard {
            guard,
            label: label.to_owned(),
        })
    }
}

/// Guard that holds a read lock on the workspace_dbs HashMap and provides
/// access to a specific window's database connection.
pub struct WorkspaceDbGuard<'a> {
    guard: tokio::sync::RwLockReadGuard<'a, HashMap<String, DatabaseConnection>>,
    label: String,
}

impl WorkspaceDbGuard<'_> {
    pub fn conn(&self) -> &DatabaseConnection {
        self.guard
            .get(&self.label)
            .expect("WorkspaceDbGuard: label was checked in workspace_db_for but missing in guard")
    }
}
