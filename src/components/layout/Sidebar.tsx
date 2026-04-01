import { Trans, useLingui } from "@lingui/react/macro";
import { FilePlus, FolderOpen, FolderPlus, PanelLeft } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { openSettingsWindow } from "@/commands/workspace";
import { FileTree } from "@/components/filetree/FileTree";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { isMac, modKey } from "@/lib/utils";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { useNetworkStore } from "@/stores/networkStore";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export function Sidebar() {
  const { t } = useLingui();
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const workspace = useWorkspaceStore((s) => s.workspace);
  const rescan = useFileTreeStore((s) => s.rescan);
  const createAndOpenFile = useFileTreeStore((s) => s.createAndOpenFile);
  const createDir = useFileTreeStore((s) => s.createDir);

  const nodeStatus = useNetworkStore((s) => s.status);
  const nodeLoading = useNetworkStore((s) => s.loading);
  const devices = useNetworkStore((s) => s.devices);
  const connectedPeers = devices.filter((d) => d.status === "online");

  // Rescan when workspace changes
  useEffect(() => {
    if (workspace) {
      rescan();
    }
  }, [workspace, rescan]);

  const handleCreateFile = useCallback(() => {
    createAndOpenFile("", t`新建笔记`);
  }, [createAndOpenFile, t]);

  const handleCreateDir = useCallback(() => {
    createDir("", t`新建文件夹`);
  }, [createDir, t]);

  // Measure available height for the tree
  const [treeHeight, setTreeHeight] = useState(400);
  const treeContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = treeContainerRef.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setTreeHeight(entry.contentRect.height);
      }
    });

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <aside
      className="flex shrink-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar transition-[width] duration-200 ease-in-out"
      style={{ width: sidebarOpen ? 256 : 0 }}
    >
      <div className="flex h-full min-w-64 flex-col gap-3 p-3">
        {/* macOS traffic light spacer */}
        {isMac && <div className="h-6 shrink-0" data-tauri-drag-region />}
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <FolderOpen className="h-4 w-4 text-sidebar-primary" />
            <span className="text-[13px] font-semibold text-sidebar-foreground">
              {workspace?.name ?? <Trans>我的笔记</Trans>}
            </span>
          </div>
          <div className="flex gap-0.5">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-muted-foreground"
                  onClick={handleCreateFile}
                >
                  <FilePlus className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <Trans>新建文件</Trans>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-muted-foreground"
                  onClick={handleCreateDir}
                >
                  <FolderPlus className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <Trans>新建文件夹</Trans>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-muted-foreground"
                  onClick={toggleSidebar}
                >
                  <PanelLeft className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                {t`收起侧边栏`} ({modKey}B)
              </TooltipContent>
            </Tooltip>
          </div>
        </div>

        {/* File Tree */}
        <div ref={treeContainerRef} className="flex-1 overflow-hidden">
          <FileTree width={232} height={treeHeight} />
        </div>

        {/* Network status indicator */}
        <div className="border-t border-sidebar-border px-1 pt-2">
          <button
            type="button"
            className="flex w-full items-center gap-1.5 rounded-sm px-1 py-1 text-left hover:bg-sidebar-accent"
            onClick={() => openSettingsWindow("network")}
          >
            <span
              className={`inline-block h-2 w-2 shrink-0 rounded-full ${
                nodeLoading
                  ? "animate-pulse bg-yellow-500"
                  : nodeStatus === "running"
                    ? "bg-green-500"
                    : nodeStatus === "error"
                      ? "bg-red-500"
                      : "bg-gray-400"
              }`}
            />
            <span className="truncate text-xs text-muted-foreground">
              {nodeLoading
                ? "连接中..."
                : nodeStatus === "running"
                  ? connectedPeers.length > 0
                    ? `已连接 · ${connectedPeers.length} 台设备在线`
                    : "已连接"
                  : nodeStatus === "error"
                    ? "连接失败"
                    : "未连接"}
            </span>
          </button>
        </div>
      </div>
    </aside>
  );
}
