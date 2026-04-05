//! 全局应用配置：设备身份、工作区历史等持久化设置。

use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock as TokioRwLock;

use crate::error::{AppError, AppResult};

const MAX_RECENT_WORKSPACES: usize = 10;

/// 全局配置状态，存储在 Tauri State 中用于运行时读写。
pub struct GlobalConfigState(TokioRwLock<GlobalConfig>);

impl GlobalConfigState {
    pub fn new(config: GlobalConfig) -> Self {
        Self(TokioRwLock::new(config))
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, GlobalConfig> {
        self.0.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, GlobalConfig> {
        self.0.write().await
    }
}

/// 持久化在 `~/.swarmnote/config.json` 的全局应用配置。
///
/// 涵盖设备身份和工作区历史。使用 `#[serde(default)]`
/// 以兼容缺少新字段的旧配置文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub device_name: String,
    pub created_at: String,
    #[serde(default)]
    pub last_workspace_path: Option<String>,
    #[serde(default)]
    pub recent_workspaces: Vec<RecentWorkspace>,
}

/// 最近工作区列表中的条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentWorkspace {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,
    /// 工作区 UUID，用于前端匹配同步状态。旧数据可能没有此字段。
    #[serde(default)]
    pub uuid: Option<String>,
}

/// 返回配置目录路径（~/.swarmnote/）。
fn config_dir() -> AppResult<PathBuf> {
    let home = directories::BaseDirs::new().ok_or(AppError::NoAppDataDir)?;
    Ok(home.home_dir().join(".swarmnote"))
}

fn config_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

/// 加载现有配置，若不存在则用默认值创建新配置。
pub fn load_or_create_config() -> AppResult<GlobalConfig> {
    let path = config_path()?;

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let config: GlobalConfig = serde_json::from_str(&content)
            .map_err(|e| AppError::Config(format!("invalid config JSON: {e}")))?;
        info!("Loaded global config from {}", path.display());
        return Ok(config);
    }

    let default_name = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "SwarmNote Device".to_string());

    let config = GlobalConfig {
        device_name: default_name,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_workspace_path: None,
        recent_workspaces: Vec::new(),
    };

    save_config(&config)?;
    info!("Created default global config at {}", path.display());
    Ok(config)
}

/// 将全局配置持久化到磁盘。
pub fn save_config(config: &GlobalConfig) -> AppResult<()> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Config(format!("serialize config: {e}")))?;

    fs::write(&path, json)?;
    Ok(())
}

/// 更新 `last_workspace_path` 并维护 `recent_workspaces` 列表。
///
/// 按路径去重，按 `last_opened_at` 降序排列，最多保留 10 条。
pub fn update_last_workspace(config: &mut GlobalConfig, path: &str, name: &str) -> AppResult<()> {
    apply_workspace_update(config, path, name, None);
    save_config(config)
}

/// 带 UUID 的工作区更新（同步创建时使用）。
pub fn update_last_workspace_with_uuid(
    config: &mut GlobalConfig,
    path: &str,
    name: &str,
    uuid: &str,
) -> AppResult<()> {
    apply_workspace_update(config, path, name, Some(uuid));
    save_config(config)
}

/// 内存中的工作区更新逻辑（无磁盘 I/O），可独立测试。
fn apply_workspace_update(config: &mut GlobalConfig, path: &str, name: &str, uuid: Option<&str>) {
    let now = chrono::Utc::now().to_rfc3339();

    config.last_workspace_path = Some(path.to_owned());

    // 移除相同路径的已有条目（去重）
    config.recent_workspaces.retain(|w| w.path != path);

    // 插入到列表头部（最近的在前）
    config.recent_workspaces.insert(
        0,
        RecentWorkspace {
            path: path.to_owned(),
            name: name.to_owned(),
            last_opened_at: now,
            uuid: uuid.map(|s| s.to_owned()),
        },
    );

    // 限制为 MAX_RECENT_WORKSPACES 条
    config.recent_workspaces.truncate(MAX_RECENT_WORKSPACES);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_config() -> GlobalConfig {
        GlobalConfig {
            device_name: "Test".to_owned(),
            created_at: "2026-01-01T00:00:00Z".to_owned(),
            last_workspace_path: None,
            recent_workspaces: Vec::new(),
        }
    }

    #[test]
    fn deserialize_old_format_without_workspace_fields() {
        let json = r#"{"device_name":"PC","created_at":"2026-01-01T00:00:00Z"}"#;
        let config: GlobalConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.device_name, "PC");
        assert!(config.last_workspace_path.is_none());
        assert!(config.recent_workspaces.is_empty());
    }

    #[test]
    fn deserialize_full_format() {
        let json = r#"{
            "device_name": "PC",
            "created_at": "2026-01-01T00:00:00Z",
            "last_workspace_path": "/tmp/notes",
            "recent_workspaces": [
                {"path": "/tmp/notes", "name": "notes", "last_opened_at": "2026-01-02T00:00:00Z"}
            ]
        }"#;
        let config: GlobalConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.last_workspace_path.as_deref(), Some("/tmp/notes"));
        assert_eq!(config.recent_workspaces.len(), 1);
        assert_eq!(config.recent_workspaces[0].name, "notes");
    }

    #[test]
    fn serialize_roundtrip() {
        let mut config = empty_config();
        config.last_workspace_path = Some("/tmp/ws".to_owned());
        config.recent_workspaces.push(RecentWorkspace {
            path: "/tmp/ws".to_owned(),
            name: "ws".to_owned(),
            last_opened_at: "2026-01-01T00:00:00Z".to_owned(),
            uuid: None,
        });

        let json = serde_json::to_string(&config).unwrap();
        let restored: GlobalConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.last_workspace_path, config.last_workspace_path);
        assert_eq!(restored.recent_workspaces.len(), 1);
    }

    #[test]
    fn update_sets_last_workspace_path() {
        let mut config = empty_config();
        apply_workspace_update(&mut config, "/tmp/notes", "notes", None);

        assert_eq!(config.last_workspace_path.as_deref(), Some("/tmp/notes"));
    }

    #[test]
    fn update_adds_to_recent_list() {
        let mut config = empty_config();
        apply_workspace_update(&mut config, "/tmp/a", "a", None);
        apply_workspace_update(&mut config, "/tmp/b", "b", None);

        assert_eq!(config.recent_workspaces.len(), 2);
        assert_eq!(config.recent_workspaces[0].path, "/tmp/b");
        assert_eq!(config.recent_workspaces[1].path, "/tmp/a");
    }

    #[test]
    fn update_deduplicates_by_path() {
        let mut config = empty_config();
        apply_workspace_update(&mut config, "/tmp/a", "a", None);
        apply_workspace_update(&mut config, "/tmp/b", "b", None);
        apply_workspace_update(&mut config, "/tmp/a", "a-renamed", None);

        assert_eq!(config.recent_workspaces.len(), 2);
        assert_eq!(config.recent_workspaces[0].path, "/tmp/a");
        assert_eq!(config.recent_workspaces[0].name, "a-renamed");
        assert_eq!(config.recent_workspaces[1].path, "/tmp/b");
    }

    #[test]
    fn update_caps_at_10_entries() {
        let mut config = empty_config();
        for i in 0..15 {
            apply_workspace_update(&mut config, &format!("/tmp/{i}"), &format!("{i}"), None);
        }

        assert_eq!(config.recent_workspaces.len(), MAX_RECENT_WORKSPACES);
        assert_eq!(config.recent_workspaces[0].path, "/tmp/14");
        assert_eq!(config.recent_workspaces[9].path, "/tmp/5");
    }
}
