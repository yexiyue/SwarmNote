import { create } from "zustand";
import { upsertDocument } from "@/commands/document";

interface EditorState {
  currentDocId: string | null;
  title: string;
  relPath: string;
  content: string;
  isDirty: boolean;
  lastSavedAt: Date | null;
  charCount: number;
}

interface EditorActions {
  loadDocument: (id: string, title: string, relPath: string) => void;
  saveDocument: (workspaceId: string) => Promise<void>;
  updateContent: (content: string) => void;
  updateTitle: (title: string) => void;
  clear: () => void;
}

const initialState: EditorState = {
  currentDocId: null,
  title: "",
  relPath: "",
  content: "",
  isDirty: false,
  lastSavedAt: null,
  charCount: 0,
};

export const useEditorStore = create<EditorState & EditorActions>()((set, get) => ({
  ...initialState,

  loadDocument: (id, title, relPath) => {
    set({ ...initialState, currentDocId: id, title, relPath });
  },

  saveDocument: async (workspaceId) => {
    const { currentDocId, title, relPath } = get();
    if (!currentDocId) return;

    await upsertDocument({
      id: currentDocId,
      workspace_id: workspaceId,
      title,
      rel_path: relPath,
    });
    set({ isDirty: false, lastSavedAt: new Date() });
  },

  updateContent: (content) => set({ content, isDirty: true, charCount: content.length }),

  updateTitle: (title) => set({ title, isDirty: true }),

  clear: () => set(initialState),
}));
