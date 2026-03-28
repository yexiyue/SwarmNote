//! 工作区相关的 Tauri State 类型。

use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use super::commands::WorkspaceInfo;
use crate::error::AppError;

/// 数据库状态：全局 devices.db + per-window 工作区数据库连接。
pub struct DbState {
    pub devices_db: DatabaseConnection,
    workspace_dbs: RwLock<HashMap<String, DatabaseConnection>>,
}

impl DbState {
    pub fn new(
        devices_db: DatabaseConnection,
        workspace_dbs: HashMap<String, DatabaseConnection>,
    ) -> Self {
        Self {
            devices_db,
            workspace_dbs: RwLock::new(workspace_dbs),
        }
    }

    /// 获取指定窗口的工作区数据库连接。
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

    /// 注册指定窗口的工作区数据库连接。
    pub async fn insert_workspace_db(&self, label: &str, conn: DatabaseConnection) {
        self.workspace_dbs
            .write()
            .await
            .insert(label.to_owned(), conn);
    }

    /// 移除指定窗口的工作区数据库连接（drop 即关闭）。
    pub async fn remove_workspace_db(&self, label: &str) -> bool {
        self.workspace_dbs.write().await.remove(label).is_some()
    }
}

/// Per-window 工作区信息。
pub struct WorkspaceState {
    infos: RwLock<HashMap<String, WorkspaceInfo>>,
}

impl WorkspaceState {
    pub fn new(infos: HashMap<String, WorkspaceInfo>) -> Self {
        Self {
            infos: RwLock::new(infos),
        }
    }

    /// 获取指定窗口的工作区信息。
    pub async fn get(&self, label: &str) -> Option<WorkspaceInfo> {
        self.infos.read().await.get(label).cloned()
    }

    /// 注册指定窗口的工作区信息。
    pub async fn insert(&self, label: &str, info: WorkspaceInfo) {
        self.infos.write().await.insert(label.to_owned(), info);
    }

    /// 移除指定窗口的工作区信息。
    pub async fn remove(&self, label: &str) -> bool {
        self.infos.write().await.remove(label).is_some()
    }

    /// 查找已打开指定路径的窗口 label。
    pub async fn find_label_by_path(&self, path: &str) -> Option<String> {
        self.infos
            .read()
            .await
            .iter()
            .find(|(_, info)| info.path == path)
            .map(|(label, _)| label.clone())
    }

    /// 获取指定窗口的工作区路径。
    pub async fn workspace_path_for(&self, label: &str) -> Result<String, AppError> {
        self.infos
            .read()
            .await
            .get(label)
            .map(|ws| ws.path.clone())
            .ok_or(AppError::NoWorkspaceOpen)
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
        // SAFETY: label existence was verified in workspace_db_for before constructing this guard
        self.guard
            .get(&self.label)
            .expect("label was checked in workspace_db_for")
    }
}
