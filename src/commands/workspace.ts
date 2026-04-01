import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceInfo {
  id: string;
  name: string;
  path: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface RecentWorkspace {
  path: string;
  name: string;
  last_opened_at: string;
}

export async function openWorkspace(path: string): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>("open_workspace", { path });
}

export async function getWorkspaceInfo(): Promise<WorkspaceInfo | null> {
  return invoke<WorkspaceInfo | null>("get_workspace_info");
}

export async function getRecentWorkspaces(): Promise<RecentWorkspace[]> {
  return invoke<RecentWorkspace[]>("get_recent_workspaces");
}

export async function openWorkspaceWindow(path: string): Promise<void> {
  return invoke("open_workspace_window", { path });
}

export async function openSettingsWindow(route?: string): Promise<void> {
  return invoke("open_settings_window", { route });
}
