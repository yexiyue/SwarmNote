import { open } from "@tauri-apps/plugin-dialog";
import { create } from "zustand";

import {
  getRecentWorkspaces,
  getWorkspaceInfo,
  openWorkspace as openWorkspaceCmd,
  type RecentWorkspace,
  type WorkspaceInfo,
} from "@/commands/workspace";
import { useEditorStore } from "@/stores/editorStore";
import { useFileTreeStore } from "@/stores/fileTreeStore";

interface WorkspaceState {
  workspace: WorkspaceInfo | null;
  recentWorkspaces: RecentWorkspace[];
  isLoading: boolean;
  error: string | null;
}

interface WorkspaceActions {
  /** Check if backend already has a workspace loaded (auto-restore). */
  initFromBackend: () => Promise<void>;
  /** Fetch the list of recently opened workspaces. */
  fetchRecentWorkspaces: () => Promise<void>;
  /** Open a workspace by path (called after dialog or programmatically). */
  openWorkspace: (path: string) => Promise<void>;
  /** Show folder picker dialog then open the selected workspace. */
  selectAndOpenWorkspace: () => Promise<void>;
  clearWorkspace: () => void;
}

function clearDependentStores() {
  useFileTreeStore.getState().clear();
  useEditorStore.getState().clear();
}

export const useWorkspaceStore = create<WorkspaceState & WorkspaceActions>()((set) => ({
  workspace: null,
  recentWorkspaces: [],
  isLoading: false,
  error: null,

  initFromBackend: async () => {
    set({ isLoading: true, error: null });
    try {
      const info = await getWorkspaceInfo();
      set({ workspace: info });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  fetchRecentWorkspaces: async () => {
    try {
      const list = await getRecentWorkspaces();
      set({ recentWorkspaces: list });
    } catch (e) {
      console.error("Failed to fetch recent workspaces:", e);
    }
  },

  openWorkspace: async (path) => {
    set({ isLoading: true, error: null });
    clearDependentStores();
    try {
      const workspace = await openWorkspaceCmd(path);
      set({ workspace });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  selectAndOpenWorkspace: async () => {
    const selected = await open({ directory: true, title: "选择工作区目录" });
    if (!selected) return; // user cancelled

    const { openWorkspace } = useWorkspaceStore.getState();
    await openWorkspace(selected);
  },

  clearWorkspace: () => {
    clearDependentStores();
    set({ workspace: null, error: null });
  },
}));
