//! Workspace-level types and helpers.
//!
//! `WorkspaceCore` (PR #2) will live alongside this module. PR #1 populates
//! only [`WorkspaceInfo`] and the low-level DB bootstrap helpers in [`db`].

pub mod db;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Runtime + DB record of an open workspace. Returned to the frontend by
/// `get_workspace_info`-style commands; held by `WorkspaceCore` (PR #2) as
/// its own metadata snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
