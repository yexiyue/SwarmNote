import { createFileRoute, Outlet, useLocation, useRouter } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { Info, MonitorSmartphone, Network, Settings } from "lucide-react";
import { useEffect } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/settings/general", icon: Settings, label: "通用" },
  { to: "/settings/network", icon: Network, label: "网络" },
  { to: "/settings/devices", icon: MonitorSmartphone, label: "设备" },
  { to: "/settings/about", icon: Info, label: "关于" },
] as const;

function SettingsLayout() {
  const router = useRouter();
  const { pathname } = useLocation();

  useEffect(() => {
    const unlisten = listen<string>("navigate", (event) => {
      router.navigate({ to: event.payload });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [router]);

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <nav className="flex w-55 flex-col border-r bg-muted/30">
        <div className="p-5 pb-2">
          <h2 className="text-base font-semibold tracking-tight">设置</h2>
          <p className="mt-0.5 text-xs text-muted-foreground">管理应用偏好与设备</p>
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
  );
}

export const Route = createFileRoute("/settings")({
  component: SettingsLayout,
});
