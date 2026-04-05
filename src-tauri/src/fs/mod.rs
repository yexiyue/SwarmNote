//! 文件系统操作：工作区树扫描、文件 CRUD、文件变更监听。

pub mod commands;
pub mod crud;
pub mod scan;
pub mod watcher;

use serde::Serialize;

/// 工作区文件树中的节点。
#[derive(Debug, Clone, Serialize)]
pub struct FileTreeNode {
    /// 相对于工作区根目录的路径（用作唯一 ID）。
    pub id: String,
    /// 显示名称（文件不含 `.md` 扩展名）。
    pub name: String,
    /// 仅目录节点存在。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileTreeNode>>,
}
