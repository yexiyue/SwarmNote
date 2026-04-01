import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

// ── Types ──

export interface PeerInfo {
  peer_id: string;
  hostname: string;
  os: string;
  platform: string;
  arch: string;
  is_connected: boolean;
  rtt_ms: number | null;
  connection_type: string | null;
}

interface NetworkStatus {
  natStatus: string | null;
  publicAddr: string | null;
}

/** Matches Rust `network::NodeStatus` — single source of truth */
export type NodeStatus = "stopped" | "running" | "error";

/** Rust serializes NodeStatus as `{ kind: "stopped" }` or `{ kind: "error", message: "..." }` */
interface NodeStatusPayload {
  kind: NodeStatus;
  message?: string;
}

interface NetworkState {
  status: NodeStatus;
  error: string | null;
  connectedPeers: PeerInfo[];
  natStatus: string | null;
  userManuallyStopped: boolean;
  /** True while an async start/stop operation is in-flight */
  loading: boolean;
}

interface NetworkActions {
  startNode: () => Promise<void>;
  stopNode: (manual?: boolean) => Promise<void>;
  refreshPeers: () => Promise<void>;
}

// ── Store ──

export const useNetworkStore = create<NetworkState & NetworkActions>()((set, get) => ({
  status: "stopped",
  error: null,
  connectedPeers: [],
  natStatus: null,
  userManuallyStopped: false,
  loading: false,

  startNode: async () => {
    const { status, loading } = get();
    if (status === "running" || loading) return;

    set({ loading: true, error: null, userManuallyStopped: false });
    try {
      await invoke("start_p2p_node");
      set({ status: "running", error: null });
    } catch (e) {
      set({ status: "error", error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  stopNode: async (manual = false) => {
    set({ loading: true });
    try {
      await invoke("stop_p2p_node");
      set({ status: "stopped", connectedPeers: [], natStatus: null });
      if (manual) {
        set({ userManuallyStopped: true });
      }
    } catch (e) {
      set({ status: "error", error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  refreshPeers: async () => {
    const peers = await invoke<PeerInfo[]>("get_connected_peers");
    set({ connectedPeers: peers });
  },
}));

// ── Tauri Event Listeners ──

let unlisteners: UnlistenFn[] = [];

export async function setupNetworkListeners() {
  await cleanupNetworkListeners();

  const u1 = await listen<PeerInfo>("peer-connected", (event) => {
    const peer = event.payload;
    useNetworkStore.setState((state) => {
      const existing = state.connectedPeers.findIndex((p) => p.peer_id === peer.peer_id);
      const peers =
        existing >= 0
          ? state.connectedPeers.map((p, i) => (i === existing ? peer : p))
          : [...state.connectedPeers, peer];
      return { connectedPeers: peers };
    });
  });

  const u2 = await listen<string>("peer-disconnected", (event) => {
    const peerId = event.payload;
    useNetworkStore.setState((state) => ({
      connectedPeers: state.connectedPeers.filter((p) => p.peer_id !== peerId),
    }));
  });

  const u3 = await listen<NetworkStatus>("network-status-changed", (event) => {
    useNetworkStore.setState({ natStatus: event.payload.natStatus });
  });

  // Events from backend — used to sync other windows
  const u4 = await listen("node-started", () => {
    useNetworkStore.setState({ status: "running", error: null, loading: false });
  });

  const u5 = await listen("node-stopped", () => {
    useNetworkStore.setState({
      status: "stopped",
      connectedPeers: [],
      natStatus: null,
      loading: false,
    });
  });

  unlisteners = [u1, u2, u3, u4, u5];

  // Sync initial status from backend (handles page refresh / new window)
  try {
    const payload = await invoke<NodeStatusPayload>("get_network_status");
    if (payload.kind === "running") {
      useNetworkStore.setState({ status: "running", error: null });
    } else if (payload.kind === "error") {
      useNetworkStore.setState({ status: "error", error: payload.message ?? null });
    }
  } catch {
    // Backend not ready yet — listeners will catch subsequent events
  }
}

export async function cleanupNetworkListeners() {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];
}

// Auto-register listeners on module load
setupNetworkListeners();
