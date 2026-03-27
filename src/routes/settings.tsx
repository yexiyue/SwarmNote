import { createFileRoute, Outlet, useRouter } from "@tanstack/react-router";
import { listen } from "@tauri-apps/api/event";
import { Info, MonitorSmartphone, Settings } from "lucide-react";
import { useEffect } from "react";

import { cn } from "@/lib/utils";

const navItems = [
  { to: "/settings/general", icon: Settings, label: "通用" },
  { to: "/settings/devices", icon: MonitorSmartphone, label: "设备" },
  { to: "/settings/about", icon: Info, label: "关于" },
] as const;

function SettingsLayout() {
  const router = useRouter();
  const pathname = router.state.location.pathname;

  useEffect(() => {
    const unlisten = listen<string>("navigate", (event) => {
      router.navigate({ to: event.payload });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [router]);

  return (
    <div className="flex h-screen">
      {/* SideNav */}
      <nav className="flex w-[200px] flex-col border-r bg-sidebar p-4">
        <h2 className="mb-3 text-sm font-semibold">设置</h2>
        <div className="flex flex-col gap-1">
          {navItems.map((item) => {
            const isActive = pathname.startsWith(item.to);
            return (
              <button
                key={item.to}
                type="button"
                onClick={() => router.navigate({ to: item.to })}
                className={cn(
                  "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-muted-foreground hover:bg-sidebar-accent/50",
                )}
              >
                <item.icon className="h-4 w-4" />
                {item.label}
              </button>
            );
          })}
        </div>
      </nav>

      {/* Content */}
      <main className="flex-1 overflow-y-auto">
        <Outlet />
      </main>
    </div>
  );
}

export const Route = createFileRoute("/settings")({
  component: SettingsLayout,
});
