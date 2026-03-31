import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, FolderPlus, PenLine, Plus } from "lucide-react";
import { useEffect, useState } from "react";

import {
  getRecentWorkspaces,
  openWorkspaceWindow,
  type RecentWorkspace,
} from "@/commands/workspace";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { WorkspaceItem } from "@/components/workspace/WorkspaceItem";
import { useWorkspaceStore } from "@/stores/workspaceStore";

interface WorkspacePickerProps {
  mode: "fullscreen" | "dialog";
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function WorkspacePicker({ mode, open: dialogOpen, onOpenChange }: WorkspacePickerProps) {
  const [recents, setRecents] = useState<RecentWorkspace[]>([]);
  const openWorkspace = useWorkspaceStore((s) => s.openWorkspace);

  useEffect(() => {
    getRecentWorkspaces().then(setRecents);
  }, []);

  async function handleSelectWorkspace(path: string) {
    if (mode === "fullscreen") {
      await openWorkspace(path);
    } else {
      await openWorkspaceWindow(path);
      onOpenChange?.(false);
    }
  }

  async function handleOpenFolder() {
    const selected = await open({ directory: true, title: "打开工作区文件夹" });
    if (!selected) return;
    if (mode === "fullscreen") {
      await openWorkspace(selected);
    } else {
      await openWorkspaceWindow(selected);
      onOpenChange?.(false);
    }
  }

  async function handleCreateWorkspace() {
    const selected = await open({ directory: true, title: "选择新工作区目录" });
    if (!selected) return;
    if (mode === "fullscreen") {
      await openWorkspace(selected);
    } else {
      await openWorkspaceWindow(selected);
      onOpenChange?.(false);
    }
  }

  const content = (
    <div className="flex w-full max-w-lg flex-col gap-6">
      {/* Header (fullscreen only) */}
      {mode === "fullscreen" && (
        <div className="flex flex-col items-center gap-2">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-primary">
            <PenLine className="h-6 w-6 text-white" />
          </div>
          <h1 className="text-xl font-bold text-foreground">选择工作区</h1>
          <p className="text-sm text-muted-foreground">
            选择一个已有工作区，或创建新的工作区开始使用。
          </p>
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-3">
        <Button variant="outline" className="flex-1 gap-1.5" onClick={handleCreateWorkspace}>
          <Plus className="h-4 w-4" />
          创建新工作区
        </Button>
        <Button variant="outline" className="flex-1 gap-1.5" onClick={handleOpenFolder}>
          <FolderOpen className="h-4 w-4" />
          打开文件夹
        </Button>
      </div>

      {/* Recent workspaces */}
      {recents.length > 0 ? (
        <div className="flex flex-col gap-2">
          <span className="text-xs font-medium text-muted-foreground">最近打开</span>
          <div className="flex flex-col gap-1.5">
            {recents.map((ws) => (
              <WorkspaceItem key={ws.path} workspace={ws} onClick={handleSelectWorkspace} />
            ))}
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center gap-2 py-8">
          <FolderPlus className="h-10 w-10 text-muted-foreground/40" />
          <p className="text-sm text-muted-foreground">还没有工作区，创建一个开始使用吧。</p>
        </div>
      )}
    </div>
  );

  if (mode === "dialog") {
    return (
      <Dialog open={dialogOpen} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>工作区管理</DialogTitle>
          </DialogHeader>
          {content}
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <div className="flex h-screen flex-col items-center justify-center bg-background px-6">
      {content}
    </div>
  );
}
