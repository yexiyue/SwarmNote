use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use tauri::{AppHandle, Emitter};

type FsNotifyWatcher = notify::RecommendedWatcher;

/// Managed Tauri state holding the active file-system watcher.
pub struct FsWatcherState(pub Mutex<Option<Debouncer<FsNotifyWatcher>>>);

impl FsWatcherState {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

/// Returns `true` if the path should trigger a tree-changed event.
fn is_relevant_change(path: &Path, workspace: &Path) -> bool {
    let rel = match path.strip_prefix(workspace) {
        Ok(r) => r,
        Err(_) => return false,
    };

    for component in rel.components() {
        let s = component.as_os_str().to_string_lossy();
        if s.starts_with('.') {
            return false;
        }
    }

    // Directory changes are always relevant (create/delete folder)
    if path.is_dir() || !path.exists() {
        // If the path no longer exists, we can't check is_dir — assume relevant
        // unless the extension tells us otherwise.
        let ext = path.extension().and_then(|e| e.to_str());
        return ext.is_none() || ext == Some("md");
    }

    // Only .md file changes
    path.extension().and_then(|e| e.to_str()) == Some("md")
}

/// Start watching a workspace directory for file changes.
///
/// Debounces events by 100ms and emits `fs:tree-changed` to the frontend.
pub fn start_watching(
    app_handle: &AppHandle,
    workspace_path: &Path,
    state: &FsWatcherState,
) -> Result<(), crate::error::AppError> {
    // Stop any existing watcher first
    stop_watching(state);

    let app = app_handle.clone();
    let ws_path = workspace_path.to_path_buf();

    let debouncer = new_debouncer(
        Duration::from_millis(100),
        move |events: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            let events = match events {
                Ok(evts) => evts,
                Err(e) => {
                    log::warn!("fs watcher error: {e}");
                    return;
                }
            };

            let any_relevant = events
                .iter()
                .filter(|e| e.kind == DebouncedEventKind::Any)
                .any(|e| is_relevant_change(&e.path, &ws_path));

            if any_relevant {
                if let Err(e) = app.emit("fs:tree-changed", ()) {
                    log::warn!("Failed to emit fs:tree-changed: {e}");
                }
            }
        },
    )
    .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?;

    // Watch the workspace directory recursively
    let watcher_path = PathBuf::from(workspace_path);
    {
        let mut guard = state.0.lock().unwrap();
        *guard = Some(debouncer);
        if let Some(ref mut d) = *guard {
            d.watcher()
                .watch(&watcher_path, notify::RecursiveMode::Recursive)
                .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?;
        }
    }

    log::info!("Started fs watcher for: {}", workspace_path.display());

    Ok(())
}

/// Stop the active file-system watcher (if any).
pub fn stop_watching(state: &FsWatcherState) {
    let mut guard = state.0.lock().unwrap();
    if guard.take().is_some() {
        log::info!("Stopped fs watcher");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn ws() -> PathBuf {
        PathBuf::from("/workspace")
    }

    #[test]
    fn hidden_dirs_filtered() {
        assert!(!is_relevant_change(
            Path::new("/workspace/.swarmnote/db.sqlite"),
            &ws()
        ));
        assert!(!is_relevant_change(
            Path::new("/workspace/.git/HEAD"),
            &ws()
        ));
    }

    #[test]
    fn non_md_files_filtered() {
        assert!(!is_relevant_change(
            Path::new("/workspace/image.png"),
            &ws()
        ));
    }

    #[test]
    fn md_files_relevant() {
        // This test checks the extension logic (file may not exist on disk)
        let path = Path::new("/workspace/note.md");
        // When path doesn't exist, is_relevant_change checks extension
        assert!(is_relevant_change(path, &ws()));
    }
}
