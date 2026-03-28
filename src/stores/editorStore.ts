import { create } from "zustand";
import { loadDocumentContent, saveDocumentContent, upsertDocument } from "@/commands/document";
import { assetUrlToRelativePath, relativePathToAssetUrl } from "@/lib/markdownMedia";
import { useWorkspaceStore } from "@/stores/workspaceStore";

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
  /** Update relPath and currentDocId after file rename. */
  updateRelPath: (newRelPath: string, newTitle: string) => void;
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
    const workspace = useWorkspaceStore.getState().workspace;
    const raw = await loadDocumentContent(relPath);
    const markdown = workspace ? relativePathToAssetUrl(raw, workspace.path) : raw;
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
    const { currentDocId, relPath, markdown, title } = get();
    if (!currentDocId || !get().isDirty) return;

    const workspace = useWorkspaceStore.getState().workspace;
    const contentToSave = workspace ? assetUrlToRelativePath(markdown, workspace.path) : markdown;
    const { file_hash } = await saveDocumentContent(relPath, contentToSave);
    set({ isDirty: false, lastSavedAt: new Date() });

    if (workspace) {
      await upsertDocument({
        id: currentDocId,
        workspace_id: workspace.id,
        title,
        rel_path: relPath,
        file_hash,
      });
    }
  },

  updateContent: (markdown) => set({ markdown, isDirty: true, charCount: markdown.length }),

  updateTitle: (title) => set({ title }),

  updateRelPath: (newRelPath, newTitle) =>
    set({ currentDocId: newRelPath, relPath: newRelPath, title: newTitle }),

  clear: () => set(initialState),
}));
