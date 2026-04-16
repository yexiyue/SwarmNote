use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use tauri::{AppHandle, Emitter};

use crate::yjs::doc_state;

type FsNotifyWatcher = notify::RecommendedWatcher;

/// Tauri 托管状态，持有 per-window 的文件系统监听器。
pub struct FsWatcherState(pub Mutex<HashMap<String, Debouncer<FsNotifyWatcher>>>);

impl FsWatcherState {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// 判断该路径是否应触发文件树变更事件，返回 `true` 表示需要触发。
fn is_relevant_change(path: &Path, workspace: &Path) -> bool {
    let rel = match path.strip_prefix(workspace) {
        Ok(r) => r,
        Err(_) => return false,
    };

    for component in rel.components() {
        let s = component.as_os_str().to_string_lossy();
        // Hidden (`.swarmnote/`, `.git/`, `.DS_Store`) + content-addressed
        // asset sidecars (`note.assets/`) never contribute tree changes.
        if s.starts_with('.') || s.ends_with(".assets") {
            return false;
        }
    }

    // 目录变更始终相关（创建/删除文件夹）
    if path.is_dir() || !path.exists() {
        // 路径已不存在时无法判断 is_dir —— 除非扩展名另有说明，否则视为相关
        let ext = path.extension().and_then(|e| e.to_str());
        return ext.is_none() || ext == Some("md");
    }

    // 仅关注 .md 文件变更
    path.extension().and_then(|e| e.to_str()) == Some("md")
}

/// Convert a path to a forward-slash workspace-relative string.
fn to_rel_path(path: &Path, workspace: &Path) -> Option<String> {
    let rel = path.strip_prefix(workspace).ok()?;
    // Normalize to forward slashes (Windows produces backslashes)
    Some(rel.to_string_lossy().replace('\\', "/"))
}

/// 开始监听工作区目录的文件变更，事件定向发送给指定窗口。
///
/// 以 100ms 防抖后向目标窗口发送 `fs:tree-changed` 事件。
/// 对 .md 文件变更还会触发 Y.Doc 外部重载检查。
pub fn start_watching(
    app_handle: &AppHandle,
    label: &str,
    workspace_path: &Path,
    state: &FsWatcherState,
) -> Result<(), crate::error::AppError> {
    // 先停止该窗口已有的监听器
    stop_watching(label, state);

    let app = app_handle.clone();
    let ws_path = workspace_path.to_path_buf();
    let target_label = label.to_owned();

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

            let relevant: Vec<&Path> = events
                .iter()
                .filter(|e| e.kind == DebouncedEventKind::Any)
                .map(|e| e.path.as_path())
                .filter(|p| is_relevant_change(p, &ws_path))
                .collect();

            if relevant.is_empty() {
                return;
            }

            // Emit tree-changed event (existing behaviour)
            if let Err(e) = app.emit_to(&target_label, "fs:tree-changed", ()) {
                log::warn!("Failed to emit fs:tree-changed to {target_label}: {e}");
            }

            // Trigger Y.Doc reload check for each changed .md file
            let md_paths: Vec<String> = relevant
                .iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
                .filter_map(|p| to_rel_path(p, &ws_path))
                .collect();

            if !md_paths.is_empty() {
                let app = app.clone();
                let label = target_label.clone();
                tauri::async_runtime::spawn(async move {
                    for rel_path in md_paths {
                        if let Err(e) = doc_state::handle_file_change(&app, &label, &rel_path).await
                        {
                            tracing::warn!("handle_file_change failed for {rel_path}: {e}");
                        }
                    }
                });
            }
        },
    )
    .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?;

    // 递归监听工作区目录
    let watcher_path = PathBuf::from(workspace_path);
    {
        let mut guard = state.0.lock().expect("FsWatcherState mutex poisoned");
        guard.insert(label.to_owned(), debouncer);
        if let Some(d) = guard.get_mut(label) {
            d.watcher()
                .watch(&watcher_path, notify::RecursiveMode::Recursive)
                .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?;
        }
    }

    log::info!(
        "Started fs watcher for window '{}': {}",
        label,
        workspace_path.display()
    );

    Ok(())
}

/// 停止指定窗口的文件系统监听器（如果存在）。
pub fn stop_watching(label: &str, state: &FsWatcherState) {
    let mut guard = state.0.lock().expect("FsWatcherState mutex poisoned");
    if guard.remove(label).is_some() {
        log::info!("Stopped fs watcher for window '{label}'");
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
        let path = Path::new("/workspace/note.md");
        assert!(is_relevant_change(path, &ws()));
    }

    #[test]
    fn to_rel_path_normalizes_slashes() {
        let ws = PathBuf::from("/workspace");
        let path = Path::new("/workspace/notes/hello.md");
        assert_eq!(to_rel_path(path, &ws), Some("notes/hello.md".to_owned()));
    }

    #[test]
    fn to_rel_path_outside_workspace() {
        let ws = PathBuf::from("/workspace");
        let path = Path::new("/other/hello.md");
        assert_eq!(to_rel_path(path, &ws), None);
    }
}
