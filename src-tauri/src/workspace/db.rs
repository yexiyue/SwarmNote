use std::path::{Path, PathBuf};

use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};

use crate::error::AppError;
use migration::{DevicesMigrator, WorkspaceMigrator};

pub fn swarmnote_global_dir() -> Result<PathBuf, AppError> {
    let home = directories::BaseDirs::new().ok_or(AppError::NoAppDataDir)?;
    Ok(home.home_dir().join(".swarmnote"))
}

pub async fn connect_sqlite(path: &Path) -> Result<DatabaseConnection, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = format!("sqlite:{}?mode=rwc", path.display());
    Ok(Database::connect(&url).await?)
}

pub async fn init_devices_db() -> Result<DatabaseConnection, AppError> {
    let db_path = swarmnote_global_dir()?.join("devices.db");
    let db = connect_sqlite(&db_path).await?;
    DevicesMigrator::up(&db, None).await?;
    Ok(db)
}

pub async fn init_workspace_db(workspace_path: &Path) -> Result<DatabaseConnection, AppError> {
    let db_path = workspace_path.join(".swarmnote").join("workspace.db");
    let db = connect_sqlite(&db_path).await?;
    WorkspaceMigrator::up(&db, None).await?;
    Ok(db)
}
