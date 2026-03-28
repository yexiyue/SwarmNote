import { invoke } from "@tauri-apps/api/core";

export interface DocumentModel {
  id: string;
  workspace_id: string;
  folder_id: string | null;
  title: string;
  rel_path: string;
  file_hash: number[] | null;
  yjs_state: number[] | null;
  state_vector: number[] | null;
  created_by: string;
  created_at: number;
  updated_at: number;
}

export interface UpsertDocumentInput {
  id?: string;
  workspace_id: string;
  folder_id?: string | null;
  title: string;
  rel_path: string;
}

export async function getDocuments(workspaceId: string): Promise<DocumentModel[]> {
  return invoke<DocumentModel[]>("db_get_documents", { workspaceId });
}

export async function upsertDocument(input: UpsertDocumentInput): Promise<DocumentModel> {
  return invoke<DocumentModel>("db_upsert_document", { input });
}

export async function deleteDocument(id: string): Promise<void> {
  return invoke("db_delete_document", { id });
}

// TODO(#6): Replace with Tauri invoke: invoke("load_document", { relPath })
export async function loadDocumentContent(relPath: string): Promise<string> {
  return localStorage.getItem(`swarmnote:doc:${relPath}`) ?? "";
}

// TODO(#6): Replace with Tauri invoke: invoke("save_document", { relPath, markdown })
export async function saveDocumentContent(relPath: string, markdown: string): Promise<void> {
  localStorage.setItem(`swarmnote:doc:${relPath}`, markdown);
}
