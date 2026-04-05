import { invoke } from "@tauri-apps/api/core";

export interface FileTreeNode {
  id: string;
  name: string;
  children?: FileTreeNode[];
}

export function scanWorkspaceTree(): Promise<FileTreeNode[]> {
  return invoke<FileTreeNode[]>("scan_workspace_tree");
}

export function fsCreateFile(parentRel: string, name: string): Promise<string> {
  return invoke<string>("fs_create_file", { parentRel, name });
}

export function fsCreateDir(parentRel: string, name: string): Promise<string> {
  return invoke<string>("fs_create_dir", { parentRel, name });
}

export function fsDeleteFile(relPath: string): Promise<void> {
  return invoke<void>("fs_delete_file", { relPath });
}

export function fsDeleteDir(relPath: string): Promise<void> {
  return invoke<void>("fs_delete_dir", { relPath });
}

export function fsRename(relPath: string, newName: string): Promise<string> {
  return invoke<string>("fs_rename", { relPath, newName });
}
