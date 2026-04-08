use std::sync::Arc;

use dashmap::DashMap;
use swarm_p2p_core::libp2p::PeerId;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::task::AbortHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use uuid::Uuid;

use crate::network::online::AppNetClient;
use crate::protocol::{AppResponse, SyncRequest, SyncResponse};
use crate::workspace::state::{DbState, WorkspaceState};

use super::pending_buffer::PendingUpdateBuffer;
use super::{asset_sync, doc_sync, full_sync};

/// Manages full-sync sessions and incremental (GossipSub) sync.
///
/// Integrated into [`NetManager`] alongside DeviceManager and PairingManager.
/// Accesses `DbState`, `WorkspaceState`, `YDocManager`, `IdentityState` via
/// `app.state::<T>()` on demand to avoid circular references.
pub struct SyncManager {
    pub(crate) app: AppHandle,
    pub(crate) client: AppNetClient,
    /// Prevents duplicate full-sync for the same (peer, workspace).
    active_syncs: DashMap<(PeerId, Uuid), CancellationToken>,
    /// Debounce: pending asset-check tasks per doc_uuid (abort old on new update).
    asset_check_handles: DashMap<Uuid, AbortHandle>,
    /// Buffer for GossipSub updates targeting closed documents (3s debounce).
    pending_buffer: PendingUpdateBuffer,
    /// Abort handle for the pending buffer flush task (used on shutdown).
    #[allow(dead_code)]
    pending_flush_handle: std::sync::Mutex<Option<AbortHandle>>,
    /// Signal to trigger an urgent SV compensation round (e.g. after publish failure).
    sv_urgent: Arc<Notify>,
}

impl SyncManager {
    pub fn new(app: AppHandle, client: AppNetClient) -> Self {
        let pending_buffer = PendingUpdateBuffer::new();
        let flush_handle = pending_buffer.spawn_flush_task(app.clone());
        Self {
            app,
            client,
            active_syncs: DashMap::new(),
            asset_check_handles: DashMap::new(),
            pending_buffer,
            pending_flush_handle: std::sync::Mutex::new(Some(flush_handle)),
            sv_urgent: Arc::new(Notify::new()),
        }
    }

    /// Called when a paired peer comes online. Spawns full-sync for each
    /// locally registered workspace.
    pub async fn on_paired_peer_connected(self: &Arc<Self>, peer_id: PeerId) {
        info!("Paired peer connected: {peer_id}, will trigger full sync");

        let ws_state = self.app.state::<WorkspaceState>();
        let workspaces = ws_state.list_all().await;

        for ws_info in &workspaces {
            self.sync_workspace(peer_id, ws_info.id).await;
        }
    }

    /// Handle an incoming ctrl-topic message from a peer.
    pub async fn handle_ctrl_message(self: &Arc<Self>, source: PeerId, msg: super::CtrlMessage) {
        match msg {
            super::CtrlMessage::WorkspaceOpened { uuid } => {
                let ws_state = self.app.state::<WorkspaceState>();
                if ws_state.get(&uuid).await.is_some() {
                    info!("Peer {source} opened workspace {uuid} which is also open locally — syncing");
                    self.sync_workspace(source, uuid).await;
                }
            }
        }
    }

    /// Subscribe (if window-bound) + full sync (always) for a workspace.
    ///
    /// Sync-only workspaces (no window) only participate in full sync,
    /// skipping GossipSub subscription to avoid unnecessary real-time I/O.
    async fn sync_workspace(self: &Arc<Self>, peer_id: PeerId, workspace_uuid: Uuid) {
        let ws_state = self.app.state::<WorkspaceState>();
        if ws_state.is_bound(&workspace_uuid).await {
            self.subscribe_workspace(workspace_uuid).await;
        }
        self.spawn_full_sync(peer_id, workspace_uuid).await;
    }

