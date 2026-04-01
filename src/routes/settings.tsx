import { createFileRoute, Outlet, useLocation, useRouter } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Info, Minus, MonitorSmartphone, RefreshCw, Settings, X } from "lucide-react";
import { useEffect } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn, isMac } from "@/lib/utils";

const navItems = [
  { to: "/settings/general", icon: Settings, label: "通用" },
  { to: "/settings/sync", icon: RefreshCw, label: "同步" },
  { to: "/settings/devices", icon: MonitorSmartphone, label: "设备" },
  { to: "/settings/about", icon: Info, label: "关于" },
] as const;

function SettingsLayout() {
  const router = useRouter();
  const { pathname } = useLocation();
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const unlisten = listen<string>("navigate", (event) => {
      router.navigate({ to: event.payload });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [router]);

  return (
    <div className="flex h-screen flex-col bg-muted/30">
      {/* Title Bar */}
      <header
        data-tauri-drag-region
        className="flex h-10 shrink-0 items-center justify-between border-b border-border bg-background/60 px-4"
      >
        {isMac ? (
          <div className="w-17.5" data-tauri-drag-region />
        ) : (
          <span className="text-sm font-semibold text-foreground" data-tauri-drag-region>
            设置
          </span>
        )}
        <div className="flex items-center gap-1">
          {!isMac && (
            <>
              <button
                type="button"
                onClick={() => appWindow.minimize()}
                className="flex h-7 w-9 items-center justify-center text-muted-foreground hover:bg-muted"
              >
                <Minus className="h-3.5 w-3.5" />
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

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <nav className="flex w-55 flex-col border-r bg-background/60">
          <div className="p-5 pb-2">
            {isMac && <h2 className="text-base font-semibold tracking-tight">设置</h2>}
            <p className={cn("text-xs text-muted-foreground", isMac && "mt-0.5")}>
              管理应用偏好与设备
            </p>
          </div>

          <div className="flex flex-col gap-0.5 px-3 pt-2">
            {navItems.map((item) => {
              const isActive = pathname.startsWith(item.to);
              return (
                <button
                  key={item.to}
                  type="button"
                  onClick={() => router.navigate({ to: item.to })}
                  className={cn(
                    "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-all",
                    isActive
                      ? "bg-background font-medium text-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                  )}
                >
                  <item.icon className="h-4 w-4 shrink-0" />
                  {item.label}
                </button>
              );
            })}
          </div>

          {/* Bottom branding */}
          <div className="mt-auto border-t px-5 py-4">
            <div className="text-xs text-muted-foreground">SwarmNote</div>
            <div className="text-[11px] text-muted-foreground/60">v0.2.0</div>
          </div>
        </nav>

        {/* Content */}
        <ScrollArea className="flex-1">
          <main className="mx-auto max-w-2xl px-8 py-8">
            <Outlet />
          </main>
        </ScrollArea>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings")({
  component: SettingsLayout,
});
