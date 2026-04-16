//! Document state coordination: hydration and external-change handling.
//!
//! This module sits between `YDocManager` (open-doc management),
//! `DbState` (persistence) and `SyncManager` (P2P broadcast).
//! It never introduces a compile-time dependency from YDocManager → sync;
//! instead it accesses all three via `AppHandle::state()`.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tauri::{ipc::Channel, AppHandle, Manager};
use uuid::Uuid;
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, StateVector, Transact};

use crate::error::{AppError, AppResult};
use crate::workspace::state::{DbState, WorkspaceState};
use crate::yjs::manager::{ReloadStatus, YDocManager};
use crate::yjs::{apply_update_to_doc, content_hash, create_doc, fill_doc_with_markdown};
use entity::workspace::documents;

// ── Public types ────────────────────────────────────────────

/// Progress message sent through Tauri Channel during hydration.
#[derive(Clone, serde::Serialize)]
pub struct HydrateProgress {
    pub current: usize,
    pub total: usize,
}

/// Summary returned when hydration completes.
#[derive(Clone, serde::Serialize)]
pub struct HydrateResult {
    pub generated: usize,
    pub merged: usize,
    pub skipped: usize,
    pub failed: usize,
}

// ── Hydration ───────────────────────────────────────────────

/// Ensure every document in the workspace has a valid `yjs_state`.
///
/// Must run after `reconcile_with_db` and before `subscribe_workspace`.
pub async fn hydrate_workspace(
    app: &AppHandle,
    workspace_uuid: Uuid,
    workspace_path: &str,
    channel: &Channel<HydrateProgress>,
) -> AppResult<HydrateResult> {
    let db_state = app.state::<DbState>();
    let guard = db_state.workspace_db(&workspace_uuid).await?;
    let db = guard.conn();

    let all_docs = documents::Entity::find()
        .filter(documents::Column::WorkspaceId.eq(workspace_uuid))
        .all(db)
        .await?;

    let total = all_docs.len();
    let mut result = HydrateResult {
        generated: 0,
        merged: 0,
        skipped: 0,
        failed: 0,
    };

    for (i, doc_model) in all_docs.into_iter().enumerate() {
        let _ = channel.send(HydrateProgress {
            current: i + 1,
            total,
        });

        match hydrate_single_doc(db, workspace_path, doc_model).await {
            Ok(HydrateAction::Generated) => result.generated += 1,
            Ok(HydrateAction::Merged) => result.merged += 1,
            Ok(HydrateAction::Skipped) => result.skipped += 1,
            Err(e) => {
                tracing::warn!("hydrate failed for doc: {e}");
                result.failed += 1;
            }
        }
    }

    tracing::info!(
        "Hydrate workspace {workspace_uuid}: generated={}, merged={}, skipped={}, failed={}",
        result.generated,
        result.merged,
        result.skipped,
        result.failed
    );

    Ok(result)
}

enum HydrateAction {
    Generated,
    Merged,
    Skipped,
}

async fn hydrate_single_doc(
    db: &sea_orm::DatabaseConnection,
    workspace_path: &str,
    doc_model: documents::Model,
) -> AppResult<HydrateAction> {
    let file_path = std::path::Path::new(workspace_path).join(&doc_model.rel_path);

    // Read current .md content
    let md_content = match tokio::fs::read_to_string(&file_path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File missing on disk — skip, preserve existing yjs_state
            return Ok(HydrateAction::Skipped);
        }
        Err(e) => return Err(AppError::Io(e)),
    };

    let file_hash = content_hash(&md_content);

    match &doc_model.yjs_state {
        None => {
            // Case 1: No yjs_state — generate from .md
            let m = generate_doc_state(&md_content, file_hash);
            persist_doc_state(db, doc_model, &m).await?;
            Ok(HydrateAction::Generated)
        }
        Some(_) => {
            // Check if file was externally modified
            if doc_model
                .file_hash
                .as_ref()
                .is_some_and(|h| *h == file_hash)
            {
                return Ok(HydrateAction::Skipped);
            }

            // Case 2: Hash mismatch — CRDT merge
            let m = merge_external_change(&doc_model, &md_content, file_hash)?;
            persist_doc_state(db, doc_model, &m).await?;
            Ok(HydrateAction::Merged)
        }
    }
}

// ── External file change handling ───────────────────────────

/// Unified entry point for the file watcher.
///
/// Dispatches to `YDocManager::reload_from_file` for open docs,
/// falls back to DB-level merge + GossipSub broadcast for closed docs.
pub async fn handle_file_change(app: &AppHandle, label: &str, rel_path: &str) -> AppResult<()> {
    let ydoc_mgr = app.state::<YDocManager>();
    let status = ydoc_mgr.reload_from_file(app, label, rel_path).await?;

    if status != ReloadStatus::NotOpen {
        return Ok(());
    }

    // Closed doc path: DB merge + GossipSub broadcast
    handle_closed_doc_change(app, label, rel_path).await
}

