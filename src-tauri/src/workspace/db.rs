//! Desktop DB shim. Connection + migration helpers live in
//! `swarmnote_core::workspace::db`; this module keeps the
//! `~/.swarmnote/devices.db` path convention that only applies on desktop.

use std::path::{Path, PathBuf};

use sea_orm::DatabaseConnection;

use crate::error::AppError;

pub fn swarmnote_global_dir() -> Result<PathBuf, AppError> {
    crate::config::swarmnote_global_dir()
}

pub async fn init_devices_db() -> Result<DatabaseConnection, AppError> {
    Ok(swarmnote_core::workspace::db::init_devices_db(&swarmnote_global_dir()?).await?)
}

pub async fn init_workspace_db(workspace_path: &Path) -> Result<DatabaseConnection, AppError> {
    Ok(swarmnote_core::workspace::db::init_workspace_db(workspace_path).await?)
}
