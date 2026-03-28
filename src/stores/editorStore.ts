import { create } from "zustand";
import { loadDocumentContent, saveDocumentContent } from "@/commands/document";

interface EditorState {
  currentDocId: string | null;
  title: string;
  relPath: string;
  markdown: string;
  isDirty: boolean;
  lastSavedAt: Date | null;
  charCount: number;
}

interface EditorActions {
  loadDocument: (id: string, title: string, relPath: string) => Promise<void>;
  saveContent: () => Promise<void>;
  updateContent: (markdown: string) => void;
  updateTitle: (title: string) => void;
  clear: () => void;
}

const initialState: EditorState = {
  currentDocId: null,
  title: "",
  relPath: "",
  markdown: "",
  isDirty: false,
  lastSavedAt: null,
  charCount: 0,
};

export const useEditorStore = create<EditorState & EditorActions>()((set, get) => ({
  ...initialState,

  loadDocument: async (id, title, relPath) => {
    const markdown = await loadDocumentContent(relPath);
    set({
      ...initialState,
      currentDocId: id,
      title,
      relPath,
      markdown,
      charCount: markdown.length,
    });
  },

  saveContent: async () => {
    const { currentDocId, relPath, markdown } = get();
    if (!currentDocId) return;

    await saveDocumentContent(relPath, markdown);
    set({ isDirty: false, lastSavedAt: new Date() });
  },

  updateContent: (markdown) => set({ markdown, isDirty: true, charCount: markdown.length }),

  updateTitle: (title) => set({ title }),

  clear: () => set(initialState),
}));
