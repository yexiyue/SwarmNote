use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_RECENT_WORKSPACES: usize = 10;

/// Global application configuration persisted at `~/.swarmnote/config.json`.
///
/// Covers device identity and workspace history. Uses `#[serde(default)]`
/// for backward compatibility with older config files that lack new fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub device_name: String,
    pub created_at: String,
    #[serde(default)]
    pub last_workspace_path: Option<String>,
    #[serde(default)]
    pub recent_workspaces: Vec<RecentWorkspace>,
}

/// Entry in the recent workspaces list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentWorkspace {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,
}

/// Return the config directory path (~/.swarmnote/).
fn config_dir() -> Result<PathBuf, crate::identity::IdentityError> {
    let home = directories::BaseDirs::new().ok_or_else(|| {
        crate::identity::IdentityError::Config("cannot determine home directory".into())
    })?;
    Ok(home.home_dir().join(".swarmnote"))
}

fn config_path() -> Result<PathBuf, crate::identity::IdentityError> {
    Ok(config_dir()?.join("config.json"))
}

/// Load existing config or create a new one with defaults.
pub fn load_or_create_config() -> Result<GlobalConfig, crate::identity::IdentityError> {
    let path = config_path()?;

    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;
        let config: GlobalConfig = serde_json::from_str(&content)
            .map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;
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

/// Persist global config to disk.
pub fn save_config(config: &GlobalConfig) -> Result<(), crate::identity::IdentityError> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;

    fs::write(&path, json).map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;

    Ok(())
}

/// Update `last_workspace_path` and maintain the `recent_workspaces` list.
///
/// Deduplicates by path, sorts by `last_opened_at` descending, and caps at 10 entries.
pub fn update_last_workspace(
    config: &mut GlobalConfig,
    path: &str,
    name: &str,
) -> Result<(), crate::identity::IdentityError> {
    apply_workspace_update(config, path, name);
    save_config(config)
}

/// In-memory workspace update logic (no disk I/O). Testable independently.
fn apply_workspace_update(config: &mut GlobalConfig, path: &str, name: &str) {
    let now = chrono::Utc::now().to_rfc3339();

    config.last_workspace_path = Some(path.to_owned());

    // Remove existing entry with same path (dedup)
    config.recent_workspaces.retain(|w| w.path != path);

    // Insert at front (most recent first)
    config.recent_workspaces.insert(
        0,
        RecentWorkspace {
            path: path.to_owned(),
            name: name.to_owned(),
            last_opened_at: now,
        },
    );

    // Cap at MAX_RECENT_WORKSPACES
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
        });

        let json = serde_json::to_string(&config).unwrap();
        let restored: GlobalConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.last_workspace_path, config.last_workspace_path);
        assert_eq!(restored.recent_workspaces.len(), 1);
    }

    #[test]
    fn update_sets_last_workspace_path() {
        let mut config = empty_config();
        apply_workspace_update(&mut config, "/tmp/notes", "notes");

        assert_eq!(config.last_workspace_path.as_deref(), Some("/tmp/notes"));
    }

    #[test]
    fn update_adds_to_recent_list() {
        let mut config = empty_config();
        apply_workspace_update(&mut config, "/tmp/a", "a");
        apply_workspace_update(&mut config, "/tmp/b", "b");

        assert_eq!(config.recent_workspaces.len(), 2);
        // Most recent first
        assert_eq!(config.recent_workspaces[0].path, "/tmp/b");
        assert_eq!(config.recent_workspaces[1].path, "/tmp/a");
    }

    #[test]
    fn update_deduplicates_by_path() {
        let mut config = empty_config();
        apply_workspace_update(&mut config, "/tmp/a", "a");
        apply_workspace_update(&mut config, "/tmp/b", "b");
        apply_workspace_update(&mut config, "/tmp/a", "a-renamed");

        assert_eq!(config.recent_workspaces.len(), 2);
        assert_eq!(config.recent_workspaces[0].path, "/tmp/a");
        assert_eq!(config.recent_workspaces[0].name, "a-renamed");
        assert_eq!(config.recent_workspaces[1].path, "/tmp/b");
    }

    #[test]
    fn update_caps_at_10_entries() {
        let mut config = empty_config();
        for i in 0..15 {
            apply_workspace_update(&mut config, &format!("/tmp/{i}"), &format!("{i}"));
        }

        assert_eq!(config.recent_workspaces.len(), MAX_RECENT_WORKSPACES);
        // Most recent (14) should be first
        assert_eq!(config.recent_workspaces[0].path, "/tmp/14");
        // Oldest surviving should be 5 (0-4 were truncated)
        assert_eq!(config.recent_workspaces[9].path, "/tmp/5");
    }
}
