import { Trans } from "@lingui/react/macro";
import { Monitor, Settings } from "lucide-react";
import { useState } from "react";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";
import { DeviceSettings } from "./DeviceSettings";
import { GeneralSettings } from "./GeneralSettings";

type SettingsTab = "general" | "device";

export function SettingsDialog() {
  const settingsOpen = useUIStore((s) => s.settingsOpen);
  const setSettingsOpen = useUIStore((s) => s.setSettingsOpen);
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");

  return (
    <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
      <DialogContent
        className="flex max-h-[85vh] w-180 max-w-[90vw] flex-col gap-0 p-0 sm:h-120"
        showCloseButton
      >
        <DialogTitle className="shrink-0 border-b border-border/70 px-6 py-4 text-base font-semibold">
          <Trans>设置</Trans>
        </DialogTitle>

        <div className="flex min-h-0 flex-1 flex-col sm:flex-row">
          <nav className="flex shrink-0 gap-1 overflow-x-auto border-b border-border/70 p-2 sm:w-40 sm:flex-col sm:overflow-x-visible sm:border-r sm:border-border/70 sm:border-b-0 sm:p-3">
            <TabButton
              icon={Settings}
              active={activeTab === "general"}
              onClick={() => setActiveTab("general")}
            >
              <Trans>通用</Trans>
            </TabButton>
            <TabButton
              icon={Monitor}
              active={activeTab === "device"}
              onClick={() => setActiveTab("device")}
            >
              <Trans>设备</Trans>
            </TabButton>
          </nav>

          <ScrollArea className="flex-1">
            <div className="p-6">
              {activeTab === "general" && <GeneralSettings />}
              {activeTab === "device" && <DeviceSettings />}
            </div>
          </ScrollArea>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function TabButton({
  icon: Icon,
  active,
  onClick,
  children,
}: {
  icon: React.ComponentType<{ className?: string }>;
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex items-center gap-2 rounded-md px-3 py-2 text-sm whitespace-nowrap transition-colors",
        active
          ? "bg-primary/10 font-medium text-primary"
          : "text-muted-foreground hover:bg-muted hover:text-foreground",
      )}
    >
      <Icon className="h-4 w-4" />
      {children}
    </button>
  );
}
