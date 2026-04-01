use entity::workspace::{deletion_log, documents, folders};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tauri::State;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::identity::IdentityState;
use crate::workspace::state::DbState;
use crate::yjs::manager::YDocManager;

// ── Document ──

#[derive(serde::Deserialize)]
pub struct UpsertDocumentInput {
    pub id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub title: String,
    pub rel_path: String,
    pub file_hash: Option<String>,
}

#[tauri::command]
pub async fn db_get_documents(
    window: tauri::Window,
    workspace_id: Uuid,
    db_state: State<'_, DbState>,
) -> AppResult<Vec<documents::Model>> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    Ok(documents::Entity::find()
        .filter(documents::Column::WorkspaceId.eq(workspace_id))
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

    if let Some(id) = input.id {
        if let Some(existing) = documents::Entity::find_by_id(id).one(db).await? {
            let mut model: documents::ActiveModel = existing.into();
            model.title = Set(input.title);
            model.folder_id = Set(input.folder_id);
            model.rel_path = Set(input.rel_path);
            if let Some(hash) = input.file_hash {
                model.file_hash = Set(Some(hash.into_bytes()));
            }
            return Ok(model.update(db).await?);
        }
    }

    let model = documents::ActiveModel {
        id: Set(input.id.unwrap_or_else(Uuid::now_v7)),
        workspace_id: Set(input.workspace_id),
        folder_id: Set(input.folder_id),
        title: Set(input.title),
        rel_path: Set(input.rel_path),
        file_hash: Set(input.file_hash.map(|h| h.into_bytes())),
        yjs_state: Set(None),
        state_vector: Set(None),
        lamport_clock: Set(0),
        created_by: Set(identity.peer_id()?),
        ..Default::default()
    };
    Ok(model.insert(db).await?)
}

#[tauri::command]
pub async fn delete_document_by_rel_path(
    window: tauri::Window,
    rel_path: String,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
) -> AppResult<()> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    let db = guard.conn();

    let Some(doc) = documents::Entity::find()
        .filter(documents::Column::RelPath.eq(&rel_path))
        .one(db)
        .await?
    else {
        return Ok(()); // no record — nothing to delete
    };

    let doc_id = doc.id;

    // Write tombstone to deletion_log
    let tombstone = deletion_log::ActiveModel {
        doc_id: Set(doc_id),
        rel_path: Set(rel_path),
        deleted_at: Set(chrono::Utc::now()),
        deleted_by: Set(identity.peer_id()?),
        lamport_clock: Set(doc.lamport_clock + 1),
    };
    deletion_log::Entity::insert(tombstone)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(deletion_log::Column::DocId)
                .update_columns([
                    deletion_log::Column::DeletedAt,
                    deletion_log::Column::DeletedBy,
                    deletion_log::Column::LamportClock,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    // Remove the document record
    documents::Entity::delete_by_id(doc_id).exec(db).await?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct RenameDocumentInput {
    pub old_rel_path: String,
    pub new_rel_path: String,
    pub new_title: String,
}

#[tauri::command]
pub async fn rename_document(
    window: tauri::Window,
    input: RenameDocumentInput,
    db_state: State<'_, DbState>,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    let db = guard.conn();

    let Some(doc) = documents::Entity::find()
        .filter(documents::Column::RelPath.eq(&input.old_rel_path))
        .one(db)
        .await?
    else {
        return Ok(()); // no record — nothing to rename
    };

    let doc_uuid = doc.id;
    let mut model: documents::ActiveModel = doc.into();
    model.rel_path = Set(input.new_rel_path.clone());
    model.title = Set(input.new_title);
    model.update(db).await?;

    // Update in-memory YDocManager entry
    ydoc_mgr.rename_doc(window.label(), doc_uuid, &input.new_rel_path);

    Ok(())
}

#[tauri::command]
pub async fn delete_documents_by_prefix(
    window: tauri::Window,
    prefix: String,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
) -> AppResult<u64> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    let db = guard.conn();

    let docs = documents::Entity::find()
        .filter(documents::Column::RelPath.starts_with(&prefix))
        .all(db)
        .await?;

    let count = docs.len() as u64;
    if count == 0 {
        return Ok(0);
    }

    let now = chrono::Utc::now();
    let peer_id = identity.peer_id()?;

    for doc in docs {
        let tombstone = deletion_log::ActiveModel {
            doc_id: Set(doc.id),
            rel_path: Set(doc.rel_path),
            deleted_at: Set(now),
            deleted_by: Set(peer_id.clone()),
            lamport_clock: Set(doc.lamport_clock + 1),
        };
        deletion_log::Entity::insert(tombstone)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(deletion_log::Column::DocId)
                    .update_columns([
                        deletion_log::Column::DeletedAt,
                        deletion_log::Column::DeletedBy,
                        deletion_log::Column::LamportClock,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await?;

        documents::Entity::delete_by_id(doc.id).exec(db).await?;
    }

    tracing::info!("Cascade-deleted {count} documents under prefix '{prefix}'");
    Ok(count)
}

// ── Folder ──

#[derive(serde::Deserialize)]
pub struct CreateFolderInput {
    pub workspace_id: Uuid,
    pub parent_folder_id: Option<Uuid>,
    pub name: String,
    pub rel_path: String,
}

#[tauri::command]
pub async fn db_get_folders(
    window: tauri::Window,
    workspace_id: Uuid,
    db_state: State<'_, DbState>,
) -> AppResult<Vec<folders::Model>> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    Ok(folders::Entity::find()
        .filter(folders::Column::WorkspaceId.eq(workspace_id))
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

    let model = folders::ActiveModel {
        workspace_id: Set(input.workspace_id),
        parent_folder_id: Set(input.parent_folder_id),
        name: Set(input.name),
        rel_path: Set(input.rel_path),
        created_by: Set(identity.peer_id()?),
        ..Default::default()
    };
    Ok(model.insert(guard.conn()).await?)
}

#[tauri::command]
pub async fn db_delete_folder(
    window: tauri::Window,
    id: Uuid,
    db_state: State<'_, DbState>,
) -> AppResult<()> {
    let guard = db_state.workspace_db_for(window.label()).await?;
    let db = guard.conn();

    let child_folders = folders::Entity::find()
        .filter(folders::Column::ParentFolderId.eq(Some(id)))
        .count(db)
        .await?;
    if child_folders > 0 {
        return Err(AppError::FolderNotEmpty("contains sub-folders".into()));
    }

    let child_docs = documents::Entity::find()
        .filter(documents::Column::FolderId.eq(Some(id)))
        .count(db)
        .await?;
    if child_docs > 0 {
        return Err(AppError::FolderNotEmpty("contains documents".into()));
    }

    folders::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}
