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
  uuid?: string;
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

export type OpenWorkspaceWindowResult =
  | { kind: "bound_to_caller"; info: WorkspaceInfo }
  | { kind: "focused_existing" }
  | { kind: "new_window" };

export interface OpenWorkspaceWindowOptions {
  /** When set, request the backend to bind the workspace to the given window
   *  label (typically the caller's own window) if that window has no workspace
   *  bound. Lets fullscreen pickers avoid spawning a second window. */
  bindToWindow?: string;
}

export async function openWorkspaceWindow(
  path: string,
  options: OpenWorkspaceWindowOptions = {},
): Promise<OpenWorkspaceWindowResult> {
  return invoke<OpenWorkspaceWindowResult>("open_workspace_window", {
    path,
    bindToWindow: options.bindToWindow ?? null,
  });
}

export async function openSettingsWindow(route?: string): Promise<void> {
  return invoke("open_settings_window", { route });
}

/** 为同步创建一个本地工作区（使用远程 UUID），不打开窗口。返回创建的完整路径。 */
export async function createWorkspaceForSync(
  uuid: string,
  name: string,
  basePath: string,
): Promise<string> {
  return invoke<string>("create_workspace_for_sync", { uuid, name, basePath });
}

/** 手动触发对指定 peer 的指定工作区的 full sync。 */
export async function triggerWorkspaceSync(workspaceUuid: string, peerId: string): Promise<void> {
  return invoke("trigger_workspace_sync", { workspaceUuid, peerId });
}
