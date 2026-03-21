import { PanelLeft } from "lucide-react";
import type { PropsWithChildren } from "react";
import { StatusBar } from "@/components/layout/StatusBar";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { modKey } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

export function EditorPane({ children }: PropsWithChildren) {
  const sidebarOpen = useUIStore((s) => s.sidebarOpen);
  const setSidebarOpen = useUIStore((s) => s.setSidebarOpen);

  return (
    <main className="relative flex flex-1 flex-col bg-background">
      {/* Sidebar expand hot zone — visible only when sidebar is closed */}
      {!sidebarOpen && (
        <div className="group absolute top-0 bottom-0 left-0 z-10 w-3">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="icon-sm"
                className="absolute top-3 left-0 opacity-0 transition-all duration-200 ease-out scale-95 -translate-x-1 shadow-sm group-hover:opacity-100 group-hover:scale-100 group-hover:translate-x-0 hover-none:opacity-100 hover-none:scale-100 hover-none:translate-x-0"
                onClick={() => setSidebarOpen(true)}
              >
                <PanelLeft className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">展开侧边栏 ({modKey}B)</TooltipContent>
          </Tooltip>
        </div>
      )}
      <div className="flex-1 overflow-auto px-20 py-10">{children}</div>
      <StatusBar />
    </main>
  );
}
