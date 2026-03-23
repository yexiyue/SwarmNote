import { invoke } from "@tauri-apps/api/core";

export interface FolderModel {
  id: string;
  workspace_id: string;
  parent_folder_id: string | null;
  name: string;
  rel_path: string;
  created_by: string;
  created_at: number;
  updated_at: number;
}

export interface CreateFolderInput {
  workspace_id: string;
  parent_folder_id?: string | null;
  name: string;
  rel_path: string;
}

export async function getFolders(workspaceId: string): Promise<FolderModel[]> {
  return invoke<FolderModel[]>("db_get_folders", { workspaceId });
}

export async function createFolder(input: CreateFolderInput): Promise<FolderModel> {
  return invoke<FolderModel>("db_create_folder", { input });
}

export async function deleteFolder(id: string): Promise<void> {
  return invoke("db_delete_folder", { id });
}
