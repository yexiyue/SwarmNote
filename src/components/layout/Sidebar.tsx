import { Trans, useLingui } from "@lingui/react/macro";
import {
  ChevronsUpDown,
  FilePlus,
  FolderOpen,
  FolderPlus,
  FolderTree,
  List,
  PanelLeft,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { DocumentOutline } from "@/components/editor/DocumentOutline";
import { FileTree } from "@/components/filetree/FileTree";
import { SyncStatusBar } from "@/components/layout/SyncStatusBar";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { WorkspacePopover } from "@/components/workspace/WorkspacePopover";
import { cn, isMac, modKey } from "@/lib/utils";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import {
  SIDEBAR_WIDTH_MAX,
  SIDEBAR_WIDTH_MIN,
  type SidebarTab,
  useUIStore,
} from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

const TABS: { id: SidebarTab; icon: typeof FolderTree }[] = [
  { id: "filetree", icon: FolderTree },
  { id: "outline", icon: List },
];

export function Sidebar() {
  const { t } = useLingui();
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const sidebarWidth = useUIStore((s) => s.sidebarWidth);
  const setSidebarWidth = useUIStore((s) => s.setSidebarWidth);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const sidebarTab = useUIStore((s) => s.sidebarTab);
  const setSidebarTab = useUIStore((s) => s.setSidebarTab);
  const workspace = useWorkspaceStore((s) => s.workspace);
  const rescan = useFileTreeStore((s) => s.rescan);
  const createAndOpenFile = useFileTreeStore((s) => s.createAndOpenFile);
  const createDir = useFileTreeStore((s) => s.createDir);

  const handleResizeMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      const startX = e.clientX;
      const startWidth = useUIStore.getState().sidebarWidth;

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
    [setSidebarWidth],
  );

  const handleResizeKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const STEP = 16;
      const current = useUIStore.getState().sidebarWidth;
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        setSidebarWidth(current - STEP);
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        setSidebarWidth(current + STEP);
      } else if (e.key === "Home") {
        e.preventDefault();
        setSidebarWidth(SIDEBAR_WIDTH_MIN);
      } else if (e.key === "End") {
        e.preventDefault();
        setSidebarWidth(SIDEBAR_WIDTH_MAX);
      }
    },
    [setSidebarWidth],
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
      <div className="flex h-full flex-col gap-3 p-3" style={{ minWidth: sidebarWidth }}>
        {/* macOS traffic light spacer */}
        {isMac && <div className="h-6 shrink-0" data-tauri-drag-region />}

        {/* Header: workspace switcher + tree actions */}
        <div className="flex items-center justify-between gap-1">
          <WorkspacePopover side="bottom">
            <button
              type="button"
              title={workspace?.path}
              aria-label={t`切换工作区`}
              className="group/ws -ml-1 flex min-w-0 flex-1 items-center gap-1.5 rounded-md px-1 py-1 text-left outline-none transition-colors hover:bg-sidebar-accent focus-visible:bg-sidebar-accent aria-expanded:bg-sidebar-accent"
            >
              <FolderOpen className="h-3.5 w-3.5 shrink-0 text-sidebar-primary" />
              <span className="min-w-0 flex-1 truncate text-[13px] font-semibold text-sidebar-foreground">
                {workspace?.name ?? <Trans>我的笔记</Trans>}
              </span>
              <ChevronsUpDown className="h-3 w-3 shrink-0 text-muted-foreground/60 transition-colors group-hover/ws:text-muted-foreground group-aria-expanded/ws:text-muted-foreground" />
            </button>
          </WorkspacePopover>
          <div className="flex shrink-0 items-center gap-0.5">
            {sidebarTab === "filetree" && (
              <>
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
                <div className="mx-0.5 h-3.5 w-px bg-sidebar-border" aria-hidden="true" />
              </>
            )}
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

        {/* Tab switcher */}
        <div className="flex shrink-0 border-b border-sidebar-border">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setSidebarTab(tab.id)}
              className={cn(
                "flex flex-1 items-center justify-center gap-1.5 py-1.5 text-xs transition-colors",
                sidebarTab === tab.id
                  ? "border-b-2 border-primary text-sidebar-foreground font-medium"
                  : "border-b-2 border-transparent text-muted-foreground hover:text-sidebar-foreground",
              )}
            >
              <tab.icon className="h-3.5 w-3.5" />
              {tab.id === "filetree" ? t`文件` : t`大纲`}
            </button>
          ))}
        </div>

        {/* Content area */}
        <div ref={treeContainerRef} className="flex-1 overflow-hidden">
          {sidebarTab === "filetree" ? (
            <FileTree width={sidebarWidth - 24} height={treeHeight} />
          ) : (
            <DocumentOutline height={treeHeight} />
          )}
        </div>

        {/* Sync status indicator */}
        <SyncStatusBar workspaceUuid={workspaceUuid} />
      </div>

      {/* Resize handle */}
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
