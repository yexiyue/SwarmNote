//! 文档与文件夹的数据库 CRUD 操作。

pub mod commands;

/// Extract a human-readable title from a workspace-relative path.
///
/// `"notes/sub/my-note.md"` → `"my-note"`
pub fn title_from_rel_path(rel_path: &str) -> String {
    rel_path
        .rsplit('/')
        .next()
        .unwrap_or(rel_path)
        .trim_end_matches(".md")
        .to_owned()
}
