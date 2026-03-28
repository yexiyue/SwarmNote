import { Trans, useLingui } from "@lingui/react/macro";
import {
  ChevronsUpDown,
  FilePlus,
  FolderOpen,
  FolderPlus,
  Globe,
  Monitor,
  Moon,
  PanelLeft,
  Settings,
  Sun,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { getDeviceInfo } from "@/commands/identity";
import { openSettingsWindow } from "@/commands/workspace";
import { FileTree } from "@/components/filetree/FileTree";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { WorkspacePopover } from "@/components/workspace/WorkspacePopover";
import { type Locale, locales } from "@/i18n";
import { isMac, modKey } from "@/lib/utils";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { useNetworkStore } from "@/stores/networkStore";
import { usePairingStore } from "@/stores/pairingStore";
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

  const pairedDevices = usePairingStore((s) => s.pairedDevices);
  const onlineCount = pairedDevices.filter((d) => d.isOnline).length;
  const [deviceName, setDeviceName] = useState("...");

  const nodeStatus = useNetworkStore((s) => s.status);
  const connectedPeers = useNetworkStore((s) => s.connectedPeers);

  const ThemeIcon = themeIcons[theme];

  useEffect(() => {
    getDeviceInfo()
      .then((info) => setDeviceName(info.device_name))
      .catch(() => setDeviceName("SwarmNote"));
  }, []);

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

        {/* Workspace + Device Info + Quick Settings */}
        <div className="flex flex-col gap-2 border-t border-sidebar-border px-1 pt-2">
          {/* Workspace row */}
          <WorkspacePopover>
            <button
              type="button"
              className="flex w-full items-center gap-1.5 rounded-sm px-1 py-1 text-left hover:bg-sidebar-accent"
            >
              <FolderOpen className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              <span className="min-w-0 flex-1 truncate text-xs font-medium text-sidebar-foreground">
                {workspace?.name ?? <Trans>选择工作区</Trans>}
              </span>
              <ChevronsUpDown className="h-3 w-3 shrink-0 text-muted-foreground" />
            </button>
          </WorkspacePopover>

          {/* Network status indicator */}
          <button
            type="button"
            className="flex w-full items-center gap-1.5 rounded-sm px-1 py-1 text-left hover:bg-sidebar-accent"
            onClick={() => openSettingsWindow("network")}
          >
            <span
              className={`inline-block h-2 w-2 shrink-0 rounded-full ${
                nodeStatus === "running"
                  ? "bg-green-500"
                  : nodeStatus === "starting"
                    ? "bg-yellow-500 animate-pulse"
                    : nodeStatus === "error"
                      ? "bg-red-500"
                      : "bg-gray-400"
              }`}
            />
            <span className="truncate text-xs text-muted-foreground">
              {nodeStatus === "running"
                ? connectedPeers.length > 0
                  ? `已连接 · ${connectedPeers.length} 台设备在线`
                  : "已连接"
                : nodeStatus === "starting"
                  ? "连接中..."
                  : nodeStatus === "error"
                    ? "连接失败"
                    : "未连接"}
            </span>
          </button>

          {/* Device info + quick settings */}
          <div className="flex items-center gap-2">
            <button
              type="button"
              className="flex min-w-0 flex-1 items-center gap-2 rounded-sm px-1 py-1 text-left hover:bg-sidebar-accent"
              onClick={() => openSettingsWindow("devices")}
            >
              <Monitor className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              <div className="flex min-w-0 flex-1 flex-col gap-px">
                <span className="text-xs font-medium text-sidebar-foreground">{deviceName}</span>
                <span className="truncate text-[10px] text-muted-foreground">
                  {onlineCount > 0 ? (
                    <>
                      <span className="inline-block h-1.5 w-1.5 rounded-full bg-green-500" />{" "}
                      {onlineCount} 在线
                    </>
                  ) : (
                    "无设备在线"
                  )}
                </span>
              </div>
            </button>
            <div className="flex shrink-0 gap-0.5">
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
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    className="text-muted-foreground"
                    onClick={() => openSettingsWindow("general")}
                    aria-label={t`设置`}
                  >
                    <Settings className="h-3.5 w-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <Trans>设置</Trans>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
        </div>
      </div>
    </aside>
  );
}