async fn handle_closed_doc_change(app: &AppHandle, label: &str, rel_path: &str) -> AppResult<()> {
    let ws_state = app.state::<WorkspaceState>();
    let ws_info = ws_state
        .get_by_label(label)
        .await
        .ok_or(AppError::NoWorkspaceOpen)?;

    let workspace_uuid = ws_info.id;
    let workspace_path = &ws_info.path;

    // Read new .md content
    let file_path = std::path::Path::new(workspace_path.as_str()).join(rel_path);
    let new_content = match tokio::fs::read_to_string(&file_path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(AppError::Io(e)),
    };

    let db_state = app.state::<DbState>();
    let guard = db_state.workspace_db(&workspace_uuid).await?;
    let db = guard.conn();

    let doc_model = documents::Entity::find()
        .filter(documents::Column::RelPath.eq(rel_path))
        .filter(documents::Column::WorkspaceId.eq(workspace_uuid))
        .one(db)
        .await?;

    let Some(doc_model) = doc_model else {
        return Ok(()); // No DB record for this file
    };

    let file_hash = content_hash(&new_content);

    // Self-write detection
    if doc_model
        .file_hash
        .as_ref()
        .is_some_and(|h| *h == file_hash)
    {
        return Ok(());
    }

    let doc_id = doc_model.id;

    let m = match &doc_model.yjs_state {
        Some(_) => merge_external_change(&doc_model, &new_content, file_hash)?,
        None => generate_doc_state(&new_content, file_hash),
    };

    persist_doc_state(db, doc_model, &m).await?;

    // Broadcast via GossipSub (if network is running)
    if let Some(net_state) = app.try_state::<crate::network::NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            sync_mgr
                .publish_doc_update(workspace_uuid, doc_id, m.diff)
                .await;
        }
    }

    tracing::info!("Closed-doc external change processed: {rel_path} (doc {doc_id})");
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────

/// Computed document state ready for DB persistence and/or GossipSub broadcast.
struct DocState {
    yjs_state: Vec<u8>,
    state_vector: Vec<u8>,
    file_hash: Vec<u8>,
    /// Incremental diff (equals `yjs_state` for freshly generated docs).
    diff: Vec<u8>,
}

/// Generate yjs_state from markdown content (no prior CRDT history).
fn generate_doc_state(md_content: &str, file_hash: Vec<u8>) -> DocState {
    let doc = create_doc();
    fill_doc_with_markdown(&doc, md_content);
    let txn = doc.transact();
    let yjs_state = txn.encode_state_as_update_v1(&StateVector::default());
    let state_vector = txn.state_vector().encode_v1();
    let diff = yjs_state.clone();
    DocState {
        yjs_state,
        state_vector,
        file_hash,
        diff,
    }
}

/// Merge an external .md change into existing CRDT state, preserving history.
fn merge_external_change(
    doc_model: &documents::Model,
    new_md: &str,
    file_hash: Vec<u8>,
) -> AppResult<DocState> {
    let doc = create_doc();

    if let Some(ref existing) = doc_model.yjs_state {
        // Tolerate legacy BlockNote states — they decode cleanly but leave our
        // Y.Text empty. `replace_doc_content` then diffs against an empty string,
        // effectively rebuilding the doc from `new_md`.
        if let Err(e) = apply_update_to_doc(&doc, existing, "existing yjs_state") {
            tracing::warn!("legacy yjs_state decode failed, rebuilding from .md: {e}");
        }
    }

    // Capture state vector before merge
    let sv_before = doc.transact().state_vector();

    // CRDT merge: text-diff replace so concurrent char-level edits are preserved.
    super::replace_doc_content(&doc, new_md);

    let txn = doc.transact();
    let diff = txn.encode_state_as_update_v1(&sv_before);
    let yjs_state = txn.encode_state_as_update_v1(&StateVector::default());
    let state_vector = txn.state_vector().encode_v1();

    Ok(DocState {
        yjs_state,
        state_vector,
        file_hash,
        diff,
    })
}

async fn persist_doc_state(
    db: &sea_orm::DatabaseConnection,
    doc_model: documents::Model,
    state: &DocState,
) -> AppResult<()> {
    let mut model: documents::ActiveModel = doc_model.into();
    model.yjs_state = Set(Some(state.yjs_state.clone()));
    model.state_vector = Set(Some(state.state_vector.clone()));
    model.file_hash = Set(Some(state.file_hash.clone()));
    model.update(db).await?;
    Ok(())
}
