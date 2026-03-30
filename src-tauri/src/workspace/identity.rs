use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

const IDENTITY_FILE: &str = "workspace.json";

/// Persistent workspace identity stored in `.swarmnote/workspace.json`.
///
/// This is the **source of truth** for a workspace's UUID. The `workspaces`
/// table in the DB is a runtime mirror that is kept in sync on every open.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceIdentity {
    pub uuid: Uuid,
    pub name: String,
    pub created_at: String,
}

/// Read workspace identity from `.swarmnote/workspace.json`.
/// Returns `None` if the file does not exist.
pub fn read_identity(workspace_path: &Path) -> AppResult<Option<WorkspaceIdentity>> {
    let path = workspace_path.join(".swarmnote").join(IDENTITY_FILE);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let identity: WorkspaceIdentity =
                serde_json::from_str(&content).map_err(|e| AppError::Config(e.to_string()))?;
            Ok(Some(identity))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::Io(e)),
    }
}

/// Write workspace identity to `.swarmnote/workspace.json`.
/// Creates the `.swarmnote/` directory if needed.
pub fn write_identity(workspace_path: &Path, identity: &WorkspaceIdentity) -> AppResult<()> {
    let dir = workspace_path.join(".swarmnote");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(IDENTITY_FILE);
    let content =
        serde_json::to_string_pretty(identity).map_err(|e| AppError::Config(e.to_string()))?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Ensure a workspace identity file exists. Returns the UUID to use.
///
/// Priority:
/// 1. Read from workspace.json (if exists)
/// 2. Fall back to the UUID from the DB workspaces table (for existing workspaces upgrading)
/// 3. Generate a new UUID (for brand new workspaces)
pub fn ensure_identity(
    workspace_path: &Path,
    db_uuid: Option<Uuid>,
    workspace_name: &str,
) -> AppResult<Uuid> {
    // Try reading existing identity file
    if let Some(identity) = read_identity(workspace_path)? {
        return Ok(identity.uuid);
    }

    // No identity file — determine UUID
    let uuid = db_uuid.unwrap_or_else(Uuid::now_v7);

    let identity = WorkspaceIdentity {
        uuid,
        name: workspace_name.to_owned(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    write_identity(workspace_path, &identity)?;

    tracing::info!(
        "Created workspace identity: {} → {}",
        workspace_path.display(),
        uuid
    );

    Ok(uuid)
}
