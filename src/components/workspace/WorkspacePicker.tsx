import { Trans, useLingui } from "@lingui/react/macro";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, FolderPlus, PenLine, Plus, RefreshCw } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import {
  getRecentWorkspaces,
  openWorkspaceWindow,
  type RecentWorkspace,
} from "@/commands/workspace";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { WorkspaceItem } from "@/components/workspace/WorkspaceItem";
import { WorkspaceSyncDialog } from "@/components/workspace/WorkspaceSyncDialog";
import { useNetworkStore } from "@/stores/networkStore";
import { useOnboardingStore } from "@/stores/onboardingStore";

interface WorkspacePickerProps {
  mode: "fullscreen" | "dialog";
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function WorkspacePicker({ mode, open: dialogOpen, onOpenChange }: WorkspacePickerProps) {
  const { t } = useLingui();
  const [recents, setRecents] = useState<RecentWorkspace[]>([]);
  const [syncDialogOpen, setSyncDialogOpen] = useState(false);
  const devices = useNetworkStore((s) => s.devices);
  const pairedInOnboarding = useOnboardingStore((s) => s.pairedInOnboarding);
  const setPairedInOnboarding = useOnboardingStore((s) => s.setPairedInOnboarding);

  const onlineDevices = devices.filter((d) => d.status === "online" && d.isPaired);
  const hasOnlineDevices = onlineDevices.length > 0;

  // Reset pairedInOnboarding flag on mount (run once)
  const didResetRef = useRef(false);
  useEffect(() => {
    if (!didResetRef.current && pairedInOnboarding) {
      didResetRef.current = true;
      setPairedInOnboarding(false);
    }
  }, [pairedInOnboarding, setPairedInOnboarding]);

  useEffect(() => {
    getRecentWorkspaces().then(setRecents);
  }, []);

  // Unified handler: both fullscreen and dialog modes go through
  // `open_workspace_window`. In fullscreen mode we pass the caller's window
  // label so the backend can bind the workspace to the current window
  // (which has no workspace yet) instead of spawning a second window.
  async function openPath(path: string) {
    const callerLabel = getCurrentWindow().label;
    const bindToWindow = mode === "fullscreen" ? callerLabel : undefined;
    await openWorkspaceWindow(path, { bindToWindow });
    if (mode === "dialog") {
      onOpenChange?.(false);
    }
  }

  async function handleSelectWorkspace(path: string) {
    await openPath(path);
  }

  async function handleOpenFolder() {
    const selected = await open({ directory: true, title: t`打开工作区文件夹` });
    if (!selected) return;
    await openPath(selected);
  }

  async function handleCreateWorkspace() {
    const selected = await open({ directory: true, title: t`选择新工作区目录` });
    if (!selected) return;
    await openPath(selected);
  }

  const content = (
    <div className="flex min-w-0 flex-col gap-6">
      {/* Header (fullscreen only) */}
      {mode === "fullscreen" && (
        <div className="flex flex-col items-center gap-2">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-primary">
            <PenLine className="h-6 w-6 text-white" />
          </div>
          <h1 className="text-xl font-bold text-foreground">
            <Trans>选择工作区</Trans>
          </h1>
          <p className="text-sm text-muted-foreground">
            <Trans>选择一个已有工作区，或创建新的工作区开始使用。</Trans>
          </p>
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-3">
        <Button variant="outline" className="flex-1 gap-1.5" onClick={handleCreateWorkspace}>
          <Plus className="h-4 w-4" />
          <Trans>创建新工作区</Trans>
        </Button>
        <Button variant="outline" className="flex-1 gap-1.5" onClick={handleOpenFolder}>
          <FolderOpen className="h-4 w-4" />
          <Trans>打开文件夹</Trans>
        </Button>
      </div>

      {/* Sync from paired devices */}
      {hasOnlineDevices && (
        <div className="flex flex-col gap-1.5">
          <Button
            variant="outline"
            className={`w-full gap-1.5${pairedInOnboarding ? " ring-2 ring-primary" : ""}`}
            onClick={() => setSyncDialogOpen(true)}
          >
            <RefreshCw className="h-4 w-4" />
            <Trans>同步已配对设备工作区</Trans>
          </Button>
          <p className="text-center text-xs text-muted-foreground">
            <Trans>{onlineDevices.length} 台设备在线，可同步工作区</Trans>
          </p>
        </div>
      )}

      {/* Recent workspaces */}
      {recents.length > 0 ? (
        <div className="flex min-h-0 flex-col gap-2">
          <span className="text-xs font-medium text-muted-foreground">
            <Trans>最近打开</Trans>
          </span>
          <div className="flex max-h-48 flex-col gap-1.5 overflow-y-auto">
            {recents.map((ws) => (
              <WorkspaceItem key={ws.path} workspace={ws} onClick={handleSelectWorkspace} />
            ))}
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center gap-2 py-8">
          <FolderPlus className="h-10 w-10 text-muted-foreground/40" />
          <p className="text-sm text-muted-foreground">
            <Trans>还没有工作区，创建一个开始使用吧。</Trans>
          </p>
        </div>
      )}
    </div>
  );

  const syncDialog = (
    <WorkspaceSyncDialog open={syncDialogOpen} onOpenChange={setSyncDialogOpen} pickerMode={mode} />
  );

  if (mode === "dialog") {
    return (
      <>
        <Dialog open={dialogOpen} onOpenChange={onOpenChange}>
          <DialogContent className="max-h-[80vh] overflow-x-hidden overflow-y-auto">
            <DialogHeader>
              <DialogTitle>
                <Trans>工作区管理</Trans>
              </DialogTitle>
            </DialogHeader>
            {content}
          </DialogContent>
        </Dialog>
        {syncDialog}
      </>
    );
  }

  return (
    <div className="flex h-screen flex-col items-center justify-center bg-background px-6">
      {content}
      {syncDialog}
    </div>
  );
}
