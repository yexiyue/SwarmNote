import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { create } from "zustand";

import type { Device } from "@/commands/pairing";

// ── Types ──

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
  /** 统一设备列表（包含连接类型、配对状态等完整信息） */
  devices: Device[];
  natStatus: string | null;
  userManuallyStopped: boolean;
  /** True while an async start/stop operation is in-flight */
  loading: boolean;
}

interface NetworkActions {
  startNode: () => Promise<void>;
  stopNode: (manual?: boolean) => Promise<void>;
  refreshDevices: () => Promise<void>;
  /** 已连接的设备 */
  getConnectedDevices: () => Device[];
  /** 已配对的设备 */
  getPairedDevices: () => Device[];
  /** 附近设备（已连接但未配对） */
  getNearbyDevices: () => Device[];
}

// ── Store ──

export const useNetworkStore = create<NetworkState & NetworkActions>()((set, get) => ({
  status: "stopped",
  error: null,
  devices: [],
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
      set({ status: "stopped", devices: [], natStatus: null });
      if (manual) {
        set({ userManuallyStopped: true });
      }
    } catch (e) {
      set({ status: "error", error: String(e) });
    } finally {
      set({ loading: false });
    }
  },

  refreshDevices: async () => {
    try {
      const result = await invoke<{ devices: Device[] }>("list_devices", { filter: "all" });
      set({ devices: result.devices });
    } catch {
      // Node might not be running
    }
  },

  getConnectedDevices: () => get().devices.filter((d) => d.status === "online"),
  getPairedDevices: () => get().devices.filter((d) => d.isPaired),
  getNearbyDevices: () => get().devices.filter((d) => d.status === "online" && !d.isPaired),
}));

// ── Tauri Event Listeners ──

let unlisteners: UnlistenFn[] = [];

export async function setupNetworkListeners() {
  await cleanupNetworkListeners();

  // 统一设备列表更新（替代旧的 peer-connected / peer-disconnected）
  const u1 = await listen<Device[]>("devices-changed", (event) => {
    useNetworkStore.setState({ devices: event.payload });
  });

  const u2 = await listen<NetworkStatus>("network-status-changed", (event) => {
    useNetworkStore.setState({ natStatus: event.payload.natStatus });
  });

  // Events from backend — used to sync other windows
  const u3 = await listen("node-started", () => {
    useNetworkStore.setState({ status: "running", error: null, loading: false });
  });

  const u4 = await listen("node-stopped", () => {
    useNetworkStore.setState({
      status: "stopped",
      devices: [],
      natStatus: null,
      loading: false,
    });
  });

  unlisteners = [u1, u2, u3, u4];

  // Sync initial status from backend (handles page refresh / new window)
  try {
    const payload = await invoke<NodeStatusPayload>("get_network_status");
    if (payload.kind === "running") {
      useNetworkStore.setState({ status: "running", error: null });
      // 初始化时也拉取一次设备列表
      useNetworkStore.getState().refreshDevices();
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
