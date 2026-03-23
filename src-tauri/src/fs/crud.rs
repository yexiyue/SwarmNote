use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Validate that `rel_path` does not escape the workspace root.
///
/// Rejects paths containing `..` components or absolute paths.
pub fn validate_rel_path(workspace: &Path, rel_path: &str) -> Result<PathBuf, AppError> {
    if rel_path.contains("..") {
        return Err(AppError::PathTraversal(rel_path.to_owned()));
    }

    let full = workspace.join(rel_path);
    let canonical_ws = workspace
        .canonicalize()
        .map_err(|e| AppError::InvalidPath(format!("{}: {e}", workspace.display())))?;

    // For new paths that don't exist yet, canonicalize the parent
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

/// Find a non-conflicting name by appending ` 1`, ` 2`, etc.
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

/// Create a new `.md` file. Returns the rel_path of the created file.
///
/// If a file with the same name exists, auto-numbers (e.g. `name 1.md`).
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

/// Create a new directory. Returns the rel_path of the created directory.
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

/// Delete a file. Idempotent — succeeds if the file doesn't exist.
pub fn delete_file(workspace: &Path, rel_path: &str) -> Result<(), AppError> {
    let full = validate_rel_path(workspace, rel_path)?;
    match std::fs::remove_file(full) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Recursively delete a directory and all its contents.
pub fn delete_dir(workspace: &Path, rel_path: &str) -> Result<(), AppError> {
    let full = validate_rel_path(workspace, rel_path)?;
    match std::fs::remove_dir_all(full) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Rename a file or directory. Returns the new rel_path.
pub fn rename(workspace: &Path, rel_path: &str, new_name: &str) -> Result<String, AppError> {
    let full = validate_rel_path(workspace, rel_path)?;
    let parent = full
        .parent()
        .ok_or_else(|| AppError::InvalidPath("no parent directory".into()))?;

    // For files, preserve extension if new_name doesn't have one
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

    std::fs::rename(&full, &new_path)?;

    let rel = new_path
        .strip_prefix(workspace)
        .unwrap_or(&new_path)
        .to_string_lossy()
        .replace('\\', "/");

    Ok(rel)
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
        // Should not error on non-existent file
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
}
