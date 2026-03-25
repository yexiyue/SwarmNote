import { Trans, useLingui } from "@lingui/react/macro";
import {
  AppWindow,
  FilePlus,
  FolderOpen,
  FolderPlus,
  Globe,
  Monitor,
  Moon,
  PanelLeft,
  Sun,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { FileTree } from "@/components/filetree/FileTree";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { type Locale, locales } from "@/i18n";
import { isMac, modKey } from "@/lib/utils";
import { openPickerWindow } from "@/lib/windowUtils";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

const themeIcons = { light: Sun, dark: Moon, system: Monitor } as const;
const themeOrder: Array<"light" | "dark" | "system"> = ["light", "dark", "system"];
const localeKeys = Object.keys(locales) as Locale[];

export function Sidebar() {
  const { t } = useLingui();
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const theme = useUIStore((s) => s.theme);
  const setTheme = useUIStore((s) => s.setTheme);
  const locale = useUIStore((s) => s.locale);
  const setLocale = useUIStore((s) => s.setLocale);
  const workspace = useWorkspaceStore((s) => s.workspace);
  const rescan = useFileTreeStore((s) => s.rescan);
  const createFile = useFileTreeStore((s) => s.createFile);
  const createDir = useFileTreeStore((s) => s.createDir);

  const ThemeIcon = themeIcons[theme];

  // Rescan when workspace changes
  useEffect(() => {
    if (workspace) {
      rescan();
    }
  }, [workspace, rescan]);

  function cycleTheme() {
    const idx = themeOrder.indexOf(theme);
    setTheme(themeOrder[(idx + 1) % themeOrder.length]);
  }

  function toggleLocale() {
    const idx = localeKeys.indexOf(locale);
    setLocale(localeKeys[(idx + 1) % localeKeys.length]);
  }

  const handleCreateFile = useCallback(() => {
    createFile("", t`新建笔记`);
  }, [createFile, t]);

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

        {/* Device Info + Quick Settings */}
        <div className="flex items-center gap-2 border-t border-sidebar-border px-1 pt-2">
          <Monitor className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <div className="flex min-w-0 flex-1 flex-col gap-px">
            <span className="text-xs font-medium text-sidebar-foreground">My-Desktop</span>
            <span className="truncate text-[10px] text-muted-foreground">12D3KooW...a8f2</span>
          </div>
          <div className="flex shrink-0 gap-0.5">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-muted-foreground"
                  onClick={openPickerWindow}
                  aria-label={t`管理工作区`}
                >
                  <AppWindow className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <Trans>管理工作区</Trans>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-muted-foreground"
                  onClick={cycleTheme}
                  aria-label={t`切换主题`}
                >
                  <ThemeIcon className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <Trans>切换主题</Trans>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-muted-foreground"
                  onClick={toggleLocale}
                  aria-label={t`切换语言`}
                >
                  <Globe className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>{locales[locale]}</TooltipContent>
            </Tooltip>
          </div>
        </div>
      </div>
    </aside>
  );
}
