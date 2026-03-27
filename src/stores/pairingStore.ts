import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import type { PairedDeviceInfo, PeerInfo } from "@/commands/pairing";
import { getNearbyDevices, getPairedDevices } from "@/commands/pairing";

interface PairingState {
  pairedDevices: PairedDeviceInfo[];
  nearbyDevices: PeerInfo[];
  isLoading: boolean;
}

interface PairingActions {
  loadPairedDevices(): Promise<void>;
  loadNearbyDevices(): Promise<void>;
  refresh(): Promise<void>;
}

export const usePairingStore = create<PairingState & PairingActions>()((set, get) => ({
  pairedDevices: [],
  nearbyDevices: [],
  isLoading: false,

  async loadPairedDevices() {
    try {
      const devices = await getPairedDevices();
      set({ pairedDevices: devices });
    } catch (e) {
      console.error("Failed to load paired devices:", e);
    }
  },

  async loadNearbyDevices() {
    try {
      const devices = await getNearbyDevices();
      set({ nearbyDevices: devices });
    } catch (e) {
      console.error("Failed to load nearby devices:", e);
    }
  },

  async refresh() {
    set({ isLoading: true });
    await Promise.all([get().loadPairedDevices(), get().loadNearbyDevices()]);
    set({ isLoading: false });
  },
}));

// Tauri event listeners (module-level, auto-runs once)
let listenersSetup = false;

export function setupPairingListeners() {
  if (listenersSetup) return;
  listenersSetup = true;

  listen("paired-device-added", () => {
    usePairingStore.getState().loadPairedDevices();
  });

  listen("paired-device-removed", () => {
    usePairingStore.getState().loadPairedDevices();
  });

  listen("nearby-devices-changed", () => {
    usePairingStore.getState().loadNearbyDevices();
  });

  listen("peer-connected", () => {
    usePairingStore.getState().loadNearbyDevices();
  });

  listen("peer-disconnected", () => {
    usePairingStore.getState().loadNearbyDevices();
  });
}
