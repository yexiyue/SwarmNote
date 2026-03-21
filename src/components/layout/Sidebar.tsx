import { Trans, useLingui } from "@lingui/react/macro";
import {
  ChevronDown,
  ChevronRight,
  FilePlus,
  FileText,
  Folder,
  FolderOpen,
  FolderPlus,
  Globe,
  Monitor,
  Moon,
  PanelLeft,
  Sun,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { Locale } from "@/i18n";
import { locales } from "@/i18n";
import { cn, isMac, modKey } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

const treeItemBase = "flex items-center gap-1.5 rounded px-2 py-[5px] text-[13px]";

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

  const ThemeIcon = themeIcons[theme];

  function cycleTheme() {
    const idx = themeOrder.indexOf(theme);
    setTheme(themeOrder[(idx + 1) % themeOrder.length]);
  }

  function toggleLocale() {
    const idx = localeKeys.indexOf(locale);
    setLocale(localeKeys[(idx + 1) % localeKeys.length]);
  }

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
              <Trans>我的笔记</Trans>
            </span>
          </div>
          <div className="flex gap-0.5">
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon-xs" className="text-muted-foreground">
                  <FilePlus className="h-3.5 w-3.5" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                <Trans>新建文件</Trans>
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon-xs" className="text-muted-foreground">
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

        {/* File Tree (static placeholder) */}
        <ScrollArea className="flex-1">
          <div className="flex flex-col gap-px">
            {/* Folder: 日记 (expanded) */}
            <div className={cn(treeItemBase, "text-sidebar-foreground")}>
              <ChevronDown className="h-3.5 w-3.5 text-muted-foreground" />
              <Folder className="h-3.5 w-3.5 text-primary" />
              <span>日记</span>
            </div>
            <div className={cn(treeItemBase, "bg-sidebar-accent pl-[34px]")}>
              <FileText className="h-3.5 w-3.5 text-primary" />
              <span className="font-medium text-sidebar-accent-foreground">2026-03-21</span>
            </div>
            <div className={cn(treeItemBase, "pl-[34px] text-sidebar-foreground")}>
              <FileText className="h-3.5 w-3.5 text-muted-foreground" />
              <span>2026-03-19</span>
            </div>
            {/* Folder: 项目笔记 (collapsed) */}
            <div className={cn(treeItemBase, "text-sidebar-foreground")}>
              <ChevronRight className="h-3.5 w-3.5 text-muted-foreground" />
              <Folder className="h-3.5 w-3.5 text-muted-foreground" />
              <span>项目笔记</span>
            </div>
            <div className={cn(treeItemBase, "text-sidebar-foreground")}>
              <FileText className="h-3.5 w-3.5 text-muted-foreground" />
              <span>快速笔记</span>
            </div>
          </div>
        </ScrollArea>

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
