import { create } from "zustand";
import { initWorkspace, type WorkspaceModel } from "@/commands/workspace";

interface WorkspaceState {
  workspace: WorkspaceModel | null;
  isLoading: boolean;
}

interface WorkspaceActions {
  openWorkspace: (path: string, name: string) => Promise<void>;
  clearWorkspace: () => void;
}

export const useWorkspaceStore = create<WorkspaceState & WorkspaceActions>()((set) => ({
  workspace: null,
  isLoading: false,

  openWorkspace: async (path, name) => {
    set({ isLoading: true });
    try {
      const workspace = await initWorkspace({ path, name });
      set({ workspace });
    } finally {
      set({ isLoading: false });
    }
  },

  clearWorkspace: () => set({ workspace: null }),
}));
