//! [`AppCore`] — device-level singleton. Holds identity, event bus,
//! keychain, global config, and (starting PR #2) the registry of open
//! [`WorkspaceCore`] instances.
//!
//! Future fields populated in PR #3:
//! - `devices_db: Arc<DatabaseConnection>` (paired device table)
//! - `device_manager: Arc<DeviceManager>`
//! - `pairing_manager: Arc<PairingManager>`
//! - `net: Arc<tokio::sync::Mutex<Option<NetManager>>>`
//! - `ctrl_dispatcher: Arc<CtrlMessageDispatcher>`

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::{load_or_create_config, GlobalConfigState};
use crate::error::{AppError, AppResult};
use crate::events::EventBus;
use crate::fs::{FileSystem, FileWatcher};
use crate::identity::IdentityManager;
use crate::keychain::KeychainProvider;
use crate::workspace::{
    self, db::init_workspace_db, load_or_create_workspace_info, WorkspaceCore, WorkspaceInfo,
};

/// Device-level core. Construct once per process via [`AppCore::new`].
pub struct AppCore {
    pub identity: Arc<IdentityManager>,
    pub config: Arc<GlobalConfigState>,
    pub event_bus: Arc<dyn EventBus>,
    pub keychain: Arc<dyn KeychainProvider>,

    /// Registry of active workspace runtimes keyed by workspace UUID.
    /// Stored as `Weak` so dropping the last external `Arc<WorkspaceCore>`
    /// actually frees the workspace; the map is cleaned on demand in
    /// [`AppCore::open_workspace`].
    workspaces: Mutex<HashMap<Uuid, Weak<WorkspaceCore>>>,
}

impl AppCore {
    /// Bootstrap the device-level core.
    pub async fn new(
        keychain: Arc<dyn KeychainProvider>,
        event_bus: Arc<dyn EventBus>,
        app_data_dir: PathBuf,
    ) -> AppResult<Arc<Self>> {
        // 1. Load (or create default) global config.
        let config_data = load_or_create_config(&app_data_dir)?;
        let config_state = Arc::new(GlobalConfigState::new(
            config_data.clone(),
            app_data_dir.join("config.json"),
        ));

        // 2. Initialize device identity.
        let identity = Arc::new(IdentityManager::new(keychain.clone(), &config_data).await?);

        Ok(Arc::new(Self {
            identity,
            config: config_state,
            event_bus,
            keychain,
            workspaces: Mutex::new(HashMap::new()),
        }))
    }

    /// Open (or return existing) workspace at `path`.
    ///
    /// Concurrency: if another window already opened the same workspace,
    /// the existing [`Arc<WorkspaceCore>`] is returned — desktop windows
    /// share one runtime instance per workspace. Mobile hosts at most one
    /// workspace active at a time.
    ///
    /// - `fs` — typically `Arc::new(LocalFs::new(&path))`.
    /// - `watcher` — `Some(NotifyFileWatcher::new())` on desktop; `None` on
    ///   mobile (sandbox never sees external changes).
    pub async fn open_workspace(
        self: &Arc<Self>,
        path: PathBuf,
        fs: Arc<dyn FileSystem>,
        watcher: Option<Arc<dyn FileWatcher>>,
    ) -> AppResult<Arc<WorkspaceCore>> {
        if !path.is_dir() {
            return Err(AppError::InvalidPath(path.to_string_lossy().into_owned()));
        }

        // Peek UUID without holding the workspaces lock across the DB open
        // (so other open_workspace calls to a different path don't serialize).
        let peeked = workspace::peek_workspace_uuid(&path).await?;

        let mut guard = self.workspaces.lock().await;

        // Fast path: if the peeked UUID is already registered and live, reuse it.
        if let Some(uuid) = peeked {
            if let Some(weak) = guard.get(&uuid) {
                if let Some(arc) = weak.upgrade() {
                    return Ok(arc);
                }
                // Stale entry — caller dropped the Arc but we never ran the
                // cleanup. Remove now.
                guard.remove(&uuid);
            }
        }

        // Miss: initialize DB + workspace row + core.
        let db = init_workspace_db(&path).await?;
        let peer_id = self.identity.peer_id()?;
        let mut info = load_or_create_workspace_info(&db, &path, &peer_id).await?;
        info.path = path.to_string_lossy().into_owned();

        let workspace_id = info.id;
        let core = WorkspaceCore::new(
            info,
            db,
            fs,
            watcher,
            self.event_bus.clone(),
            peer_id,
            Arc::downgrade(self),
        )
        .await?;

        guard.insert(workspace_id, Arc::downgrade(&core));
        Ok(core)
    }

    /// Close the workspace with the given UUID. `flush()` runs regardless
    /// of the number of outstanding `Arc` references — this is an
    /// authoritative shutdown hook used by the host when the last window
    /// referencing a workspace closes.
    pub async fn close_workspace(&self, uuid: Uuid) -> AppResult<()> {
        let mut guard = self.workspaces.lock().await;
        let Some(weak) = guard.remove(&uuid) else {
            return Ok(());
        };
        drop(guard);

        if let Some(arc) = weak.upgrade() {
            arc.close().await;
        }
        Ok(())
    }

    /// Look up an active workspace by UUID. Returns `None` if the workspace
    /// is not open or if all external `Arc`s were dropped.
    pub async fn get_workspace(&self, uuid: &Uuid) -> Option<Arc<WorkspaceCore>> {
        let guard = self.workspaces.lock().await;
        guard.get(uuid).and_then(|w| w.upgrade())
    }

    /// Snapshot of every live workspace (active `Arc` upgrades only).
    /// Stale `Weak` entries are *not* pruned here — that happens on the
    /// next `open_workspace` call.
    pub async fn list_workspaces(&self) -> Vec<Arc<WorkspaceCore>> {
        let guard = self.workspaces.lock().await;
        guard.values().filter_map(|w| w.upgrade()).collect()
    }

    /// Resolve `WorkspaceInfo` for a workspace UUID without forcing the
    /// caller to hold an `Arc<WorkspaceCore>`. Returns `None` when not open.
    pub async fn workspace_info(&self, uuid: &Uuid) -> Option<WorkspaceInfo> {
        self.get_workspace(uuid).await.map(|w| w.info.clone())
    }

    /// Read access to the underlying app_data_dir-relative config path.
    pub fn config_path(&self) -> &Path {
        self.config.path()
    }
}
