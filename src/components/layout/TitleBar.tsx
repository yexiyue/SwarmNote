import { useLingui } from "@lingui/react/macro";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  ChevronsUpDown,
  Minus,
  PanelLeft,
  PenLine,
  Search,
  Settings,
  Square,
  X,
} from "lucide-react";
import { openSettingsWindow } from "@/commands/workspace";
import { OPEN_COMMAND_PALETTE } from "@/components/layout/CommandPalette";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { WorkspacePopover } from "@/components/workspace/WorkspacePopover";
import { isMac, modKey } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export function TitleBar() {
  const { t } = useLingui();
  const appWindow = getCurrentWindow();
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const workspace = useWorkspaceStore((s) => s.workspace);

  // When sidebar is collapsed on macOS, add left padding to avoid traffic light overlap
  const needsTrafficLightPadding = isMac && !sidebarOpen;

  return (
    <header
      data-tauri-drag-region
      className="flex h-10 shrink-0 items-center justify-between border-b border-border bg-card px-3"
    >
      {/* Left: Logo + Nav */}
      <div
        className={`flex items-center gap-3 ${needsTrafficLightPadding ? "pl-17.5" : ""}`}
        data-tauri-drag-region
      >
        <div className="group/logo flex items-center gap-1.5">
          {sidebarOpen ? (
            <div className="flex h-5.5 w-5.5 items-center justify-center rounded bg-primary">
              <PenLine className="h-3.5 w-3.5 text-white" />
            </div>
          ) : (
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  onClick={toggleSidebar}
                  className="relative flex h-5.5 w-5.5 items-center justify-center rounded bg-primary"
                >
                  <PenLine className="h-3.5 w-3.5 text-white transition-opacity duration-150 group-hover/logo:opacity-0" />
                  <PanelLeft className="absolute h-3.5 w-3.5 text-white opacity-0 transition-opacity duration-150 group-hover/logo:opacity-100" />
                </button>
              </TooltipTrigger>
              <TooltipContent>
                {t`展开侧边栏`} ({modKey}B)
              </TooltipContent>
            </Tooltip>
          )}
          <span className="text-sm font-semibold text-foreground">SwarmNote</span>
        </div>
        <div className="h-4 w-px bg-border" />
        <WorkspacePopover side="bottom">
          <button
            type="button"
            className="flex items-center gap-1 rounded-md px-1.5 py-1 text-[13px] font-medium text-foreground hover:bg-muted"
          >
            <span className="max-w-32 truncate">{workspace?.name ?? "SwarmNote"}</span>
            <ChevronsUpDown className="h-3 w-3 shrink-0 text-muted-foreground" />
          </button>
        </WorkspacePopover>
      </div>

      {/* Right: Command Palette + Settings + Window Controls */}
      <div className="flex items-center gap-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              onClick={() => document.dispatchEvent(new CustomEvent(OPEN_COMMAND_PALETTE))}
              className="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
            >
              <Search className="h-3.5 w-3.5" />
            </button>
          </TooltipTrigger>
          <TooltipContent>
            {t`命令面板`} ({modKey}P)
          </TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              onClick={() => openSettingsWindow("general")}
              className="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-muted"
            >
              <Settings className="h-3.5 w-3.5" />
            </button>
          </TooltipTrigger>
          <TooltipContent>{t`设置`}</TooltipContent>
        </Tooltip>

        {!isMac && (
          <>
            <div className="h-4 w-px bg-border" />

            {/* Window Controls */}
            <button
              type="button"
              onClick={() => appWindow.minimize()}
              className="flex h-7 w-9 items-center justify-center text-muted-foreground hover:bg-muted"
            >
              <Minus className="h-3.5 w-3.5" />
            </button>
            <button
              type="button"
              onClick={() => appWindow.toggleMaximize()}
              className="flex h-7 w-9 items-center justify-center text-muted-foreground hover:bg-muted"
            >
              <Square className="h-3 w-3" />
            </button>
            <button
              type="button"
              onClick={() => appWindow.close()}
              className="flex h-7 w-9 items-center justify-center text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          </>
        )}
      </div>
    </header>
  );
}
