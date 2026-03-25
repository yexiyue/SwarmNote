import { Trans } from "@lingui/react/macro";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { FolderOpen, FolderPlus, Loader2, Minus, MoreVertical, PenLine, X } from "lucide-react";
import { useEffect } from "react";

import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { isMac } from "@/lib/utils";
import { transitionPickerToApp } from "@/lib/windowUtils";
import { useWorkspaceStore } from "@/stores/workspaceStore";

interface WorkspacePickerProps {
  onSelected: () => void;
}

export function WorkspacePicker({ onSelected }: WorkspacePickerProps) {
  const {
    recentWorkspaces,
    fetchRecentWorkspaces,
    openWorkspace,
    selectAndOpenWorkspace,
    isLoading,
  } = useWorkspaceStore();

  useEffect(() => {
    fetchRecentWorkspaces();
  }, [fetchRecentWorkspaces]);

  async function handleOpenRecent(path: string, name: string) {
    try {
      await openWorkspace(path);
      await transitionPickerToApp(name);
      onSelected();
    } catch (err) {
      console.error("Failed to open workspace:", err);
    }
  }

  async function handleSelectFolder() {
    try {
      const ws = await selectAndOpenWorkspace();
      if (ws) {
        await transitionPickerToApp(ws.name);
        onSelected();
      }
    } catch (err) {
      console.error("Failed to open workspace:", err);
    }
  }

  const appWindow = getCurrentWebviewWindow();

  return (
    <div className="flex h-screen flex-row bg-background">
      {/* Left panel — recent workspaces */}
      <div className="flex w-60 shrink-0 flex-col border-r border-border bg-card">
        {/* Drag region for left panel */}
        <div data-tauri-drag-region className="h-10 shrink-0" />

        <ScrollArea className="flex-1">
          {recentWorkspaces.length === 0 ? (
            <div className="flex items-center justify-center p-6 text-xs text-muted-foreground">
              <Trans>暂无最近工作区</Trans>
            </div>
          ) : (
            <div className="px-1 pb-2">
              {recentWorkspaces.map((ws) => (
                <button
                  key={ws.path}
                  type="button"
                  className="group flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors hover:bg-accent"
                  onClick={() => handleOpenRecent(ws.path, ws.name)}
                  disabled={isLoading}
                >
                  <div className="min-w-0 flex-1">
                    <div className="truncate text-sm font-medium text-foreground">{ws.name}</div>
                    <div className="truncate text-xs text-muted-foreground">{ws.path}</div>
                  </div>
                  {/* TODO: implement context menu (remove from list, reveal in explorer, etc.) */}
                  <MoreVertical className="h-3.5 w-3.5 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100" />
                </button>
              ))}
            </div>
          )}
        </ScrollArea>
      </div>

      {/* Right panel — logo + actions */}
      <div className="flex flex-1 flex-col">
        {/* Title bar / drag region with window controls */}
        <div data-tauri-drag-region className="flex h-10 shrink-0 items-center justify-end px-2">
          {!isMac && (
            <div className="flex">
              <button
                type="button"
                onClick={() => appWindow.minimize()}
                className="flex h-7 w-9 items-center justify-center text-muted-foreground hover:bg-muted"
              >
                <Minus className="h-3.5 w-3.5" />
              </button>
              <button
                type="button"
                onClick={() => appWindow.close()}
                className="flex h-7 w-9 items-center justify-center text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          )}
        </div>

        {/* Content */}
        <div className="flex flex-1 flex-col items-center justify-center px-10 pb-10">
          {/* Logo */}
          <div className="mb-2 flex h-16 w-16 items-center justify-center rounded-2xl bg-primary">
            <PenLine className="h-8 w-8 text-white" />
          </div>
          <h1 className="text-xl font-semibold text-foreground">SwarmNote</h1>
          <p className="mb-8 text-xs text-muted-foreground">v0.1.0</p>

          {/* Action cards */}
          {/* TODO: "新建工作区" and "打开已有文件夹" currently share the same handler;
              differentiate once backend supports workspace creation vs opening separately */}
          <div className="-ml-6 flex w-full max-w-sm flex-col gap-3">
            <ActionCard
              title={<Trans>新建工作区</Trans>}
              description={<Trans>在指定文件夹下创建一个新的工作区。</Trans>}
              buttonLabel={<Trans>创建</Trans>}
              icon={<FolderPlus className="h-4 w-4" />}
              onClick={handleSelectFolder}
              isLoading={isLoading}
            />
            <ActionCard
              title={<Trans>打开已有文件夹</Trans>}
              description={<Trans>将一个本地文件夹作为工作区打开。</Trans>}
              buttonLabel={<Trans>打开</Trans>}
              icon={<FolderOpen className="h-4 w-4" />}
              onClick={handleSelectFolder}
              isLoading={isLoading}
              variant="outline"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function ActionCard({
  title,
  description,
  buttonLabel,
  icon,
  onClick,
  isLoading,
  variant = "default",
}: {
  title: React.ReactNode;
  description: React.ReactNode;
  buttonLabel: React.ReactNode;
  icon: React.ReactNode;
  onClick: () => void;
  isLoading: boolean;
  variant?: "default" | "outline";
}) {
  return (
    <div className="flex items-center gap-4 rounded-lg border border-border p-4">
      <div className="min-w-0 flex-1">
        <div className="text-sm font-semibold text-primary">{title}</div>
        <div className="mt-0.5 text-xs text-muted-foreground">{description}</div>
      </div>
      <Button
        variant={variant}
        size="sm"
        onClick={onClick}
        disabled={isLoading}
        className="gap-1.5"
      >
        {isLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : icon}
        {buttonLabel}
      </Button>
    </div>
  );
}
