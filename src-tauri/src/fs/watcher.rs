use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use tauri::{AppHandle, Emitter};

type FsNotifyWatcher = notify::RecommendedWatcher;

/// Tauri 托管状态，持有活跃的文件系统监听器。
pub struct FsWatcherState(pub Mutex<Option<Debouncer<FsNotifyWatcher>>>);

impl FsWatcherState {
    pub fn new() -> Self {
        Self(Mutex::new(None))
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
        if s.starts_with('.') {
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

/// 开始监听工作区目录的文件变更。
///
/// 以 100ms 防抖后向前端发送 `fs:tree-changed` 事件。
pub fn start_watching(
    app_handle: &AppHandle,
    workspace_path: &Path,
    state: &FsWatcherState,
) -> Result<(), crate::error::AppError> {
    // 先停止已有的监听器
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

    // 递归监听工作区目录
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

/// 停止活跃的文件系统监听器（如果存在）。
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
        // 此测试检查扩展名逻辑（文件可能不存在于磁盘上）
        let path = Path::new("/workspace/note.md");
        // 路径不存在时，is_relevant_change 通过扩展名判断
        assert!(is_relevant_change(path, &ws()));
    }
}
