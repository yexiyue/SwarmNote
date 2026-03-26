use chrono::Utc;
use entity::workspace::{documents, folders};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tauri::State;
use uuid::Uuid;

use super::peer_id;
use crate::error::{AppError, AppResult};
use crate::identity::IdentityState;
use crate::workspace::state::DbState;

// ── Document ──

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
    window: tauri::Window,
    workspace_id: String,
    db_state: State<'_, DbState>,
) -> AppResult<Vec<documents::Model>> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    Ok(documents::Entity::find()
        .filter(documents::Column::WorkspaceId.eq(&workspace_id))
        .all(guard.conn())
        .await?)
}

#[tauri::command]
pub async fn db_upsert_document(
    window: tauri::Window,
    input: UpsertDocumentInput,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
) -> AppResult<documents::Model> {
    let guard = db_state.workspace_db_for(window.label()).await?;
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
pub async fn db_delete_document(
    window: tauri::Window,
    id: String,
    db_state: State<'_, DbState>,
) -> AppResult<()> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    documents::Entity::delete_by_id(&id)
        .exec(guard.conn())
        .await?;
    Ok(())
}

// ── Folder ──

#[derive(serde::Deserialize)]
pub struct CreateFolderInput {
    pub workspace_id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub rel_path: String,
}

#[tauri::command]
pub async fn db_get_folders(
    window: tauri::Window,
    workspace_id: String,
    db_state: State<'_, DbState>,
) -> AppResult<Vec<folders::Model>> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    Ok(folders::Entity::find()
        .filter(folders::Column::WorkspaceId.eq(&workspace_id))
        .all(guard.conn())
        .await?)
}

#[tauri::command]
pub async fn db_create_folder(
    window: tauri::Window,
    input: CreateFolderInput,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
) -> AppResult<folders::Model> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    let now = Utc::now().timestamp();

    #[allow(clippy::needless_update)]
    let model = folders::ActiveModel {
        id: Set(Uuid::now_v7().to_string()),
        workspace_id: Set(input.workspace_id),
        parent_folder_id: Set(input.parent_folder_id),
        name: Set(input.name),
        rel_path: Set(input.rel_path),
        created_by: Set(peer_id(&identity)?),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    Ok(model.insert(guard.conn()).await?)
}

#[tauri::command]
pub async fn db_delete_folder(
    window: tauri::Window,
    id: String,
    db_state: State<'_, DbState>,
) -> AppResult<()> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    let db = guard.conn();

    let child_folders = folders::Entity::find()
        .filter(folders::Column::ParentFolderId.eq(Some(id.clone())))
        .count(db)
        .await?;
    if child_folders > 0 {
        return Err(AppError::FolderNotEmpty("contains sub-folders".into()));
    }

    let child_docs = documents::Entity::find()
        .filter(documents::Column::FolderId.eq(Some(id.clone())))
        .count(db)
        .await?;
    if child_docs > 0 {
        return Err(AppError::FolderNotEmpty("contains documents".into()));
    }

    folders::Entity::delete_by_id(&id).exec(db).await?;
    Ok(())
}
