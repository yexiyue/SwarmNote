//! Workspace-level core: `WorkspaceCore` owns every per-workspace resource
//! — DB connection, filesystem, file watcher, YDocManager, document CRUD —
//! and is handed out by `AppCore::open_workspace`.
//!
//! Desktop may hold many `Arc<WorkspaceCore>` instances (one per workspace,
//! shared across windows of the same workspace). Mobile holds at most one.

pub mod db;
pub mod sync;

use std::path::Path;
use std::sync::{Arc, Weak};

use chrono::{DateTime, Utc};
use entity::workspace::{workspaces, workspaces::Entity as WorkspacesEntity};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppCore;
use crate::document::DocumentCrud;
use crate::error::AppResult;
use crate::events::{AppEvent, EventBus};
use crate::fs::{FileEvent, FileEventCallback, FileSystem, FileWatcher};
use crate::yjs::manager::YDocManager;

/// Runtime + DB record of an open workspace. Returned to the frontend by
/// `get_workspace_info`-style commands; held by [`WorkspaceCore`] as its
/// own metadata snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Workspace-level unit. Constructed only by [`AppCore::open_workspace`] —
/// never directly.
///
/// **Lifecycle**: host MUST call [`WorkspaceCore::close`] before the last
/// `Arc<WorkspaceCore>` reference is dropped. `Drop` aborts background
/// tasks as a best-effort fallback but does NOT flush pending writes —
/// `close().await` is the only path that guarantees persistence.
pub struct WorkspaceCore {
    pub info: WorkspaceInfo,
    /// Shared DB connection. Wrapped in `Arc` so `DocumentCrud`,
    /// `YDocManager`, and future `WorkspaceSync` can hold it without
    /// cloning the underlying pool.
    db: Arc<DatabaseConnection>,
    fs: Arc<dyn FileSystem>,
    watcher: Option<Arc<dyn FileWatcher>>,
    ydoc: Arc<YDocManager>,
    documents: Arc<DocumentCrud>,
    event_bus: Arc<dyn EventBus>,
    /// Weak back-reference to the owning [`AppCore`] — avoids the obvious
    /// AppCore ↔ WorkspaceCore ownership cycle. Unused in PR #2; PR #3's
    /// `WorkspaceSync` uses it to reach `NetManager`.
    _app: Weak<AppCore>,
}

impl WorkspaceCore {
    /// Construct a new workspace runtime. Called by
    /// [`AppCore::open_workspace`] — not a public entry point.
    pub(crate) async fn new(
        info: WorkspaceInfo,
        db: DatabaseConnection,
        fs: Arc<dyn FileSystem>,
        watcher: Option<Arc<dyn FileWatcher>>,
        event_bus: Arc<dyn EventBus>,
        peer_id: String,
        app: Weak<AppCore>,
    ) -> AppResult<Arc<Self>> {
        let db = Arc::new(db);
        let documents = Arc::new(DocumentCrud::new(Arc::clone(&db), peer_id.clone()));
        let ydoc = YDocManager::new(
            info.id,
            Arc::clone(&fs),
            Arc::clone(&event_bus),
            Arc::clone(&db),
            peer_id,
        );

        // Start the watcher (if any) BEFORE we hand out `Arc<Self>` so
        // reload callbacks see a fully-formed workspace.
        if let Some(w) = watcher.as_ref() {
            let callback =
                build_watcher_callback(info.id, Arc::clone(&ydoc), Arc::clone(&event_bus));
            w.watch(callback).await?;
        }

        Ok(Arc::new(Self {
            info,
            db,
            fs,
            watcher,
            ydoc,
            documents,
            event_bus,
            _app: app,
        }))
    }

