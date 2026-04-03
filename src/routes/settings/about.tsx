import { Trans, useLingui } from "@lingui/react/macro";
import { createFileRoute } from "@tanstack/react-router";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  CheckCircle2,
  Cpu,
  Download,
  Fingerprint,
  Info,
  Loader2,
  Monitor,
  RefreshCw,
} from "lucide-react";
import { useEffect, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import type { DeviceInfo } from "@/commands/identity";
import { getDeviceInfo } from "@/commands/identity";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useUpgradeStore } from "@/stores/upgradeStore";

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
  const { t } = useLingui();
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [appVersion, setAppVersion] = useState<string | null>(null);

  const { status, latestVersion, currentVersion, progress, checkForUpdate, startDownload } =
    useUpgradeStore(
      useShallow((s) => ({
        status: s.status,
        latestVersion: s.latestVersion,
        currentVersion: s.currentVersion,
        progress: s.progress,
        checkForUpdate: s.checkForUpdate,
        startDownload: s.startDownload,
      })),
    );

  useEffect(() => {
    getDeviceInfo().then(setDeviceInfo).catch(console.error);
    getVersion().then(setAppVersion).catch(console.error);
  }, []);

  const displayVersion = currentVersion ?? appVersion;

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold tracking-tight">
          <Trans>关于</Trans>
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          <Trans>SwarmNote 版本与设备信息</Trans>
        </p>
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
              <p className="text-xs text-muted-foreground">
                <UpdateStatusText status={status} version={displayVersion} />
              </p>
            </div>
            <div className="ml-auto flex items-center gap-3">
              {displayVersion && (
                <span className="rounded-full bg-muted px-3 py-1 text-xs font-medium">
                  v{displayVersion}
                </span>
              )}
              <UpdateButton
                status={status}
                latestVersion={latestVersion}
                onCheck={() => checkForUpdate(true)}
                onUpdate={startDownload}
              />
            </div>
          </div>

          {/* 下载进度 */}
          {(status === "downloading" || status === "ready") && progress && (
            <>
              <Separator />
              <div className="px-5 py-4 space-y-2">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-muted-foreground">
                    <Trans>正在下载 v{latestVersion ?? "?"}</Trans>
                  </span>
                  <span className="font-medium text-primary">{progress.percent}%</span>
                </div>
                <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                  <div
                    className="h-full rounded-full bg-primary transition-all duration-300"
                    style={{ width: `${progress.percent}%` }}
                  />
                </div>
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>
                    {formatBytes(progress.downloaded)} / {formatBytes(progress.total)}
                  </span>
                  <span>{formatBytes(progress.speed)}/s</span>
                </div>
              </div>
            </>
          )}
        </div>

        {/* Device Info */}
        {deviceInfo && (
          <div className="rounded-xl border bg-card">
            <div className="px-5 py-4">
              <h3 className="text-sm font-medium">
                <Trans>设备信息</Trans>
              </h3>
              <p className="mt-0.5 text-xs text-muted-foreground">
                <Trans>当前运行设备的详细信息</Trans>
              </p>
            </div>
            <Separator />
            <div className="px-5 py-3">
              <div className="space-y-1">
                <InfoRow icon={Monitor} label={t`设备名称`} value={deviceInfo.device_name} />
                <Separator />
                <InfoRow icon={Fingerprint} label="Peer ID" value={deviceInfo.peer_id} mono />
                <Separator />
                <InfoRow
                  icon={Cpu}
                  label={t`操作系统`}
                  value={`${deviceInfo.os} · ${deviceInfo.platform} · ${deviceInfo.arch}`}
                />
              </div>
            </div>
          </div>
        )}

        {/* Links */}
        <div className="rounded-xl border bg-card">
          <div className="px-5 py-4">
            <h3 className="text-sm font-medium">
              <Trans>更多</Trans>
            </h3>
          </div>
          <Separator />
          <div className="px-5 py-3">
            <button
              type="button"
              onClick={() => openUrl("https://github.com/yexiyue/SwarmNote")}
              className="flex w-full items-center justify-between py-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
                  <Info className="h-4 w-4 text-muted-foreground" />
                </div>
                <span>
                  <Trans>开源仓库</Trans>
                </span>
              </div>
              <span className="text-xs">GitHub →</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function UpdateStatusText({ status }: { status: string }) {
  switch (status) {
    case "checking":
      return <Trans>检查更新中...</Trans>;
    case "available":
      return <Trans>有新版本可用</Trans>;
    case "force-required":
      return <Trans>需要强制更新</Trans>;
    case "downloading":
      return <Trans>正在更新...</Trans>;
    case "up-to-date":
      return <Trans>已是最新版本</Trans>;
    case "error":
      return <Trans>检查更新失败</Trans>;
    default:
      return <Trans>去中心化 P2P 笔记应用</Trans>;
  }
}

function UpdateButton({
  status,
  latestVersion,
  onCheck,
  onUpdate,
}: {
  status: string;
  latestVersion: string | null;
  onCheck: () => void;
  onUpdate: () => void;
}) {
  switch (status) {
    case "checking":
      return (
        <Button variant="outline" size="sm" disabled>
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
          <Trans>检查中...</Trans>
        </Button>
      );
    case "available":
      return (
        <Button size="sm" onClick={onUpdate}>
          <Download className="h-3.5 w-3.5" />
          <Trans>更新到 v{latestVersion ?? "?"}</Trans>
        </Button>
      );
    case "force-required":
    case "downloading":
    case "ready":
      return (
        <Button size="sm" disabled>
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
          <Trans>下载中...</Trans>
        </Button>
      );
    case "up-to-date":
      return (
        <Button variant="outline" size="sm" onClick={onCheck}>
          <CheckCircle2 className="h-3.5 w-3.5 text-green-500" />
          <Trans>已是最新</Trans>
        </Button>
      );
    default:
      return (
        <Button variant="outline" size="sm" onClick={onCheck}>
          <RefreshCw className="h-3.5 w-3.5" />
          <Trans>检查更新</Trans>
        </Button>
      );
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export const Route = createFileRoute("/settings/about")({
  component: AboutPage,
});
