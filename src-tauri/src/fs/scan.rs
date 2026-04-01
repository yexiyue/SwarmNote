use std::collections::HashSet;
use std::path::Path;

use entity::workspace::documents;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::FileTreeNode;
use crate::error::{AppError, AppResult};

/// 递归扫描工作区目录并构建 `FileTreeNode` 树。
///
/// 规则：
/// - 仅包含 `.md` 文件和目录。
/// - 排除隐藏条目（以 `.` 开头的）。
/// - 跳过符号链接（使用 `fs::metadata` 而非 `symlink_metadata`）。
/// - 排序：目录在前、文件在后，不区分大小写的字母序。
pub fn scan_workspace_tree(workspace_path: &Path) -> Result<Vec<FileTreeNode>, AppError> {
    scan_dir(workspace_path, workspace_path)
}

fn scan_dir(root: &Path, dir: &Path) -> Result<Vec<FileTreeNode>, AppError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // 跳过隐藏条目
        if name_str.starts_with('.') {
            continue;
        }

        // 使用 metadata（会跟踪符号链接）—— 若失败则表示是损坏的
        // 符号链接或无法访问，跳过。
        let meta = match std::fs::metadata(entry.path()) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // 跳过符号链接：直接通过 read_link 检查是否为符号链接。
        if entry.path().read_link().is_ok() {
            continue;
        }

        let rel_path = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path().as_path())
            .to_string_lossy()
            .replace('\\', "/");

        if meta.is_dir() {
            let children = scan_dir(root, &entry.path())?;
            dirs.push(FileTreeNode {
                id: rel_path,
                name: name_str.into_owned(),
                children: Some(children),
            });
        } else if meta.is_file() && name_str.ends_with(".md") {
            let display_name = name_str.strip_suffix(".md").unwrap_or(&name_str);
            files.push(FileTreeNode {
                id: rel_path,
                name: display_name.to_string(),
                children: None,
            });
        }
    }

    // 排序：不区分大小写的字母序
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // 目录在前，文件在后
    dirs.extend(files);
    Ok(dirs)
}

// ── Reconcile scan results with DB ───────────────────────────

/// Collect all .md file rel_paths from a scan tree (flattened).
fn collect_md_paths(nodes: &[FileTreeNode], out: &mut HashSet<String>) {
    for node in nodes {
        if let Some(children) = &node.children {
            collect_md_paths(children, out);
        } else if node.id.ends_with(".md") {
            out.insert(node.id.clone());
        }
    }
}

/// Reconcile scanned .md files with the documents table.
///
/// - Files on disk but not in DB → INSERT new records (assign UUID).
/// - Files in DB but not on disk → **not deleted** (may be a file move;
///   cleanup is left to the tombstone GC mechanism).
pub async fn reconcile_with_db(
    db: &DatabaseConnection,
    workspace_id: Uuid,
    peer_id: &str,
    tree: &[FileTreeNode],
) -> AppResult<usize> {
    let mut disk_paths = HashSet::new();
    collect_md_paths(tree, &mut disk_paths);

    // Fetch existing rel_paths from DB
    let existing: HashSet<String> = documents::Entity::find()
        .filter(documents::Column::WorkspaceId.eq(workspace_id))
        .all(db)
        .await?
        .into_iter()
        .map(|m| m.rel_path)
        .collect();

    // INSERT missing files
    let missing: Vec<&String> = disk_paths.difference(&existing).collect();
    let count = missing.len();
    let now = chrono::Utc::now().timestamp();

    for rel_path in &missing {
        let title = crate::document::title_from_rel_path(rel_path);

        let model = documents::ActiveModel {
            id: Set(Uuid::now_v7()),
            workspace_id: Set(workspace_id),
            folder_id: Set(None),
            title: Set(title),
            rel_path: Set((*rel_path).clone()),
            lamport_clock: Set(0),
            created_by: Set(peer_id.to_owned()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        // INSERT OR IGNORE: UNIQUE(workspace_id, rel_path) ensures idempotency
        // if another call has already inserted a record for this path.
        let _ = documents::Entity::insert(model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    documents::Column::WorkspaceId,
                    documents::Column::RelPath,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await;
    }

    if count > 0 {
        tracing::info!("Reconcile: inserted {count} missing document records");
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn scan_empty_directory() {
        let dir = setup_temp_dir();
        let result = scan_workspace_tree(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn scan_mixed_content() {
        let dir = setup_temp_dir();
        fs::write(dir.path().join("hello.md"), "# Hello").unwrap();
        fs::write(dir.path().join("image.png"), "binary").unwrap();
        fs::create_dir(dir.path().join("notes")).unwrap();
        fs::write(dir.path().join("notes").join("sub.md"), "# Sub").unwrap();

        let result = scan_workspace_tree(dir.path()).unwrap();

        // 应包含：notes/ 文件夹、hello.md 文件（不含 image.png）
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "notes"); // folder first
        assert!(result[0].children.is_some());
        assert_eq!(result[0].children.as_ref().unwrap().len(), 1);
        assert_eq!(result[1].name, "hello"); // file second, no .md suffix
        assert!(result[1].children.is_none());
    }

    #[test]
    fn scan_hidden_files_filtered() {
        let dir = setup_temp_dir();
        fs::create_dir(dir.path().join(".swarmnote")).unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".hidden.md"), "secret").unwrap();
        fs::write(dir.path().join("visible.md"), "public").unwrap();

        let result = scan_workspace_tree(dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "visible");
    }

    #[test]
    fn scan_sorting_folders_first_case_insensitive() {
        let dir = setup_temp_dir();
        fs::write(dir.path().join("zebra.md"), "").unwrap();
        fs::write(dir.path().join("Alice.md"), "").unwrap();
        fs::create_dir(dir.path().join("Notes")).unwrap();
        fs::create_dir(dir.path().join("archive")).unwrap();

        let result = scan_workspace_tree(dir.path()).unwrap();

        // 文件夹在前：archive、Notes；然后文件：Alice、zebra
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].name, "archive");
        assert_eq!(result[1].name, "Notes");
        assert_eq!(result[2].name, "Alice");
        assert_eq!(result[3].name, "zebra");
    }

    #[test]
    fn scan_rel_path_uses_forward_slashes() {
        let dir = setup_temp_dir();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("note.md"), "").unwrap();

        let result = scan_workspace_tree(dir.path()).unwrap();
        let sub = &result[0];
        let note = &sub.children.as_ref().unwrap()[0];
        assert_eq!(note.id, "sub/note.md");
    }
}
