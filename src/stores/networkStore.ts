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

interface NetworkState {
  isNodeRunning: boolean;
  connectedPeers: PeerInfo[];
  natStatus: string | null;
}

interface NetworkActions {
  startNode: () => Promise<void>;
  stopNode: () => Promise<void>;
  refreshPeers: () => Promise<void>;
}

// ── Store ──

export const useNetworkStore = create<NetworkState & NetworkActions>()((set) => ({
  isNodeRunning: false,
  connectedPeers: [],
  natStatus: null,

  startNode: async () => {
    await invoke("start_p2p_node");
    set({ isNodeRunning: true });
  },

  stopNode: async () => {
    await invoke("stop_p2p_node");
    set({ isNodeRunning: false, connectedPeers: [], natStatus: null });
  },

  refreshPeers: async () => {
    const peers = await invoke<PeerInfo[]>("get_connected_peers");
    set({ connectedPeers: peers });
  },
}));

// ── Tauri Event Listeners ──

let unlisteners: UnlistenFn[] = [];

export async function setupNetworkListeners() {
  // 避免重复监听
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

  unlisteners = [u1, u2, u3];
}

export async function cleanupNetworkListeners() {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];
}
