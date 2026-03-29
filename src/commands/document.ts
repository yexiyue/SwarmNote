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
  file_hash?: string | null;
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

export async function saveMedia(
  relPath: string,
  fileName: string,
  data: number[],
): Promise<string> {
  return invoke<string>("save_media", { relPath, fileName, data });
}

// ── Y.Doc commands ──

export interface OpenDocResult {
  doc_uuid: string;
  yjs_state: number[];
}

export async function openYDoc(
  relPath: string,
  workspaceId: string,
  assetUrlPrefix: string,
): Promise<OpenDocResult> {
  return invoke<OpenDocResult>("open_ydoc", { relPath, workspaceId, assetUrlPrefix });
}

export async function applyYDocUpdate(docUuid: string, update: number[]): Promise<void> {
  return invoke("apply_ydoc_update", { docUuid, update });
}

export async function closeYDoc(docUuid: string): Promise<void> {
  return invoke("close_ydoc", { docUuid });
}

export async function renameYDoc(docUuid: string, newRelPath: string): Promise<void> {
  return invoke("rename_ydoc", { docUuid, newRelPath });
}
