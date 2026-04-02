//! 工作区相关的 Tauri State 类型。
//!
//! 主键为 workspace UUID，window label 作为辅助索引。

use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::commands::WorkspaceInfo;
use crate::error::AppError;

// ── DbState ──

/// 数据库状态：全局 devices.db + per-workspace 数据库连接。
pub struct DbState {
    pub devices_db: DatabaseConnection,
    /// 主索引：workspace UUID → DatabaseConnection
    workspace_dbs: RwLock<HashMap<Uuid, DatabaseConnection>>,
    /// 辅助索引：window label → workspace UUID
    label_to_uuid: RwLock<HashMap<String, Uuid>>,
}

impl DbState {
    pub fn new(devices_db: DatabaseConnection) -> Self {
        Self {
            devices_db,
            workspace_dbs: RwLock::new(HashMap::new()),
            label_to_uuid: RwLock::new(HashMap::new()),
        }
    }

    /// 通过 UUID 获取工作区数据库连接。
    pub async fn workspace_db(&self, uuid: &Uuid) -> Result<WorkspaceDbGuard<'_>, AppError> {
        let guard = self.workspace_dbs.read().await;
        if !guard.contains_key(uuid) {
            return Err(AppError::NoWorkspaceDb);
        }
        Ok(WorkspaceDbGuard { guard, key: *uuid })
    }

    /// 通过 window label 获取工作区数据库连接（Tauri 命令便捷方法）。
    pub async fn workspace_db_by_label(
        &self,
        label: &str,
    ) -> Result<WorkspaceDbGuard<'_>, AppError> {
        let uuid = self.resolve_uuid(label).await?;
        self.workspace_db(&uuid).await
    }

    /// 注册工作区数据库连接。
    pub async fn insert_workspace_db(&self, label: &str, uuid: Uuid, conn: DatabaseConnection) {
        self.workspace_dbs.write().await.insert(uuid, conn);
        self.label_to_uuid
            .write()
            .await
            .insert(label.to_owned(), uuid);
    }

    /// 移除工作区数据库连接。
    pub async fn remove_workspace_db(&self, label: &str) -> bool {
        let Some(uuid) = self.label_to_uuid.write().await.remove(label) else {
            return false;
        };
        self.workspace_dbs.write().await.remove(&uuid).is_some()
    }

    async fn resolve_uuid(&self, label: &str) -> Result<Uuid, AppError> {
        self.label_to_uuid
            .read()
            .await
            .get(label)
            .copied()
            .ok_or(AppError::NoWorkspaceDb)
    }
}

/// Guard that holds a read lock on workspace_dbs and provides access to a DB connection.
pub struct WorkspaceDbGuard<'a> {
    guard: tokio::sync::RwLockReadGuard<'a, HashMap<Uuid, DatabaseConnection>>,
    key: Uuid,
}

impl WorkspaceDbGuard<'_> {
    pub fn conn(&self) -> &DatabaseConnection {
        self.guard
            .get(&self.key)
            .expect("key was checked before constructing guard")
    }
}

// ── WorkspaceState ──

/// Per-workspace 工作区信息。主键为 UUID。
pub struct WorkspaceState {
    /// 主索引：workspace UUID → WorkspaceInfo
    workspaces: RwLock<HashMap<Uuid, WorkspaceInfo>>,
    /// 辅助索引：window label → workspace UUID
    bindings: RwLock<HashMap<String, Uuid>>,
}

impl WorkspaceState {
    pub fn new() -> Self {
        Self {
            workspaces: RwLock::new(HashMap::new()),
            bindings: RwLock::new(HashMap::new()),
        }
    }

    /// 绑定工作区到窗口（初始化和运行时共用）
    pub async fn bind(&self, label: &str, info: WorkspaceInfo) {
        let uuid = info.id;
        self.workspaces.write().await.insert(uuid, info);
        self.bindings.write().await.insert(label.to_owned(), uuid);
    }

    /// 解绑窗口
    pub async fn unbind_by_label(&self, label: &str) -> bool {
        let Some(uuid) = self.bindings.write().await.remove(label) else {
            return false;
        };
        self.workspaces.write().await.remove(&uuid).is_some()
    }

    /// 通过 label 获取工作区信息（Tauri 命令用）
    pub async fn get_by_label(&self, label: &str) -> Option<WorkspaceInfo> {
        let uuid = self.bindings.read().await.get(label).copied()?;
        self.workspaces.read().await.get(&uuid).cloned()
    }

    /// 获取所有已打开的工作区信息（同步层用）
    pub async fn list_all(&self) -> Vec<WorkspaceInfo> {
        self.workspaces.read().await.values().cloned().collect()
    }

    /// 查找已打开指定路径的窗口 label
    pub async fn find_label_by_path(&self, path: &str) -> Option<String> {
        let workspaces = self.workspaces.read().await;
        let bindings = self.bindings.read().await;
        bindings
            .iter()
            .find(|(_, uuid)| workspaces.get(uuid).is_some_and(|info| info.path == path))
            .map(|(label, _)| label.clone())
    }

    /// 获取指定窗口的工作区路径
    pub async fn workspace_path_for(&self, label: &str) -> Result<String, AppError> {
        self.get_by_label(label)
            .await
            .map(|ws| ws.path)
            .ok_or(AppError::NoWorkspaceOpen)
    }
}