    /// Spawn a full sync task if not already running for this (peer, workspace).
    pub async fn spawn_full_sync(self: &Arc<Self>, peer_id: PeerId, workspace_uuid: Uuid) {
        let key = (peer_id, workspace_uuid);

        // Atomic check-and-insert to prevent duplicate syncs
        let cancel = {
            use dashmap::mapref::entry::Entry;
            match self.active_syncs.entry(key) {
                Entry::Occupied(_) => {
                    info!("Full sync already active for {peer_id} / {workspace_uuid}, skipping");
                    return;
                }
                Entry::Vacant(e) => {
                    let cancel = CancellationToken::new();
                    e.insert(cancel.clone());
                    cancel
                }
            }
        };

        let this = Arc::clone(self);
        let app = self.app.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let result = full_sync::full_sync(app, client, peer_id, workspace_uuid, cancel).await;

            if let Err(e) = result {
                warn!("Full sync failed for {peer_id} / {workspace_uuid}: {e}");
            }

            // Cleanup active_syncs entry
            this.active_syncs.remove(&key);
        });
    }

    /// Handle an inbound sync request from a remote peer.
    pub async fn handle_inbound_request(
        &self,
        peer_id: PeerId,
        pending_id: u64,
        request: SyncRequest,
    ) {
        match request {
            SyncRequest::DocList { workspace_uuid } => {
                info!("Inbound DocList request from {peer_id} for workspace {workspace_uuid}");
                match full_sync::build_local_doc_list(&self.app, workspace_uuid).await {
                    Ok(docs) => {
                        let resp = AppResponse::Sync(SyncResponse::DocList { docs });
                        if let Err(e) = self.client.send_response(pending_id, resp).await {
                            warn!("Failed to send DocList response to {peer_id}: {e}");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to build DocList for workspace {workspace_uuid}: {e}");
                    }
                }
            }
            SyncRequest::StateVector { doc_id, sv } => {
                info!("Inbound StateVector request from {peer_id} for doc {doc_id}");
                match self.find_doc_context(doc_id).await.map(|(ws, _)| ws) {
                    Some(ws_uuid) => {
                        if let Err(e) = doc_sync::handle_state_vector_request(
                            &self.app,
                            &self.client,
                            pending_id,
                            doc_id,
                            &sv,
                            ws_uuid,
                        )
                        .await
                        {
                            warn!("SV response failed for {doc_id}: {e}");
                        }
                    }
                    None => warn!("No workspace found for doc {doc_id}"),
                }
            }
            SyncRequest::FullSync { doc_id } => {
                info!("Inbound FullSync request from {peer_id} for doc {doc_id}");
                match self.find_doc_context(doc_id).await.map(|(ws, _)| ws) {
                    Some(ws_uuid) => {
                        if let Err(e) = doc_sync::handle_full_sync_request(
                            &self.app,
                            &self.client,
                            pending_id,
                            doc_id,
                            ws_uuid,
                        )
                        .await
                        {
                            warn!("FullSync response failed for {doc_id}: {e}");
                        }
                    }
                    None => warn!("No workspace found for doc {doc_id}"),
                }
            }
            SyncRequest::AssetManifest { doc_id } => {
                info!("Inbound AssetManifest request from {peer_id} for doc {doc_id}");
                if let Some((ws_uuid, rel_path)) = self.find_doc_context(doc_id).await {
                    if let Err(e) = asset_sync::handle_asset_manifest_request(
                        &self.app,
                        &self.client,
                        pending_id,
                        doc_id,
                        ws_uuid,
                        &rel_path,
                    )
                    .await
                    {
                        warn!("AssetManifest response failed for {doc_id}: {e}");
                    }
                }
            }
            SyncRequest::AssetChunk {
                doc_id,
                name,
                chunk_index,
            } => {
                if let Some((ws_uuid, rel_path)) = self.find_doc_context(doc_id).await {
                    if let Err(e) = asset_sync::handle_asset_chunk_request(
                        &self.app,
                        &self.client,
                        pending_id,
                        doc_id,
                        &name,
                        chunk_index,
                        ws_uuid,
                        &rel_path,
                    )
                    .await
                    {
                        warn!("AssetChunk response failed for {name}#{chunk_index}: {e}");
                    }
                }
            }
        }
    }

    /// Handle an incoming workspace-level GossipSub message.
    ///
    /// Routes to open doc (real-time apply) or pending buffer (closed doc).
    pub async fn handle_ws_gossip_update(
        self: &Arc<Self>,
        source: Option<PeerId>,
        workspace_uuid: Uuid,
        doc_uuid: Uuid,
        data: Vec<u8>,
    ) {
        // 1. Try the open-doc path first
        let ydoc_mgr = self.app.state::<crate::yjs::manager::YDocManager>();
        match ydoc_mgr
            .apply_sync_update(&self.app, &doc_uuid, &data)
            .await
        {
            Some(Ok(())) => {
                // Successfully applied to open doc — do asset check
                self.schedule_asset_check(source, doc_uuid);
            }
            Some(Err(e)) => {
                warn!("Failed to apply ws gossip update for {doc_uuid}: {e}");
            }
            None => {
                // Document not open — buffer for later flush
                // If cap exceeded, push returns overflow for immediate apply
                if let Some((ws, _source_peer, overflow)) = self
                    .pending_buffer
                    .push(workspace_uuid, doc_uuid, data, source)
                    .await
                {
                    for update in &overflow {
                        if let Err(e) =
                            doc_sync::apply_remote_update(&self.app, ws, doc_uuid, update).await
                        {
                            warn!("Overflow flush failed for doc {doc_uuid}: {e}");
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Schedule a debounced asset check for a document after receiving a remote update.
    fn schedule_asset_check(self: &Arc<Self>, source: Option<PeerId>, doc_uuid: Uuid) {
        let Some(peer) = source else { return };

        // Abort any previous pending asset check for this doc
        if let Some((_, old_handle)) = self.asset_check_handles.remove(&doc_uuid) {
            old_handle.abort();
        }

        let this = Arc::clone(self);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            this.asset_check_handles.remove(&doc_uuid);
            if let Some((ws_uuid, rel_path)) = this.find_doc_context(doc_uuid).await {
                if let Err(e) = asset_sync::sync_doc_assets(
                    &this.app,
                    &this.client,
                    peer,
                    ws_uuid,
                    doc_uuid,
                    &rel_path,
                )
                .await
                {
                    warn!("Incremental asset sync failed for {doc_uuid}: {e}");
                }
            }
        });
        self.asset_check_handles
            .insert(doc_uuid, handle.abort_handle());
    }

    /// Find workspace UUID and rel_path for a document by searching all open workspace DBs.
    async fn find_doc_context(&self, doc_id: Uuid) -> Option<(Uuid, String)> {
        use entity::workspace::documents;
        use sea_orm::EntityTrait;

        let db_state = self.app.state::<DbState>();
        for ws_uuid in db_state.list_workspace_uuids().await {
            if let Ok(guard) = db_state.workspace_db(&ws_uuid).await {
                if let Ok(Some(doc)) = documents::Entity::find_by_id(doc_id)
                    .one(guard.conn())
                    .await
                {
                    return Some((ws_uuid, doc.rel_path));
                }
            }
        }
        None
    }

    /// Start periodic SV compensation for open documents.
    ///
    /// Runs every 5 minutes, or 30 s after a GossipSub publish failure (urgent signal).
    /// Stops when the cancellation token is triggered (node shutdown).
    pub fn start_sv_compensation(self: &Arc<Self>, cancel: CancellationToken) {
        let this = Arc::clone(self);
        let urgent = Arc::clone(&self.sv_urgent);

        tokio::spawn(async move {
            let period = std::time::Duration::from_secs(300); // 5 min
            let urgent_debounce = std::time::Duration::from_secs(30);
            let mut interval = tokio::time::interval(period);

            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        info!("SV compensation task stopped");
                        return;
                    }
                    _ = interval.tick() => {
                        // Normal 5-min periodic tick
                    }
                    _ = urgent.notified() => {
                        // Publish failure — debounce 30s then run
                        tokio::time::sleep(urgent_debounce).await;
                        // Reset the 5-min timer so we don't run again too soon
                        interval.reset();
                    }
                }

                this.run_sv_compensation().await;
            }
        });
    }

    /// Execute one round of SV compensation for all open documents.
    async fn run_sv_compensation(self: &Arc<Self>) {
        let ydoc_mgr = self.app.state::<crate::yjs::manager::YDocManager>();
        let open_docs = ydoc_mgr.list_open_doc_uuids();

        if open_docs.is_empty() {
            return;
        }

        let device_mgr_result = {
            if let Some(net_state) = self.app.try_state::<crate::network::NetManagerState>() {
                net_state.devices().await.ok()
            } else {
                None
            }
        };

        let Some(device_mgr) = device_mgr_result else {
            return;
        };

        let connected_peers = device_mgr.connected_paired_peers();
        if connected_peers.is_empty() {
            return;
        }

        for doc_uuid in &open_docs {
            for peer_id in &connected_peers {
                if let Some((ws_uuid, _)) = self.find_doc_context(*doc_uuid).await {
                    if let Err(e) = doc_sync::sync_via_state_vector(
                        &self.app,
                        &self.client,
                        *peer_id,
                        ws_uuid,
                        *doc_uuid,
                    )
                    .await
                    {
                        tracing::trace!(
                            "SV compensation failed for doc {doc_uuid} with {peer_id}: {e}"
                        );
                    }
                }
            }
        }
    }

    /// Subscribe to a workspace-level GossipSub topic.
    pub async fn subscribe_workspace(&self, workspace_uuid: Uuid) {
        let topic = super::ws_topic(&workspace_uuid);
        match self.client.subscribe(&topic).await {
            Ok(_) => info!("Subscribed to workspace GossipSub topic: {topic}"),
            Err(e) => warn!("Failed to subscribe to {topic}: {e}"),
        }
    }

    /// Broadcast a WorkspaceOpened ctrl message so connected peers can subscribe + sync.
    pub async fn publish_workspace_opened(&self, workspace_uuid: Uuid) {
        let payload = super::encode_ctrl_message(&super::CtrlMessage::WorkspaceOpened {
            uuid: workspace_uuid,
        });
        if let Err(e) = self.client.publish(super::CTRL_TOPIC, payload).await {
            warn!("Failed to publish WorkspaceOpened for {workspace_uuid}: {e}");
        }
    }

    /// Unsubscribe from a workspace-level GossipSub topic (called on workspace close).
    pub async fn unsubscribe_workspace(&self, workspace_uuid: Uuid) {
        let topic = super::ws_topic(&workspace_uuid);
        match self.client.unsubscribe(&topic).await {
            Ok(_) => info!("Unsubscribed from workspace GossipSub topic: {topic}"),
            Err(e) => warn!("Failed to unsubscribe from {topic}: {e}"),
        }
    }

    /// Publish a document update to the workspace GossipSub topic.
    pub async fn publish_doc_update(&self, workspace_uuid: Uuid, doc_uuid: Uuid, update: Vec<u8>) {
        let topic = super::ws_topic(&workspace_uuid);
        let payload = super::encode_ws_gossip(&doc_uuid, &update);
        if let Err(e) = self.client.publish(&topic, payload).await {
            // NoPeersSubscribedToTopic is expected when no other peer has the workspace open
            tracing::debug!("Failed to publish doc update to {topic}: {e}");
            // Schedule urgent SV compensation to ensure data consistency
            self.sv_urgent.notify_one();
        }
    }
}

/// Parse a GossipSub topic string to extract the doc UUID (legacy per-doc topic).
/// Topic format: `swarmnote/doc/{uuid}`
pub fn parse_sync_topic(topic: &str) -> Option<Uuid> {
    topic
        .strip_prefix("swarmnote/doc/")
        .and_then(|s| Uuid::parse_str(s).ok())
}

/// Parse a workspace-level GossipSub topic to extract the workspace UUID.
/// Topic format: `swarmnote/ws/{uuid}`
pub fn parse_ws_topic(topic: &str) -> Option<Uuid> {
    topic
        .strip_prefix("swarmnote/ws/")
        .and_then(|s| Uuid::parse_str(s).ok())
}
