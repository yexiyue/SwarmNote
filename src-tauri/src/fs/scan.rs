use std::path::Path;

use crate::error::AppError;

use super::FileTreeNode;

/// Recursively scan a workspace directory and build a `FileTreeNode` tree.
///
/// Rules:
/// - Only `.md` files and directories are included.
/// - Hidden entries (starting with `.`) are excluded.
/// - Symlinks are skipped (uses `fs::metadata`, not `symlink_metadata`).
/// - Sorted: directories first, then files, case-insensitive alphabetical.
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

        // Skip hidden entries
        if name_str.starts_with('.') {
            continue;
        }

        // Use metadata (follows symlinks) — if it fails, the entry is a broken
        // symlink or inaccessible; skip it.
        let meta = match std::fs::metadata(entry.path()) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Skip symlinks: if symlink_metadata differs from metadata in file_type,
        // the entry is a symlink. But simpler: just check symlink_metadata directly.
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

    // Sort: case-insensitive alphabetical
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Directories first, then files
    dirs.extend(files);
    Ok(dirs)
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

        // Should have: notes/ folder, hello.md file (no image.png)
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

        // Folders first: archive, Notes; then files: Alice, zebra
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
