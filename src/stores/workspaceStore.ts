import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { create } from "zustand";

import {
  getWorkspaceInfo,
  openWorkspace as openWorkspaceCmd,
  type WorkspaceInfo,
} from "@/commands/workspace";
import { useEditorStore } from "@/stores/editorStore";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { useNetworkStore } from "@/stores/networkStore";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface WorkspaceState {
  workspace: WorkspaceInfo | null;
  isLoading: boolean;
  error: string | null;
}

interface WorkspaceActions {
  /** Check if backend already has a workspace loaded (auto-restore). */
  initFromBackend: () => Promise<void>;
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

/** 工作区打开后，根据偏好自动启动 P2P 节点 */
function maybeAutoStartP2P() {
  const { autoStartP2P } = usePreferencesStore.getState();
  const { status, userManuallyStopped, startNode } = useNetworkStore.getState();
  if (autoStartP2P && status === "stopped" && !userManuallyStopped) {
    startNode();
  }
}

export const useWorkspaceStore = create<WorkspaceState & WorkspaceActions>()((set) => ({
  workspace: null,
  isLoading: false,
  error: null,

  initFromBackend: async () => {
    set({ isLoading: true, error: null });
    try {
      // 先注册事件监听，再调用 get_workspace_info，避免新建窗口场景下事件丢失
      let unlistenFn: (() => void) | null = null;
      const unlistenPromise = listen<WorkspaceInfo>("workspace:ready", (event) => {
        set({ workspace: event.payload });
        maybeAutoStartP2P();
        unlistenFn?.();
      });
      const info = await getWorkspaceInfo();
      unlistenFn = await unlistenPromise;
      if (info) {
        // auto-restore 或新建窗口（Rust 已在建窗口前完成绑定）场景：直接拿到数据
        set({ workspace: info });
        maybeAutoStartP2P();
        unlistenFn();
      }
      // info 为 null 时（全新启动无历史工作区）：保留监听，等待 workspace:ready 事件
      // 若用户通过 WorkspacePicker fullscreen 选择，openWorkspace() 会直接 set，监听自然不触发
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ isLoading: false });
    }
  },

  openWorkspace: async (path) => {
    set({ isLoading: true, error: null });
    clearDependentStores();
    try {
      const workspace = await openWorkspaceCmd(path);
      set({ workspace });
      if (workspace) maybeAutoStartP2P();
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
