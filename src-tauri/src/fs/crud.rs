use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 校验 `rel_path` 未逃逸出工作区根目录。
///
/// 拒绝包含 `..` 组件或绝对路径的路径。
pub fn validate_rel_path(workspace: &Path, rel_path: &str) -> Result<PathBuf, AppError> {
    if rel_path.contains("..") {
        return Err(AppError::PathTraversal(rel_path.to_owned()));
    }

    let full = workspace.join(rel_path);
    let canonical_ws = workspace
        .canonicalize()
        .map_err(|e| AppError::InvalidPath(format!("{}: {e}", workspace.display())))?;

    // 对尚不存在的新路径，规范化其父目录
    let check_path = if full.exists() {
        full.canonicalize()
            .map_err(|e| AppError::InvalidPath(e.to_string()))?
    } else if let Some(parent) = full.parent() {
        let canonical_parent = parent
            .canonicalize()
            .map_err(|e| AppError::InvalidPath(e.to_string()))?;
        canonical_parent.join(full.file_name().unwrap_or_default())
    } else {
        return Err(AppError::PathTraversal(rel_path.to_owned()));
    };

    if !check_path.starts_with(&canonical_ws) {
        return Err(AppError::PathTraversal(rel_path.to_owned()));
    }

    Ok(full)
}

