import { createFileRoute } from "@tanstack/react-router";
import { Cpu, Fingerprint, Info, Monitor } from "lucide-react";
import { useEffect, useState } from "react";
import type { DeviceInfo } from "@/commands/identity";
import { getDeviceInfo } from "@/commands/identity";
import { Separator } from "@/components/ui/separator";

function InfoRow({
  icon: Icon,
  label,
  value,
  mono,
}: {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex items-center gap-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
          <Icon className="h-4 w-4 text-muted-foreground" />
        </div>
        <span className="text-sm">{label}</span>
      </div>
      <span
        className={`text-sm text-muted-foreground ${mono ? "max-w-60 truncate font-mono text-xs" : ""}`}
      >
        {value}
      </span>
    </div>
  );
}

function AboutPage() {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);

  useEffect(() => {
    getDeviceInfo().then(setDeviceInfo).catch(console.error);
  }, []);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold tracking-tight">关于</h1>
        <p className="mt-1 text-sm text-muted-foreground">SwarmNote 版本与设备信息</p>
      </div>

      <div className="space-y-4">
        {/* App Info */}
        <div className="rounded-xl border bg-card">
          <div className="flex items-center gap-4 px-5 py-5">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10">
              <span className="text-lg font-bold text-primary">SN</span>
            </div>
            <div>
              <h3 className="text-sm font-semibold">SwarmNote</h3>
              <p className="text-xs text-muted-foreground">去中心化 P2P 笔记应用</p>
            </div>
            <span className="ml-auto rounded-full bg-muted px-3 py-1 text-xs font-medium">
              v0.2.0
            </span>
          </div>
        </div>

        {/* Device Info */}
        {deviceInfo && (
          <div className="rounded-xl border bg-card">
            <div className="px-5 py-4">
              <h3 className="text-sm font-medium">设备信息</h3>
              <p className="mt-0.5 text-xs text-muted-foreground">当前运行设备的详细信息</p>
            </div>
            <Separator />
            <div className="px-5 py-3">
              <div className="space-y-1">
                <InfoRow icon={Monitor} label="设备名称" value={deviceInfo.device_name} />
                <Separator />
                <InfoRow icon={Fingerprint} label="Peer ID" value={deviceInfo.peer_id} mono />
                <Separator />
                <InfoRow
                  icon={Cpu}
                  label="操作系统"
                  value={`${deviceInfo.os} · ${deviceInfo.platform} · ${deviceInfo.arch}`}
                />
              </div>
            </div>
          </div>
        )}

        {/* Links */}
        <div className="rounded-xl border bg-card">
          <div className="px-5 py-4">
            <h3 className="text-sm font-medium">更多</h3>
          </div>
          <Separator />
          <div className="px-5 py-3">
            <a
              href="https://github.com"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center justify-between py-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
                  <Info className="h-4 w-4 text-muted-foreground" />
                </div>
                <span>开源仓库</span>
              </div>
              <span className="text-xs">GitHub &rarr;</span>
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/about")({
  component: AboutPage,
});
