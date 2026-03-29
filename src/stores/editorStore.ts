import { create } from "zustand";

interface EditorState {
  currentDocId: string | null;
  /** Stable UUID from the database (set after open_ydoc returns). */
  docUuid: string | null;
  title: string;
  relPath: string;
  isDirty: boolean;
  lastSavedAt: Date | null;
  charCount: number;
}

interface EditorActions {
  loadDocument: (id: string, title: string, relPath: string) => void;
  setDocUuid: (uuid: string) => void;
  updateTitle: (title: string) => void;
  updateRelPath: (newRelPath: string, newTitle: string) => void;
  markDirty: () => void;
  markFlushed: (lastSavedAt: Date) => void;
  setCharCount: (count: number) => void;
  clear: () => void;
}

const initialState: EditorState = {
  currentDocId: null,
  docUuid: null,
  title: "",
  relPath: "",
  isDirty: false,
  lastSavedAt: null,
  charCount: 0,
};

export const useEditorStore = create<EditorState & EditorActions>()((set) => ({
  ...initialState,

  loadDocument: (id, title, relPath) => {
    set({
      ...initialState,
      currentDocId: id,
      title,
      relPath,
    });
  },

  setDocUuid: (uuid) => set({ docUuid: uuid }),

  updateTitle: (title) => set({ title }),

  updateRelPath: (newRelPath, newTitle) =>
    set({ currentDocId: newRelPath, relPath: newRelPath, title: newTitle }),

  markDirty: () => set((state) => (state.isDirty ? state : { isDirty: true })),

  markFlushed: (lastSavedAt) => set({ isDirty: false, lastSavedAt }),

  setCharCount: (count) => set({ charCount: count }),

  clear: () => set(initialState),
}));
