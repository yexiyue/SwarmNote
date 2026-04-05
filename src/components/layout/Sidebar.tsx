import { Trans, useLingui } from "@lingui/react/macro";
import { FilePlus, FolderOpen, FolderPlus, PanelLeft } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { FileTree } from "@/components/filetree/FileTree";
import { SyncStatusBar } from "@/components/layout/SyncStatusBar";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { isMac, modKey } from "@/lib/utils";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { SIDEBAR_WIDTH_MAX, SIDEBAR_WIDTH_MIN, useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export function Sidebar() {
  const { t } = useLingui();
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const setSidebarWidth = useUIStore((s) => s.setSidebarWidth);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const workspace = useWorkspaceStore((s) => s.workspace);
  const rescan = useFileTreeStore((s) => s.rescan);
  const createAndOpenFile = useFileTreeStore((s) => s.createAndOpenFile);
  const createDir = useFileTreeStore((s) => s.createDir);

  const handleResizeMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      // Closure locals are enough — the handlers below are all defined inside
      // this mousedown and close over `startX` / `startWidth` directly.
      const startX = e.clientX;
      const startWidth = sidebarWidth;

      const onMove = (moveEvent: MouseEvent) => {
        setSidebarWidth(startWidth + (moveEvent.clientX - startX));
      };

      const onUp = () => {
        document.body.style.removeProperty("cursor");
        document.body.style.removeProperty("user-select");
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
      };

      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
      window.addEventListener("mousemove", onMove);
      window.addEventListener("mouseup", onUp);
    },
    [sidebarWidth, setSidebarWidth],
  );

  const handleResizeKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Keyboard users: Left/Right arrows resize by 16px, Home/End jump to min/max.
      const STEP = 16;
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        setSidebarWidth(sidebarWidth - STEP);
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        setSidebarWidth(sidebarWidth + STEP);
      } else if (e.key === "Home") {
        e.preventDefault();
        setSidebarWidth(SIDEBAR_WIDTH_MIN);
      } else if (e.key === "End") {
        e.preventDefault();
        setSidebarWidth(SIDEBAR_WIDTH_MAX);
      }
    },
    [sidebarWidth, setSidebarWidth],
  );

  const workspaceUuid = workspace?.id;

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
      className="group/sidebar relative flex shrink-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar transition-[width] duration-200 ease-in-out"
      style={{ width: sidebarOpen ? sidebarWidth : 0 }}
    >
      {/* `minWidth` prevents content from squeezing while the outer `<aside>`
          animates `width` to 0 during collapse; the outer aside still caps
          the visible width, so we don't need to set width here again. */}
      <div className="flex h-full flex-col gap-3 p-3" style={{ minWidth: sidebarWidth }}>
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
          <FileTree width={sidebarWidth - 24} height={treeHeight} />
        </div>

        {/* Sync status indicator */}
        <SyncStatusBar workspaceUuid={workspaceUuid} />
      </div>

      {/* Resize handle: 4px drag strip on the right edge. Keyboard users can
          use arrow keys / Home / End to resize. Hidden when the sidebar is
          collapsed so there's no stray affordance. */}
      {sidebarOpen && (
        <button
          type="button"
          aria-label={t`调整侧边栏宽度 (当前 ${sidebarWidth} 像素)`}
          onMouseDown={handleResizeMouseDown}
          onKeyDown={handleResizeKeyDown}
          className="absolute top-0 right-0 bottom-0 z-10 w-1 cursor-col-resize border-0 bg-transparent p-0 hover:bg-primary/20 active:bg-primary/30 focus-visible:bg-primary/30 focus-visible:outline-none"
        />
      )}
    </aside>
  );
}
