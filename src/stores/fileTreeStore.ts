import { create } from "zustand";
import {
  type DocumentModel,
  deleteDocument as deleteDocumentCmd,
  getDocuments,
  type UpsertDocumentInput,
  upsertDocument,
} from "@/commands/document";
import {
  type CreateFolderInput,
  createFolder as createFolderCmd,
  deleteFolder as deleteFolderCmd,
  type FolderModel,
  getFolders,
} from "@/commands/folder";

interface FileTreeState {
  documents: DocumentModel[];
  folders: FolderModel[];
  selectedFile: string | null;
  expandedFolders: Set<string>;
  isLoading: boolean;
}

interface FileTreeActions {
  loadTree: (workspaceId: string) => Promise<void>;
  selectFile: (id: string | null) => void;
  toggleFolder: (folderId: string) => void;
  createDocument: (input: UpsertDocumentInput) => Promise<DocumentModel>;
  deleteDocument: (id: string) => Promise<void>;
  createFolder: (input: CreateFolderInput) => Promise<FolderModel>;
  deleteFolder: (id: string) => Promise<void>;
  clear: () => void;
}

const initialState: FileTreeState = {
  documents: [],
  folders: [],
  selectedFile: null,
  expandedFolders: new Set<string>(),
  isLoading: false,
};

export const useFileTreeStore = create<FileTreeState & FileTreeActions>()((set, get) => ({
  ...initialState,

  loadTree: async (workspaceId) => {
    set({ isLoading: true });
    try {
      const [docs, dirs] = await Promise.all([getDocuments(workspaceId), getFolders(workspaceId)]);
      set({ documents: docs, folders: dirs });
    } finally {
      set({ isLoading: false });
    }
  },

  selectFile: (id) => set({ selectedFile: id }),

  toggleFolder: (folderId) => {
    const { expandedFolders } = get();
    const next = new Set(expandedFolders);
    if (next.has(folderId)) {
      next.delete(folderId);
    } else {
      next.add(folderId);
    }
    set({ expandedFolders: next });
  },

  createDocument: async (input) => {
    const doc = await upsertDocument(input);
    set((s) => ({
      documents: input.id
        ? s.documents.map((d) => (d.id === doc.id ? doc : d))
        : [...s.documents, doc],
    }));
    return doc;
  },

  deleteDocument: async (id) => {
    await deleteDocumentCmd(id);
    set((s) => ({
      documents: s.documents.filter((d) => d.id !== id),
      selectedFile: s.selectedFile === id ? null : s.selectedFile,
    }));
  },

  createFolder: async (input) => {
    const folder = await createFolderCmd(input);
    set((s) => ({ folders: [...s.folders, folder] }));
    return folder;
  },

  deleteFolder: async (id) => {
    await deleteFolderCmd(id);
    set((s) => ({
      folders: s.folders.filter((f) => f.id !== id),
      documents: s.documents.filter((d) => d.folder_id !== id),
      selectedFile:
        s.selectedFile && s.documents.some((d) => d.folder_id === id && d.id === s.selectedFile)
          ? null
          : s.selectedFile,
    }));
  },

  clear: () => set(initialState),
}));
