//! [`AppCore`] — device-level singleton. Holds identity, event bus,
//! keychain, global config, devices DB, and (when P2P is running) the
//! network session manager + per-peer sync state. Also keeps a `Weak`
//! registry of open [`WorkspaceCore`] instances so desktop windows can
//! share one runtime per workspace.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use sea_orm::DatabaseConnection;
use swarm_p2p_core::libp2p::identity::Keypair;
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::config::{load_or_create_config, GlobalConfigState};
use crate::error::{AppError, AppResult};
use crate::events::{AppEvent, EventBus};
use crate::fs::{FileSystem, FileWatcher};
use crate::identity::IdentityManager;
use crate::keychain::KeychainProvider;
use crate::network::config::create_node_config;
use crate::network::event_loop::spawn_event_loop;
use crate::network::{AppNetClient, NetManager, NodeStatus};
use crate::protocol::{AppRequest, AppResponse, OsInfo};
use crate::workspace::sync::{AppSyncCoordinator, WorkspaceSync};
use crate::workspace::{
    self, db::init_devices_db, db::init_workspace_db, load_or_create_workspace_info, WorkspaceCore,
    WorkspaceInfo,
};

/// Device-level core. Construct once per process via [`AppCore::new`].
pub struct AppCore {
    pub identity: Arc<IdentityManager>,
    pub config: Arc<GlobalConfigState>,
    pub event_bus: Arc<dyn EventBus>,
    pub keychain: Arc<dyn KeychainProvider>,

    /// Shared connection to `devices.db` (paired devices table). Always open
    /// for the lifetime of `AppCore` — does not track P2P lifecycle.
    pub devices_db: Arc<DatabaseConnection>,

    /// Absolute path to the app data directory (`~/.swarmnote/` or
    /// platform-specific). Platform layers inject this; core stores for
    /// reference only.
    pub app_data_dir: PathBuf,

    /// Running P2P session. `None` until [`AppCore::start_network`]
    /// completes; reset to `None` by [`AppCore::stop_network`].
    net: Mutex<Option<Arc<NetManager>>>,

    /// Global sync coordinator. Populated together with `net` — starts /
    /// stops with the P2P session. Per-workspace sync state lives in
    /// [`WorkspaceCore::sync`].
    sync_coordinator: Mutex<Option<Arc<AppSyncCoordinator>>>,

    /// Registry of active workspace runtimes keyed by workspace UUID.
    /// Stored as `Weak` so dropping the last external `Arc<WorkspaceCore>`
    /// actually frees the workspace; the map is cleaned on demand in
    /// [`AppCore::open_workspace`].
    workspaces: Mutex<HashMap<Uuid, Weak<WorkspaceCore>>>,
}

impl AppCore {
    /// Bootstrap the device-level core: load config, initialize identity,
    /// open `devices.db`. P2P network stays stopped until the host calls
    /// [`AppCore::start_network`] explicitly.
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

        // 3. Open devices.db.
        let devices_db = Arc::new(init_devices_db(&app_data_dir).await?);

