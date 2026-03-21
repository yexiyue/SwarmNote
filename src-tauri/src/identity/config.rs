use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persisted device configuration (non-sensitive data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub device_name: String,
    pub created_at: String,
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
pub fn load_or_create_config() -> Result<DeviceConfig, crate::identity::IdentityError> {
    let path = config_path()?;

    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;
        let config: DeviceConfig = serde_json::from_str(&content)
            .map_err(|e| crate::identity::IdentityError::Config(e.to_string()))?;
        info!("Loaded device config from {}", path.display());
        return Ok(config);
    }

    let default_name = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "SwarmNote Device".to_string());

    let config = DeviceConfig {
        device_name: default_name,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    save_config(&config)?;
    info!("Created default device config at {}", path.display());
    Ok(config)
}

/// Persist device config to disk.
pub fn save_config(config: &DeviceConfig) -> Result<(), crate::identity::IdentityError> {
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