    pub fn id(&self) -> Uuid {
        self.info.id
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn fs(&self) -> &Arc<dyn FileSystem> {
        &self.fs
    }

    pub fn watcher(&self) -> Option<&Arc<dyn FileWatcher>> {
        self.watcher.as_ref()
    }

    pub fn ydoc(&self) -> &Arc<YDocManager> {
        &self.ydoc
    }

    pub fn documents(&self) -> &Arc<DocumentCrud> {
        &self.documents
    }

    pub fn event_bus(&self) -> &Arc<dyn EventBus> {
        &self.event_bus
    }

    /// Flush every open Y.Doc, stop the file watcher, and tear down
    /// background tasks. Must be called before the last `Arc<WorkspaceCore>`
    /// reference drops — otherwise pending writeback tasks are aborted
    /// mid-flight and data can be lost.
    pub async fn close(&self) {
        self.ydoc.close_all().await;
        if let Some(w) = &self.watcher {
            w.unwatch().await;
        }
        tracing::info!("WorkspaceCore closed: {}", self.info.id);
    }
}

impl Drop for WorkspaceCore {
    fn drop(&mut self) {
        // Best-effort warning: if we reach Drop with dirty docs, the host
        // forgot to call `close().await`. We can't run async cleanup here
        // (no reliable runtime handle), so just log — the writeback tasks
        // will be aborted when the `Arc<DocEntry>`s drop.
        let open = self.ydoc.list_open_doc_uuids();
        if !open.is_empty() {
            tracing::warn!(
                "WorkspaceCore {} dropped with {} open docs; host should have called close().await first",
                self.info.id,
                open.len()
            );
        }
    }
}

/// Build the callback handed to `FileWatcher::watch`:
///
/// 1. Emit [`AppEvent::FileTreeChanged`] so the frontend re-scans the tree.
/// 2. For each modified `.md` path, spawn a tokio task calling
///    [`YDocManager::reload_from_file`] — that handles self-write detection
///    and fires `ExternalUpdate` / `ExternalConflict` events as appropriate.
fn build_watcher_callback(
    workspace_id: Uuid,
    ydoc: Arc<YDocManager>,
    event_bus: Arc<dyn EventBus>,
) -> FileEventCallback {
    // Per the [`FileWatcher`] trait contract, implementations MUST invoke
    // this callback from a tokio runtime context, so `tokio::spawn` is safe.
    Arc::new(move |events: Vec<FileEvent>| {
        event_bus.emit(AppEvent::FileTreeChanged { workspace_id });

        for ev in events {
            let rel = match ev {
                FileEvent::Modified(r) | FileEvent::Created(r) | FileEvent::Deleted(r) => r,
                FileEvent::Renamed { to, .. } => to,
            };
            if !rel.ends_with(".md") {
                continue;
            }
            let ydoc = Arc::clone(&ydoc);
            tokio::spawn(async move {
                if let Err(e) = ydoc.reload_from_file(&rel).await {
                    tracing::warn!("reload_from_file({rel}) failed: {e}");
                }
            });
        }
    })
}

/// Read the workspace UUID from `{path}/.swarmnote/workspace.db` without
/// running migrations or keeping the connection open. Used by
/// [`AppCore::open_workspace`] to dedup concurrent opens of the same
/// workspace across multiple windows.
pub async fn peek_workspace_uuid(path: &Path) -> AppResult<Option<Uuid>> {
    let db_path = path.join(".swarmnote").join("workspace.db");
    if !db_path.exists() {
        return Ok(None);
    }
    let db = db::connect_sqlite(&db_path).await?;
    let row = WorkspacesEntity::find().one(&db).await?;
    // Sea-orm doesn't expose an explicit close — dropping the connection
    // closes the underlying pool.
    drop(db);
    Ok(row.map(|w| w.id))
}

/// Load or create the workspace row in `workspace.db`. Returns the
/// [`WorkspaceInfo`] populated with runtime fields (`path`).
pub async fn load_or_create_workspace_info(
    db: &DatabaseConnection,
    path: &Path,
    peer_id: &str,
) -> AppResult<WorkspaceInfo> {
    if let Some(row) = WorkspacesEntity::find().one(db).await? {
        return Ok(WorkspaceInfo {
            id: row.id,
            name: row.name,
            path: path.to_string_lossy().into_owned(),
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".to_string());
    let now = Utc::now();
    let id = Uuid::now_v7();

    let model = workspaces::ActiveModel {
        id: Set(id),
        name: Set(name.clone()),
        created_by: Set(peer_id.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = model.insert(db).await?;
    Ok(WorkspaceInfo {
        id: created.id,
        name: created.name,
        path: path.to_string_lossy().into_owned(),
        created_by: created.created_by,
        created_at: created.created_at,
        updated_at: created.updated_at,
    })
}

/// Register (or overwrite) the workspace row with a specific UUID — used
/// when a sync peer tells us the authoritative workspace ID. Idempotent.
pub async fn ensure_workspace_row(
    db: &DatabaseConnection,
    id: Uuid,
    name: &str,
    peer_id: &str,
) -> AppResult<WorkspaceInfo> {
    if let Some(existing) = WorkspacesEntity::find_by_id(id).one(db).await? {
        return Ok(WorkspaceInfo {
            id: existing.id,
            name: existing.name,
            path: String::new(), // populated by caller
            created_by: existing.created_by,
            created_at: existing.created_at,
            updated_at: existing.updated_at,
        });
    }
    // Skip matching on name — the UUID is authoritative.
    let _ = WorkspacesEntity::find()
        .filter(workspaces::Column::Name.eq(name))
        .one(db)
        .await?;

    let now = Utc::now();
    let model = workspaces::ActiveModel {
        id: Set(id),
        name: Set(name.to_owned()),
        created_by: Set(peer_id.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = model.insert(db).await?;
    Ok(WorkspaceInfo {
        id: created.id,
        name: created.name,
        path: String::new(),
        created_by: created.created_by,
        created_at: created.created_at,
        updated_at: created.updated_at,
    })
}
