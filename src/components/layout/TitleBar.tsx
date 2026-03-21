import { useNavigate } from "@tanstack/react-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, PenLine, Search, Settings, Square, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { isMac, modKey } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

export function TitleBar() {
  const navigate = useNavigate();
  const appWindow = getCurrentWindow();
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);

  // When sidebar is collapsed on macOS, add left padding to avoid traffic light overlap
  const needsTrafficLightPadding = isMac && !sidebarOpen;

  return (
    <header
      data-tauri-drag-region
      className="flex h-10 shrink-0 items-center justify-between border-b border-border bg-card px-3"
    >
      {/* Left: Logo + Nav */}
      <div
        className={`flex items-center gap-3 ${needsTrafficLightPadding ? "pl-[70px]" : ""}`}
        data-tauri-drag-region
      >
        <div className="flex items-center gap-1.5">
          <div className="flex h-[22px] w-[22px] items-center justify-center rounded bg-primary">
            <PenLine className="h-3.5 w-3.5 text-white" />
          </div>
          <span className="text-sm font-semibold text-foreground">SwarmNote</span>
        </div>
        <div className="h-4 w-px bg-border" />
        <span className="text-[13px] font-medium text-foreground">笔记</span>
      </div>

      {/* Right: Search + Settings + Window Controls */}
      <div className="flex items-center gap-1">
        <button
          type="button"
          onClick={() => document.dispatchEvent(new CustomEvent("open-command-palette"))}
          className="flex items-center gap-1.5 rounded-md bg-secondary px-2.5 py-[5px] text-muted-foreground hover:bg-secondary/80"
        >
          <Search className="h-3.5 w-3.5" />
          <span className="text-xs">搜索...</span>
          <span className="text-[10px] font-medium">{modKey}P</span>
        </button>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon-sm" onClick={() => navigate({ to: "/settings" })}>
              <Settings className="h-4 w-4 text-muted-foreground" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>设置</TooltipContent>
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
