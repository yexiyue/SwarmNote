//! Desktop-side config shim. The `GlobalConfig` / `RecentWorkspace` /
//! `GlobalConfigState` types now live in `swarmnote_core::config`; this
//! module only keeps the desktop-specific path convention
//! (`~/.swarmnote/config.json`) and no-arg convenience wrappers used by
//! legacy call sites. Removed entirely in PR #3.

use std::path::PathBuf;

use crate::error::{AppError, AppResult};

// Canonical types — single nominal `GlobalConfig` across the whole app.
pub use swarmnote_core::config::{
    save_config as core_save_config, GlobalConfig, GlobalConfigState, RecentWorkspace,
};

/// Desktop config directory: `~/.swarmnote/`.
pub fn swarmnote_global_dir() -> AppResult<PathBuf> {
    let home = directories::BaseDirs::new().ok_or(AppError::NoAppDataDir)?;
    Ok(home.home_dir().join(".swarmnote"))
}

/// Load (or create default) the desktop config at `~/.swarmnote/config.json`.
pub fn load_or_create_config() -> AppResult<GlobalConfig> {
    Ok(swarmnote_core::config::load_or_create_config(
        &swarmnote_global_dir()?,
    )?)
}

/// Persist config to the desktop-default path. Wrapper around the core
/// path-explicit `save_config` so legacy callers that didn't track a path
/// keep working.
pub fn save_config(config: &GlobalConfig) -> AppResult<()> {
    let path = swarmnote_global_dir()?.join("config.json");
    Ok(core_save_config(&path, config)?)
}

/// Update `last_workspace_path` + maintain `recent_workspaces`, then persist.
pub fn update_last_workspace(config: &mut GlobalConfig, path: &str, name: &str) -> AppResult<()> {
    apply_update(config, path, name, None);
    save_config(config)
}

/// Variant that also records a workspace UUID (used when creating a
/// workspace from a sync peer).
pub fn update_last_workspace_with_uuid(
    config: &mut GlobalConfig,
    path: &str,
    name: &str,
    uuid: &str,
) -> AppResult<()> {
    apply_update(config, path, name, Some(uuid));
    save_config(config)
}

fn apply_update(config: &mut GlobalConfig, path: &str, name: &str, uuid: Option<&str>) {
    const MAX_RECENT: usize = 10;
    let now = chrono::Utc::now().to_rfc3339();

    config.last_workspace_path = Some(path.to_owned());
    config.recent_workspaces.retain(|w| w.path != path);
    config.recent_workspaces.insert(
        0,
        RecentWorkspace {
            path: path.to_owned(),
            name: name.to_owned(),
            last_opened_at: now,
            uuid: uuid.map(|s| s.to_owned()),
        },
    );
    config.recent_workspaces.truncate(MAX_RECENT);
}
