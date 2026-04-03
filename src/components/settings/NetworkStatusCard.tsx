import { Trans, useLingui } from "@lingui/react/macro";
import { Power } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { type NodeStatus, useNetworkStore } from "@/stores/networkStore";

const statusConfig: Record<
  NodeStatus,
  {
    variant: "default" | "secondary" | "destructive" | "outline";
    indicatorClass: string;
    iconClass: string;
    cardClass: string;
  }
> = {
  stopped: {
    variant: "secondary",
    indicatorClass: "bg-muted-foreground/40",
    iconClass: "text-muted-foreground",
    cardClass: "border-border",
  },
  running: {
    variant: "default",
    indicatorClass: "bg-green-500",
    iconClass: "text-white",
    cardClass: "border-green-500/30 bg-green-500/5",
  },
  error: {
    variant: "destructive",
    indicatorClass: "bg-red-500",
    iconClass: "text-white",
    cardClass: "border-red-500/30 bg-red-500/5",
  },
};

export function NetworkStatusCard() {
  const { t } = useLingui();
  const status = useNetworkStore((s) => s.status);
  const error = useNetworkStore((s) => s.error);
  const loading = useNetworkStore((s) => s.loading);
  const devices = useNetworkStore((s) => s.devices);
  const connectedCount = devices.filter((d) => d.status === "online").length;
  const natStatus = useNetworkStore((s) => s.natStatus);
  const startNode = useNetworkStore((s) => s.startNode);
  const stopNode = useNetworkStore((s) => s.stopNode);

  const [showStopConfirm, setShowStopConfirm] = useState(false);

  const statusLabels: Record<NodeStatus, string> = {
    stopped: t`已停止`,
    running: t`运行中`,
    error: t`错误`,
  };

  const { variant, indicatorClass, iconClass, cardClass } = statusConfig[status];
  const label = statusLabels[status];

  const natSuffix = natStatus ? ` · ${natStatus}` : "";
  const statusDescription =
    status === "running"
      ? connectedCount > 0
        ? `${t`已连接 ${connectedCount} 台设备`}${natSuffix}`
        : `${t`已连接，暂无设备在线`}${natSuffix}`
      : status === "error"
        ? error || t`节点启动失败`
        : t`P2P 节点未运行`;

  const confirmStop = async () => {
    setShowStopConfirm(false);
    await stopNode(true);
  };

  return (
    <>
      <div className={cn("rounded-xl border-2 px-5 py-4", cardClass)}>
        <div className="flex items-center gap-4">
          <div
            className={cn(
              "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl",
              indicatorClass,
            )}
          >
            <Power className={cn("h-4.5 w-4.5", iconClass)} />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">{label}</span>
              <Badge variant={variant} className="text-[10px]">
                P2P
              </Badge>
            </div>
            <p className="mt-0.5 text-xs text-muted-foreground">{statusDescription}</p>
          </div>
          {status === "running" ? (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowStopConfirm(true)}
              disabled={loading}
              className="shrink-0"
            >
              <Trans>关闭网络</Trans>
            </Button>
          ) : (
            <Button size="sm" onClick={startNode} loading={loading} className="shrink-0">
              {loading ? <Trans>启动中...</Trans> : <Trans>启动网络</Trans>}
            </Button>
          )}
        </div>
      </div>

      {showStopConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-90 rounded-xl border bg-background p-6 shadow-lg">
            <h3 className="mb-2 text-sm font-semibold">
              <Trans>确认关闭网络？</Trans>
            </h3>
            <p className="mb-5 text-sm text-muted-foreground">
              <Trans>关闭 P2P 网络将断开与所有设备的连接，笔记将停止同步。</Trans>
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="outline" size="sm" onClick={() => setShowStopConfirm(false)}>
                <Trans>取消</Trans>
              </Button>
              <Button variant="destructive" size="sm" onClick={confirmStop}>
                <Trans>确认关闭</Trans>
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
