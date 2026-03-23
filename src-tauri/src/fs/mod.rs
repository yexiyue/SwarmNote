pub mod commands;
pub mod crud;
pub mod scan;
pub mod watcher;

use serde::Serialize;

/// A node in the workspace file tree.
#[derive(Debug, Clone, Serialize)]
pub struct FileTreeNode {
    /// Relative path from workspace root (used as unique ID).
    pub id: String,
    /// Display name (filename without `.md` extension for files).
    pub name: String,
    /// Present only for directories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileTreeNode>>,
}
