use std::path::PathBuf;

use chrono::Utc;
use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tauri::State;
use uuid::Uuid;

use super::peer_id;
use crate::db::{init_workspace_db, state::DbState};
use crate::error::AppResult;
use crate::identity::IdentityState;

#[derive(serde::Deserialize)]
pub struct InitWorkspaceInput {
    pub path: String,
    pub name: String,
}

#[tauri::command]
pub async fn db_init_workspace(
    input: InitWorkspaceInput,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
) -> AppResult<workspaces::Model> {
    let conn = init_workspace_db(&PathBuf::from(&input.path)).await?;

    let workspace = match workspaces::Entity::find().one(&conn).await? {
        Some(ws) => ws,
        None => {
            let now = Utc::now().timestamp();
            #[allow(clippy::needless_update)]
            let model = workspaces::ActiveModel {
                id: Set(Uuid::now_v7().to_string()),
                name: Set(input.name),
                created_by: Set(peer_id(&identity)),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            model.insert(&conn).await?
        }
    };

    *db_state.workspace_db.write().await = Some(conn);
    Ok(workspace)
}
