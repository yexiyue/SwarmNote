use chrono::Utc;
use entity::workspace::documents;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tauri::State;
use uuid::Uuid;

use super::peer_id;
use crate::db::state::DbState;
use crate::error::AppResult;
use crate::identity::IdentityState;

#[derive(serde::Deserialize)]
pub struct UpsertDocumentInput {
    pub id: Option<String>,
    pub workspace_id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub rel_path: String,
}

#[tauri::command]
pub async fn db_get_documents(
    workspace_id: String,
    db_state: State<'_, DbState>,
) -> AppResult<Vec<documents::Model>> {
    let guard = db_state.workspace_db().await?;
    Ok(documents::Entity::find()
        .filter(documents::Column::WorkspaceId.eq(&workspace_id))
        .all(guard.conn())
        .await?)
}

#[tauri::command]
pub async fn db_upsert_document(
    input: UpsertDocumentInput,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
) -> AppResult<documents::Model> {
    let guard = db_state.workspace_db().await?;
    let db = guard.conn();
    let now = Utc::now().timestamp();

    if let Some(ref id) = input.id {
        if let Some(existing) = documents::Entity::find_by_id(id).one(db).await? {
            let mut model: documents::ActiveModel = existing.into();
            model.title = Set(input.title);
            model.folder_id = Set(input.folder_id);
            model.rel_path = Set(input.rel_path);
            model.updated_at = Set(now);
            return Ok(model.update(db).await?);
        }
    }

    #[allow(clippy::needless_update)]
    let model = documents::ActiveModel {
        id: Set(input.id.unwrap_or_else(|| Uuid::now_v7().to_string())),
        workspace_id: Set(input.workspace_id),
        folder_id: Set(input.folder_id),
        title: Set(input.title),
        rel_path: Set(input.rel_path),
        file_hash: Set(None),
        yjs_state: Set(None),
        state_vector: Set(None),
        created_by: Set(peer_id(&identity)?),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    Ok(model.insert(db).await?)
}

#[tauri::command]
pub async fn db_delete_document(id: String, db_state: State<'_, DbState>) -> AppResult<()> {
    let guard = db_state.workspace_db().await?;
    documents::Entity::delete_by_id(&id)
        .exec(guard.conn())
        .await?;
    Ok(())
}
