use entity::workspace::{deletion_log, documents, folders};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use tauri::{Emitter, State};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::fs::crud as fs_crud;
use crate::identity::IdentityState;
use crate::workspace::state::{DbState, WorkspaceState};
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
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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

#[derive(serde::Deserialize)]
pub struct MoveDocumentInput {
    /// 源路径（文件或目录），相对工作区根。
    pub from_rel_path: String,
    /// 目标完整路径（不是目标父目录），相对工作区根。
    pub to_rel_path: String,
}

#[derive(serde::Serialize)]
pub struct MoveDocumentResult {
    pub new_rel_path: String,
    pub is_dir: bool,
}

#[tauri::command]
pub async fn rename_document(
    window: tauri::Window,
    input: RenameDocumentInput,
    db_state: State<'_, DbState>,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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
    let guard = db_state.workspace_db_by_label(window.label()).await?;

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
    let guard = db_state.workspace_db_by_label(window.label()).await?;
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

// ── Move ──

/// Atomically move a document or folder from `from_rel_path` to `to_rel_path`.
///
/// - For a file: moves the `.md` file (plus its `.assets/` sidecar if present),
///   updates the single DB row, and notifies `YDocManager` so any open Y.Doc
///   keeps its handle under the new path.
/// - For a folder: physically renames the folder on disk and updates every
///   DB row whose `rel_path` starts with the old prefix; each currently-open
///   Y.Doc under that prefix is also rebased.
///
/// The target must not exist and must not be a descendant of the source
/// (folder-into-self is rejected).
#[tauri::command]
pub async fn move_document(
    app: tauri::AppHandle,
    window: tauri::Window,
    input: MoveDocumentInput,
    db_state: State<'_, DbState>,
    ws_state: State<'_, WorkspaceState>,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<MoveDocumentResult> {
    let label = window.label().to_owned();
    let ws_path_str = ws_state.workspace_path_for(&label).await?;
    let ws_path = std::path::PathBuf::from(&ws_path_str);

    let from_rel = input.from_rel_path;
    let to_rel = input.to_rel_path;

    // 1) Physical move (synchronous, wrapped in spawn_blocking).
    let move_result = {
        let ws_path = ws_path.clone();
        let from_rel = from_rel.clone();
        let to_rel = to_rel.clone();
        tokio::task::spawn_blocking(move || fs_crud::move_node(&ws_path, &from_rel, &to_rel))
            .await
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))??
    };

    // 2) Update database rows and rebase open Y.Docs.
    let guard = db_state.workspace_db_by_label(&label).await?;
    let db = guard.conn();

    if move_result.is_dir {
        // Folder case: update every document whose rel_path starts with `from_rel/`.
        let prefix_from = if from_rel.ends_with('/') {
            from_rel.clone()
        } else {
            format!("{from_rel}/")
        };
        let prefix_to = if to_rel.ends_with('/') {
            to_rel.clone()
        } else {
            format!("{to_rel}/")
        };

        let docs = documents::Entity::find()
            .filter(documents::Column::RelPath.starts_with(&prefix_from))
            .all(db)
            .await?;

        for doc in docs {
            let new_path = format!("{prefix_to}{}", &doc.rel_path[prefix_from.len()..]);
            let doc_uuid = doc.id;
            let mut active: documents::ActiveModel = doc.into();
            active.rel_path = Set(new_path.clone());
            active.update(db).await?;
            ydoc_mgr.rename_doc(&label, doc_uuid, &new_path);
        }
    } else if let Some(doc) = documents::Entity::find()
        .filter(documents::Column::RelPath.eq(&from_rel))
        .one(db)
        .await?
    {
        let doc_uuid = doc.id;
        let mut active: documents::ActiveModel = doc.into();
        active.rel_path = Set(to_rel.clone());
        active.update(db).await?;
        ydoc_mgr.rename_doc(&label, doc_uuid, &to_rel);
    }

    // 3) Notify the window to rescan the tree. Redundant with the fs watcher
    //    (which would also fire on the physical rename), but emitting directly
    //    after the DB commit guarantees the frontend sees DB + tree consistent
    //    instead of racing against the watcher's 200ms debounce.
    if let Err(e) = app.emit_to(&label, "fs:tree-changed", ()) {
        log::warn!("Failed to emit fs:tree-changed after move: {e}");
    }

    Ok(MoveDocumentResult {
        new_rel_path: to_rel,
        is_dir: move_result.is_dir,
    })
}
