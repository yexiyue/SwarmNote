import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { ArrowLeft, Monitor, Moon, Sun } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { isMac } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

export const Route = createFileRoute("/settings")({
  component: SettingsPage,
});

const themeOptions = [
  { value: "light" as const, label: "浅色", icon: Sun },
  { value: "dark" as const, label: "深色", icon: Moon },
  { value: "system" as const, label: "跟随系统", icon: Monitor },
];

function SettingsPage() {
  const navigate = useNavigate();
  const theme = useUIStore((s) => s.theme);
  const setTheme = useUIStore((s) => s.setTheme);
  const [deviceName, setDeviceName] = useState("My-Desktop");

  return (
    <div className="flex h-screen flex-col bg-background">
      <header
        data-tauri-drag-region
        className={`flex h-10 shrink-0 items-center gap-3 border-b border-border bg-card px-3 ${isMac ? "pl-[70px]" : ""}`}
      >
        <Button variant="ghost" size="icon-sm" onClick={() => navigate({ to: "/" })}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <h1 className="text-sm font-semibold text-foreground">设置</h1>
      </header>

      {/* Content */}
      <div className="mx-auto w-full max-w-xl space-y-6 overflow-y-auto px-6 py-8">
        {/* Appearance */}
        <section className="rounded-lg border border-border bg-card p-5">
          <h2 className="text-sm font-medium text-foreground">外观</h2>
          <Separator className="my-3" />
          <div className="space-y-2">
            <Label>主题</Label>
            <div className="flex gap-2">
              {themeOptions.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  onClick={() => setTheme(opt.value)}
                  className={`flex flex-1 items-center justify-center gap-2 rounded-lg border px-3 py-2 text-sm transition-colors ${
                    theme === opt.value
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-border text-muted-foreground hover:bg-muted"
                  }`}
                >
                  <opt.icon className="h-4 w-4" />
                  {opt.label}
                </button>
              ))}
            </div>
          </div>
        </section>

        {/* Device */}
        <section className="rounded-lg border border-border bg-card p-5">
          <h2 className="text-sm font-medium text-foreground">设备</h2>
          <Separator className="my-3" />
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="device-name">设备名称</Label>
              <Input
                id="device-name"
                value={deviceName}
                onChange={(e) => setDeviceName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>Peer ID</Label>
              <p className="rounded-md bg-muted px-3 py-2 font-mono text-xs text-muted-foreground">
                12D3KooWAbCdEfGhIjKlMnOpQrStUvWxYz...a8f2
              </p>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