/// 通过追加 ` 1`、` 2` 等后缀来寻找不冲突的名称。
fn resolve_conflict(dir: &Path, base_name: &str, extension: &str) -> String {
    let mut counter = 1u32;
    loop {
        let candidate = if extension.is_empty() {
            format!("{base_name} {counter}")
        } else {
            format!("{base_name} {counter}.{extension}")
        };
        if !dir.join(&candidate).exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// 创建新的 `.md` 文件。返回所创建文件的相对路径。
///
/// 如果同名文件已存在，自动编号（如 `name 1.md`）。
pub fn create_file(workspace: &Path, parent_rel: &str, name: &str) -> Result<String, AppError> {
    let parent = if parent_rel.is_empty() {
        workspace.to_path_buf()
    } else {
        validate_rel_path(workspace, parent_rel)?
    };

    let filename = format!("{name}.md");
    let full_path = parent.join(&filename);

    let actual_name = if full_path.exists() {
        resolve_conflict(&parent, name, "md")
    } else {
        filename
    };

    std::fs::write(parent.join(&actual_name), "")?;

    let rel = parent
        .join(&actual_name)
        .strip_prefix(workspace)
        .unwrap_or(Path::new(&actual_name))
        .to_string_lossy()
        .replace('\\', "/");

    Ok(rel)
}

/// 创建新目录。返回所创建目录的相对路径。
pub fn create_dir(workspace: &Path, parent_rel: &str, name: &str) -> Result<String, AppError> {
    let parent = if parent_rel.is_empty() {
        workspace.to_path_buf()
    } else {
        validate_rel_path(workspace, parent_rel)?
    };

    let full_path = parent.join(name);

    let actual_name = if full_path.exists() {
        resolve_conflict(&parent, name, "")
    } else {
        name.to_owned()
    };

    std::fs::create_dir_all(parent.join(&actual_name))?;

    let rel = parent
        .join(&actual_name)
        .strip_prefix(workspace)
        .unwrap_or(Path::new(&actual_name))
        .to_string_lossy()
        .replace('\\', "/");

    Ok(rel)
}

/// 删除文件。幂等操作 —— 文件不存在时也视为成功。
///
/// 对 `.md` 文件，同步删除同名资源目录（best-effort）。
pub fn delete_file(workspace: &Path, rel_path: &str) -> Result<(), AppError> {
    let full = validate_rel_path(workspace, rel_path)?;
    match std::fs::remove_file(&full) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    // Best-effort: delete resource directory for .md files (e.g. "note.md" → "note.assets/")
    if full.extension().and_then(|e| e.to_str()) == Some("md") {
        let resource_dir = full.with_extension("assets");
        if resource_dir.is_dir() {
            let _ = std::fs::remove_dir_all(resource_dir);
        }
    }

    Ok(())
}

/// 递归删除目录及其所有内容。
pub fn delete_dir(workspace: &Path, rel_path: &str) -> Result<(), AppError> {
    let full = validate_rel_path(workspace, rel_path)?;
    match std::fs::remove_dir_all(full) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// 重命名文件或目录。返回新的相对路径。
pub fn rename(workspace: &Path, rel_path: &str, new_name: &str) -> Result<String, AppError> {
    let full = validate_rel_path(workspace, rel_path)?;
    let parent = full
        .parent()
        .ok_or_else(|| AppError::InvalidPath("no parent directory".into()))?;

    // 对文件，如果新名称不含扩展名则保留原扩展名
    let target_name = if full.is_file() && !new_name.contains('.') {
        let ext = full.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.is_empty() {
            new_name.to_owned()
        } else {
            format!("{new_name}.{ext}")
        }
    } else {
        new_name.to_owned()
    };

    let new_path = parent.join(&target_name);
    if new_path.exists() {
        return Err(AppError::NameConflict(target_name));
    }

    move_assets_sidecar(&full, &new_path, false)?;
    std::fs::rename(&full, &new_path)?;

    let rel = new_path
        .strip_prefix(workspace)
        .unwrap_or(&new_path)
        .to_string_lossy()
        .replace('\\', "/");

    Ok(rel)
}

/// 对 `.md` 文件的 `.assets/` 伴生目录进行重命名/移动。
///
/// - 如果源不是 `.md` 或没有 `.assets/` 伴生目录，直接返回（no-op）。
/// - `check_conflict=true`：若目标 `.assets/` 已存在则返回 `NameConflict`
///   （move 语义要求不覆盖）。
/// - `check_conflict=false`：已存在则跳过（rename 路径保持原有 best-effort 行为）。
fn move_assets_sidecar(from: &Path, to: &Path, check_conflict: bool) -> Result<(), AppError> {
    if !from.is_file() || from.extension().and_then(|e| e.to_str()) != Some("md") {
        return Ok(());
    }
    let old_assets = from.with_extension("assets");
    if !old_assets.is_dir() {
        return Ok(());
    }
    let new_assets = to.with_extension("assets");
    if new_assets.exists() {
        if check_conflict {
            return Err(AppError::NameConflict(
                new_assets.to_string_lossy().into_owned(),
            ));
        }
        return Ok(());
    }
    std::fs::rename(&old_assets, &new_assets)?;
    Ok(())
}

/// 结果信息：表示一次移动操作影响的元数据。
#[derive(Debug)]
pub struct MoveResult {
    /// 是否为目录（false 表示文件）。
    pub is_dir: bool,
}

/// 将文件或目录从 `from_rel` 移动到 `to_rel`（都是相对工作区根的路径）。
///
/// - `to_rel` 必须是**完整的目标路径**（包含文件名/目录名），而不是目标父目录。
/// - 如果源是 `.md` 文件且存在同名 `.assets/` 伴生目录，会原子地一起移动。
/// - 拒绝把目录移入自身的后代路径（防止递归嵌套）。
/// - 拒绝覆盖已有目标。
pub fn move_node(workspace: &Path, from_rel: &str, to_rel: &str) -> Result<MoveResult, AppError> {
    let from_full = validate_rel_path(workspace, from_rel)?;
    // `metadata()` fails with NotFound if the source does not exist; the `?`
    // converts that into an AppError::Io with a clear message, so we can skip
    // a separate `exists()` pre-check (TOCTOU) and get `is_dir` in one syscall.
    let is_dir = from_full.metadata()?.is_dir();

    if from_rel == to_rel {
        // no-op：调用方通常应在客户端过滤掉这种情况
        return Ok(MoveResult { is_dir });
    }

    // Target existence check IS load-bearing on Windows, where `std::fs::rename`
    // can silently overwrite an existing file instead of failing.
    let to_full = validate_rel_path(workspace, to_rel)?;
    if to_full.exists() {
        return Err(AppError::NameConflict(to_rel.to_owned()));
    }

    // 目录不允许被移入自身的后代路径（否则递归嵌套）。
    if is_dir {
        let from_with_sep = if from_rel.ends_with('/') {
            from_rel.to_owned()
        } else {
            format!("{from_rel}/")
        };
        if to_rel.starts_with(&from_with_sep) {
            return Err(AppError::InvalidPath(
                "cannot move a folder into its own descendant".into(),
            ));
        }
    }

    // 确保目标父目录存在（create_dir_all 是幂等的）
    if let Some(to_parent) = to_full.parent() {
        std::fs::create_dir_all(to_parent)?;
    }

    move_assets_sidecar(&from_full, &to_full, true)?;
    std::fs::rename(&from_full, &to_full)?;

    Ok(MoveResult { is_dir })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn create_file_basic() {
        let dir = tmp();
        let rel = create_file(dir.path(), "", "test").unwrap();
        assert_eq!(rel, "test.md");
        assert!(dir.path().join("test.md").exists());
    }

    #[test]
    fn create_file_in_subdir() {
        let dir = tmp();
        fs::create_dir(dir.path().join("notes")).unwrap();
        let rel = create_file(dir.path(), "notes", "diary").unwrap();
        assert_eq!(rel, "notes/diary.md");
    }

    #[test]
    fn create_file_conflict_auto_numbers() {
        let dir = tmp();
        fs::write(dir.path().join("note.md"), "").unwrap();
        let rel = create_file(dir.path(), "", "note").unwrap();
        assert_eq!(rel, "note 1.md");
        assert!(dir.path().join("note 1.md").exists());
    }

    #[test]
    fn create_dir_basic() {
        let dir = tmp();
        let rel = create_dir(dir.path(), "", "folder").unwrap();
        assert_eq!(rel, "folder");
        assert!(dir.path().join("folder").is_dir());
    }

    #[test]
    fn create_dir_conflict_auto_numbers() {
        let dir = tmp();
        fs::create_dir(dir.path().join("folder")).unwrap();
        let rel = create_dir(dir.path(), "", "folder").unwrap();
        assert_eq!(rel, "folder 1");
    }

    #[test]
    fn delete_file_idempotent() {
        let dir = tmp();
        // 对不存在的文件不应报错
        delete_file(dir.path(), "nonexistent.md").unwrap();
    }

    #[test]
    fn delete_file_existing() {
        let dir = tmp();
        fs::write(dir.path().join("del.md"), "content").unwrap();
        delete_file(dir.path(), "del.md").unwrap();
        assert!(!dir.path().join("del.md").exists());
    }

    #[test]
    fn delete_dir_recursive() {
        let dir = tmp();
        let sub = dir.path().join("folder");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("file.md"), "").unwrap();
        delete_dir(dir.path(), "folder").unwrap();
        assert!(!sub.exists());
    }

    #[test]
    fn rename_file_basic() {
        let dir = tmp();
        fs::write(dir.path().join("old.md"), "").unwrap();
        let rel = rename(dir.path(), "old.md", "new").unwrap();
        assert_eq!(rel, "new.md");
        assert!(dir.path().join("new.md").exists());
        assert!(!dir.path().join("old.md").exists());
    }

    #[test]
    fn rename_conflict_errors() {
        let dir = tmp();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::write(dir.path().join("b.md"), "").unwrap();
        let err = rename(dir.path(), "a.md", "b").unwrap_err();
        assert!(matches!(err, AppError::NameConflict(_)));
    }

    #[test]
    fn path_traversal_rejected() {
        let dir = tmp();
        let err = validate_rel_path(dir.path(), "../etc/passwd").unwrap_err();
        assert!(matches!(err, AppError::PathTraversal(_)));
    }

    #[test]
    fn rename_directory() {
        let dir = tmp();
        fs::create_dir(dir.path().join("old-folder")).unwrap();
        let rel = rename(dir.path(), "old-folder", "new-folder").unwrap();
        assert_eq!(rel, "new-folder");
        assert!(dir.path().join("new-folder").is_dir());
    }

    #[test]
    fn move_file_into_folder() {
        let dir = tmp();
        fs::create_dir(dir.path().join("archive")).unwrap();
        fs::write(dir.path().join("note.md"), "body").unwrap();
        let result = move_node(dir.path(), "note.md", "archive/note.md").unwrap();
        assert!(!result.is_dir);
        assert!(!dir.path().join("note.md").exists());
        assert!(dir.path().join("archive/note.md").exists());
        assert_eq!(
            fs::read_to_string(dir.path().join("archive/note.md")).unwrap(),
            "body"
        );
    }

    #[test]
    fn move_file_moves_assets_sidecar() {
        let dir = tmp();
        fs::create_dir(dir.path().join("archive")).unwrap();
        fs::write(dir.path().join("photo.md"), "body").unwrap();
        fs::create_dir(dir.path().join("photo.assets")).unwrap();
        fs::write(dir.path().join("photo.assets/img.png"), b"data").unwrap();

        move_node(dir.path(), "photo.md", "archive/photo.md").unwrap();
        assert!(dir.path().join("archive/photo.md").exists());
        assert!(dir.path().join("archive/photo.assets").is_dir());
        assert!(dir.path().join("archive/photo.assets/img.png").exists());
        assert!(!dir.path().join("photo.md").exists());
        assert!(!dir.path().join("photo.assets").exists());
    }

    #[test]
    fn move_folder_recursive() {
        let dir = tmp();
        fs::create_dir(dir.path().join("archive")).unwrap();
        fs::create_dir(dir.path().join("drafts")).unwrap();
        fs::write(dir.path().join("drafts/a.md"), "").unwrap();
        let result = move_node(dir.path(), "drafts", "archive/drafts").unwrap();
        assert!(result.is_dir);
        assert!(dir.path().join("archive/drafts").is_dir());
        assert!(dir.path().join("archive/drafts/a.md").exists());
        assert!(!dir.path().join("drafts").exists());
    }

    #[test]
    fn move_folder_into_own_descendant_rejected() {
        let dir = tmp();
        fs::create_dir_all(dir.path().join("drafts/sub")).unwrap();
        let err = move_node(dir.path(), "drafts", "drafts/sub/drafts").unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
    }

    #[test]
    fn move_rejects_existing_target() {
        let dir = tmp();
        fs::write(dir.path().join("a.md"), "").unwrap();
        fs::write(dir.path().join("b.md"), "").unwrap();
        let err = move_node(dir.path(), "a.md", "b.md").unwrap_err();
        assert!(matches!(err, AppError::NameConflict(_)));
    }

    #[test]
    fn move_noop_same_path() {
        let dir = tmp();
        fs::write(dir.path().join("x.md"), "").unwrap();
        let result = move_node(dir.path(), "x.md", "x.md").unwrap();
        assert!(!result.is_dir);
        assert!(dir.path().join("x.md").exists());
    }
}