        Ok(Arc::new(Self {
            identity,
            config: config_state,
            event_bus,
            keychain,
            devices_db,
            app_data_dir,
            net: Mutex::new(None),
            sync_coordinator: Mutex::new(None),
            workspaces: Mutex::new(HashMap::new()),
        }))
    }

    /// Open (or return existing) workspace at `path`.
    ///
    /// Concurrency: uses a double-checked pattern so parallel callers
    /// (e.g. two windows opening different workspaces at once) don't
    /// serialize on DB / filesystem I/O. The `workspaces` lock is only
    /// held for the registry peek / insert — the DB open and
    /// [`WorkspaceCore::new`] run **without** the lock held.
    ///
    /// If another task wins the race to insert the same UUID, this
    /// method drops its own freshly-built core and returns the winner's
    /// instance so desktop windows still share one runtime per workspace.
    pub async fn open_workspace(
        self: &Arc<Self>,
        path: PathBuf,
        fs: Arc<dyn FileSystem>,
        watcher: Option<Arc<dyn FileWatcher>>,
    ) -> AppResult<Arc<WorkspaceCore>> {
        if !path.is_dir() {
            return Err(AppError::InvalidPath(path.to_string_lossy().into_owned()));
        }

        // Peek UUID without holding any lock, so two calls to different
        // paths never serialize.
        let peeked = workspace::peek_workspace_uuid(&path).await?;

        // Fast path: if the peeked UUID is already registered and live, reuse it.
        if let Some(uuid) = peeked {
            let guard = self.workspaces.lock().await;
            if let Some(weak) = guard.get(&uuid) {
                if let Some(arc) = weak.upgrade() {
                    return Ok(arc);
                }
            }
            // Fall through to the slow path below — either no entry or a
            // stale Weak. We deliberately drop the lock here so the I/O
            // under slow-path runs without blocking other open_workspace
            // calls.
            drop(guard);
        }

        // Slow path: run DB open + workspace row load WITHOUT holding the
        // registry lock.
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

        // Re-take the lock and double-check: another task may have won the
        // race while we were doing I/O.
        let winner = {
            let mut guard = self.workspaces.lock().await;
            if let Some(weak) = guard.get(&workspace_id) {
                if let Some(existing) = weak.upgrade() {
                    // Lost the race — drop our freshly-built core so its
                    // writeback tasks are aborted and fall through to return
                    // the existing Arc. The drop must happen outside the
                    // guard (close is async).
                    drop(guard);
                    // Close our orphan core so its YDocManager writeback
                    // tasks are cancelled and file watcher released.
                    core.close().await;
                    existing
                } else {
                    guard.insert(workspace_id, Arc::downgrade(&core));
                    drop(guard);
                    core
                }
            } else {
                guard.insert(workspace_id, Arc::downgrade(&core));
                drop(guard);
                core
            }
        };

        // If P2P is running, install per-workspace sync + subscribe.
        if let Some(coordinator) = self.sync_coordinator().await {
            self.install_workspace_sync(&winner, coordinator.client(), true)
                .await;
        }

        Ok(winner)
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

        // WorkspaceCore::close() tears down its WorkspaceSync (if any)
        // including GossipSub unsubscribe — no explicit unsubscribe needed here.
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

    // ── Network lifecycle ──

    /// Current network session, if any.
    pub async fn net(&self) -> Option<Arc<NetManager>> {
        self.net.lock().await.clone()
    }

    /// Current global sync coordinator, if P2P is running.
    pub async fn sync_coordinator(&self) -> Option<Arc<AppSyncCoordinator>> {
        self.sync_coordinator.lock().await.clone()
    }

    /// Node-level status for frontend display.
    pub async fn network_status(&self) -> NodeStatus {
        if self.net.lock().await.is_some() {
            NodeStatus::Running
        } else {
            NodeStatus::Stopped
        }
    }

    /// Start the P2P node. Idempotent: returns `Network("already running")`
    /// if a session is already active.
    pub async fn start_network(self: &Arc<Self>) -> AppResult<()> {
        let mut net_guard = self.net.lock().await;
        if net_guard.is_some() {
            return Err(AppError::Network("P2P node is already running".to_string()));
        }

        let keypair_bytes = self.identity.keypair_protobuf()?;
        let keypair = Keypair::from_protobuf_encoding(&keypair_bytes)
            .map_err(|e| AppError::Identity(format!("keypair decode: {e}")))?;
        let peer_id = keypair.public().to_peer_id();

        // Build agent_version from current device name.
        let device_name = self.identity.device_info()?.device_name;
        let mut os_info = OsInfo::default();
        if device_name != os_info.hostname {
            os_info.name = Some(device_name.clone());
        }
        let agent_version = os_info.to_agent_version(env!("CARGO_PKG_VERSION"));
        let config = create_node_config(agent_version);

        // Start the libp2p swarm + req/resp channels.
        let (client, receiver): (AppNetClient, _) =
            swarm_p2p_core::start::<AppRequest, AppResponse>(keypair, config)
                .map_err(|e| AppError::Network(format!("Failed to start P2P: {e}")))?;

        let net_manager = Arc::new(NetManager::new(
            client.clone(),
            peer_id,
            (*self.devices_db).clone(),
            os_info.name,
        ));
        let cancel_token = net_manager.cancel_token();

        // Global sync coordinator (full-sync de-dup, SV compensation, inbound routing).
        let coordinator = Arc::new(AppSyncCoordinator::new(self.clone(), client.clone()));

        // Event loop.
        spawn_event_loop(
            receiver,
            self.clone(),
            net_manager.client.clone(),
            net_manager.device_manager.clone(),
            net_manager.pairing_manager.clone(),
            coordinator.clone(),
            cancel_token.clone(),
        );

        // Periodic SV compensation.
        coordinator.start_sv_compensation(cancel_token.clone());

        // Subscribe to global ctrl topic for workspace-opened notifications.
        if let Err(e) = client.subscribe(crate::workspace::sync::CTRL_TOPIC).await {
            tracing::warn!("Failed to subscribe to ctrl topic: {e}");
        }

        // Retroactively inject WorkspaceSync into already-open workspaces.
        for ws in self.list_workspaces().await {
            self.install_workspace_sync(&ws, &client, false).await;
        }

        // Background: announce online, bootstrap DHT, reload paired devices,
        // reconnect to paired peers.
        let announcer = net_manager.online_announcer.clone();
        let bootstrap_client = client.clone();
        let pairing_for_bootstrap = net_manager.pairing_manager.clone();

        tokio::spawn(async move {
            if let Err(e) = announcer.announce_online().await {
                tracing::warn!("Failed to announce online: {e}");
            }

            match bootstrap_client.bootstrap().await {
                Ok(_) => info!("DHT bootstrap completed"),
                Err(e) => tracing::warn!("DHT bootstrap failed: {e}"),
            }

            if let Err(e) = pairing_for_bootstrap.load_paired_devices().await {
                tracing::warn!("Failed to load paired devices: {e}");
            }
            let paired_peer_ids = pairing_for_bootstrap.get_paired_peer_ids();
            announcer.check_paired_online(paired_peer_ids).await;
        });

        // Periodic DHT record renewal.
        net_manager
            .online_announcer
            .clone()
            .spawn_renewal_task(cancel_token);

        *net_guard = Some(net_manager);
        drop(net_guard);

        *self.sync_coordinator.lock().await = Some(coordinator);

        self.event_bus.emit(AppEvent::NodeStarted);
        info!("P2P node started, PeerId: {peer_id}");
        Ok(())
    }

    /// Stop the P2P node. No-op if already stopped.
    pub async fn stop_network(&self) -> AppResult<()> {
        let mut net_guard = self.net.lock().await;
        if let Some(manager) = net_guard.take() {
            manager.shutdown().await;
            drop(net_guard);

            // Tear down per-workspace sync runtimes.
            for ws in self.list_workspaces().await {
                if let Some(sync) = ws.take_sync().await {
                    sync.close().await;
                }
            }
            *self.sync_coordinator.lock().await = None;

            self.event_bus.emit(AppEvent::NodeStopped);
            info!("P2P node stopped");
        }
        Ok(())
    }

    /// Convenience: get a shared `AppNetClient` if P2P is running.
    pub async fn client(&self) -> AppResult<AppNetClient> {
        self.net()
            .await
            .map(|n| n.client.clone())
            .ok_or_else(AppError::node_not_running)
    }

    /// Convenience: get `PairingManager` if P2P is running.
    pub async fn pairing(&self) -> AppResult<Arc<crate::pairing::PairingManager>> {
        self.net()
            .await
            .map(|n| n.pairing_manager.clone())
            .ok_or_else(AppError::node_not_running)
    }

    /// Convenience: get `DeviceManager` if P2P is running.
    pub async fn devices(&self) -> AppResult<Arc<crate::device::DeviceManager>> {
        self.net()
            .await
            .map(|n| n.device_manager.clone())
            .ok_or_else(AppError::node_not_running)
    }

    /// Convenience: get `AppSyncCoordinator` if P2P is running.
    pub async fn sync_coordinator_or_err(&self) -> AppResult<Arc<AppSyncCoordinator>> {
        self.sync_coordinator()
            .await
            .ok_or_else(AppError::node_not_running)
    }

    /// Create a [`WorkspaceSync`], subscribe to GossipSub, and install it
    /// on the workspace. If `publish_opened` is true, also broadcast a
    /// `WorkspaceOpened` ctrl message to connected peers.
    async fn install_workspace_sync(
        self: &Arc<Self>,
        ws: &Arc<WorkspaceCore>,
        client: &AppNetClient,
        publish_opened: bool,
    ) {
        let ws_sync = Arc::new(WorkspaceSync::new(ws.info.id, self.clone(), client.clone()));
        ws_sync.subscribe().await;
        if publish_opened {
            ws_sync.publish_workspace_opened().await;
        }
        ws.set_sync(Some(ws_sync)).await;
    }
}
