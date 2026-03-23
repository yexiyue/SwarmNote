import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceModel {
  id: string;
  name: string;
  created_by: string;
  created_at: number;
  updated_at: number;
}

export interface InitWorkspaceInput {
  path: string;
  name: string;
}

export async function initWorkspace(input: InitWorkspaceInput): Promise<WorkspaceModel> {
  return invoke<WorkspaceModel>("db_init_workspace", { input });
}
