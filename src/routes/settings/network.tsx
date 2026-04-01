import { createFileRoute } from "@tanstack/react-router";
import { Fingerprint, Power, Shield, Users, Zap } from "lucide-react";
import { useEffect, useState } from "react";

import { getDeviceInfo } from "@/commands/identity";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { type NodeStatus, useNetworkStore } from "@/stores/networkStore";
import { usePreferencesStore } from "@/stores/preferencesStore";

const statusConfig: Record<
  NodeStatus,
  {
    label: string;
    description: string;
    variant: "default" | "secondary" | "destructive" | "outline";
    indicatorClass: string;
    iconClass: string;
    cardClass: string;
  }
> = {
  stopped: {
    label: "已停止",
    description: "P2P 节点未运行",
    variant: "secondary",
    indicatorClass: "bg-muted",
    iconClass: "text-muted-foreground",
    cardClass: "border-border",
  },
  starting: {
    label: "启动中...",
    description: "正在建立 P2P 连接",
    variant: "outline",
    indicatorClass: "bg-yellow-500 animate-pulse",
    iconClass: "text-white",
    cardClass: "border-yellow-500/30 bg-yellow-500/5",
  },
  running: {
    label: "运行中",
    description: "",
    variant: "default",
    indicatorClass: "bg-green-500",
    iconClass: "text-white",
    cardClass: "border-green-500/30 bg-green-500/5",
  },
  error: {
    label: "错误",
    description: "节点启动失败",
    variant: "destructive",
    indicatorClass: "bg-red-500",
    iconClass: "text-white",
    cardClass: "border-red-500/30 bg-red-500/5",
  },
};

function SettingCard({
  children,
  title,
  description,
  action,
}: {
  children: React.ReactNode;
  title: string;
  description?: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="rounded-xl border bg-card">
      <div className="flex items-center justify-between px-5 py-4">
        <div>
          <h3 className="text-sm font-medium">{title}</h3>
          {description && <p className="mt-0.5 text-xs text-muted-foreground">{description}</p>}
        </div>
        {action}
      </div>
      <Separator />
      <div className="px-5 py-3">{children}</div>
    </div>
  );
}

function SettingRow({
  icon: Icon,
  label,
  description,
  children,
}: {
  icon?: React.ComponentType<{ className?: string }>;
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex items-center gap-3">
        {Icon && (
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
            <Icon className="h-4 w-4 text-muted-foreground" />
          </div>
        )}
        <div>
          <div className="text-sm">{label}</div>
          {description && <div className="text-xs text-muted-foreground">{description}</div>}
        </div>
      </div>
      {children}
    </div>
  );
}

function NetworkSettingsPage() {
  const status = useNetworkStore((s) => s.status);
  const error = useNetworkStore((s) => s.error);
  const connectedPeers = useNetworkStore((s) => s.connectedPeers);
  const natStatus = useNetworkStore((s) => s.natStatus);
  const startNode = useNetworkStore((s) => s.startNode);
  const stopNode = useNetworkStore((s) => s.stopNode);

  const autoStartP2P = usePreferencesStore((s) => s.autoStartP2P);
  const setAutoStartP2P = usePreferencesStore((s) => s.setAutoStartP2P);

  const [peerId, setPeerId] = useState<string | null>(null);
  const [showStopConfirm, setShowStopConfirm] = useState(false);

  useEffect(() => {
    getDeviceInfo().then((info) => setPeerId(info.peer_id));
  }, []);

  const { label, variant, description, indicatorClass, iconClass, cardClass } =
    statusConfig[status];

  const handleStop = () => {
    setShowStopConfirm(true);
  };

  const confirmStop = async () => {
    setShowStopConfirm(false);
    await stopNode(true);
  };

  const statusDescription =
    status === "running"
      ? connectedPeers.length > 0
        ? `已连接 ${connectedPeers.length} 台设备`
        : "已连接，暂无设备在线"
      : status === "error"
        ? error || description
        : description;

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold tracking-tight">网络</h1>
        <p className="mt-1 text-sm text-muted-foreground">P2P 节点状态和网络设置</p>
      </div>

      <div className="space-y-4">
        {/* Network Power Control — prominent status + toggle */}
        <div className={cn("rounded-xl border-2 px-5 py-4", cardClass)}>
          <div className="flex items-center gap-4">
            <div
              className={cn(
                "flex h-11 w-11 shrink-0 items-center justify-center rounded-xl",
                indicatorClass,
              )}
            >
              <Power className={cn("h-5 w-5", iconClass)} />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="font-medium">{label}</span>
                <Badge variant={variant} className="text-[10px]">
                  P2P
                </Badge>
              </div>
              <p className="mt-0.5 text-xs text-muted-foreground">{statusDescription}</p>
            </div>
            {status === "running" ? (
              <Button variant="outline" size="sm" onClick={handleStop} className="shrink-0">
                停止节点
              </Button>
            ) : (
              <Button
                size="sm"
                onClick={startNode}
                disabled={status === "starting"}
                className="shrink-0"
              >
                {status === "starting" ? "启动中..." : "启动节点"}
              </Button>
            )}
          </div>
        </div>

        {/* Node Status Card */}
        <SettingCard title="节点信息" description="当前 P2P 节点详情">
          <div className="space-y-1">
            {/* Peer ID */}
            {peerId && (
              <SettingRow icon={Fingerprint} label="Peer ID" description="本设备的 P2P 网络标识">
                <code className="max-w-50 truncate rounded-md bg-muted px-2.5 py-1 text-xs">
                  {peerId}
                </code>
              </SettingRow>
            )}

            {/* NAT status */}
            {status === "running" && natStatus && (
              <>
                {peerId && <Separator />}
                <SettingRow icon={Shield} label="NAT 状态" description="网络地址转换类型">
                  <span className="text-sm text-muted-foreground">{natStatus}</span>
                </SettingRow>
              </>
            )}

            {/* Connected peers */}
            {status === "running" && (
              <>
                {(peerId || natStatus) && <Separator />}
                <SettingRow icon={Users} label="连接设备" description="当前已连接的 P2P 设备数">
                  <span className="rounded-md bg-muted px-2.5 py-1 text-sm font-medium">
                    {connectedPeers.length}
                  </span>
                </SettingRow>
              </>
            )}

            {!peerId && status === "stopped" && (
              <p className="py-2 text-center text-xs text-muted-foreground">
                启动节点后显示详细信息
              </p>
            )}
          </div>
        </SettingCard>

        {/* Auto-start Setting */}
        <SettingCard title="启动设置">
          <SettingRow icon={Zap} label="自动启动" description="打开工作区时自动启动 P2P 节点">
            <Switch checked={autoStartP2P} onCheckedChange={setAutoStartP2P} />
          </SettingRow>
        </SettingCard>
      </div>

      {/* Stop confirmation dialog */}
      {showStopConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-90 rounded-xl border bg-background p-6 shadow-lg">
            <h3 className="mb-2 text-sm font-semibold">确认停止节点？</h3>
            <p className="mb-5 text-sm text-muted-foreground">
              停止 P2P 节点将断开与所有设备的连接，笔记将停止同步。
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="outline" size="sm" onClick={() => setShowStopConfirm(false)}>
                取消
              </Button>
              <Button variant="destructive" size="sm" onClick={confirmStop}>
                确认停止
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export const Route = createFileRoute("/settings/network")({
  component: NetworkSettingsPage,
});
