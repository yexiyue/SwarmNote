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

    /// 列出所有已打开的工作区 UUID（同步层用）。
    pub async fn list_workspace_uuids(&self) -> Vec<Uuid> {
        self.workspace_dbs.read().await.keys().copied().collect()
    }

    /// 注册工作区数据库连接（无窗口，仅按 UUID 索引）。
    ///
    /// 用于 sync-only 场景：sync 层只按 UUID 访问 DB，不需要 label 映射。
    pub async fn register_db(&self, uuid: Uuid, conn: DatabaseConnection) {
        self.workspace_dbs.write().await.insert(uuid, conn);
    }

    /// 注册工作区数据库连接（有窗口，建立 label → UUID 映射）。
    pub async fn insert_workspace_db(&self, label: &str, uuid: Uuid, conn: DatabaseConnection) {
        self.workspace_dbs.write().await.insert(uuid, conn);
        self.label_to_uuid
            .write()
            .await
            .insert(label.to_owned(), uuid);
    }

    /// 移除工作区数据库连接。
    /// 只在没有其他窗口引用同一 workspace UUID 时才真正移除 DB 连接。
    pub async fn remove_workspace_db(&self, label: &str) -> bool {
        let mut labels = self.label_to_uuid.write().await;
        let Some(uuid) = labels.remove(label) else {
            return false;
        };
        // Check if any other window still references the same workspace UUID
        let still_referenced = labels.values().any(|v| *v == uuid);
        drop(labels);
        if !still_referenced {
            self.workspace_dbs.write().await.remove(&uuid).is_some()
        } else {
            true
        }
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

    /// 注册工作区元信息（无窗口，仅按 UUID 索引）。
    ///
    /// 用于 sync-only 场景：sync 层通过 `get(&uuid)` 获取路径等信息。
    pub async fn register(&self, info: WorkspaceInfo) {
        self.workspaces.write().await.insert(info.id, info);
    }

    /// 绑定工作区到窗口（初始化和运行时共用）
    pub async fn bind(&self, label: &str, info: WorkspaceInfo) {
        let uuid = info.id;
        self.workspaces.write().await.insert(uuid, info);
        self.bindings.write().await.insert(label.to_owned(), uuid);
    }

    /// 解绑窗口。只在没有其他窗口绑定同一 UUID 时才移除 WorkspaceInfo。
    pub async fn unbind_by_label(&self, label: &str) -> bool {
        let mut bindings = self.bindings.write().await;
        let Some(uuid) = bindings.remove(label) else {
            return false;
        };
        let still_referenced = bindings.values().any(|v| *v == uuid);
        drop(bindings);
        if !still_referenced {
            self.workspaces.write().await.remove(&uuid).is_some()
        } else {
            true
        }
    }

    /// 通过 label 获取工作区信息（Tauri 命令用）
    pub async fn get_by_label(&self, label: &str) -> Option<WorkspaceInfo> {
        let uuid = self.bindings.read().await.get(label).copied()?;
        self.workspaces.read().await.get(&uuid).cloned()
    }

    /// 按 UUID 获取工作区信息（同步层用）
    pub async fn get(&self, uuid: &Uuid) -> Option<WorkspaceInfo> {
        self.workspaces.read().await.get(uuid).cloned()
    }

    /// 获取所有已打开的工作区信息（同步层用）
    pub async fn list_all(&self) -> Vec<WorkspaceInfo> {
        self.workspaces.read().await.values().cloned().collect()
    }

    /// 检查指定工作区是否有真实窗口绑定
    pub async fn is_bound(&self, uuid: &Uuid) -> bool {
        self.bindings.read().await.values().any(|v| v == uuid)
    }

    /// 返回有窗口绑定的工作区列表（不包含 sync-only 工作区）
    pub async fn list_bound(&self) -> Vec<WorkspaceInfo> {
        let workspaces = self.workspaces.read().await;
        let bindings = self.bindings.read().await;
        let bound_uuids: std::collections::HashSet<&Uuid> = bindings.values().collect();
        workspaces
            .iter()
            .filter(|(uuid, _)| bound_uuids.contains(uuid))
            .map(|(_, info)| info.clone())
            .collect()
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
