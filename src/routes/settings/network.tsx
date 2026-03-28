import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { getDeviceInfo } from "@/commands/identity";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { type NodeStatus, useNetworkStore } from "@/stores/networkStore";
import { usePreferencesStore } from "@/stores/preferencesStore";

const statusConfig: Record<
  NodeStatus,
  { label: string; variant: "default" | "secondary" | "destructive" | "outline" }
> = {
  stopped: { label: "已停止", variant: "secondary" },
  starting: { label: "启动中...", variant: "outline" },
  running: { label: "运行中", variant: "default" },
  error: { label: "错误", variant: "destructive" },
};

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

  const { label, variant } = statusConfig[status];

  const handleStop = () => {
    setShowStopConfirm(true);
  };

  const confirmStop = async () => {
    setShowStopConfirm(false);
    await stopNode(true);
  };

  return (
    <div className="p-6">
      <h1 className="mb-1 text-lg font-semibold">网络</h1>
      <p className="mb-6 text-sm text-muted-foreground">P2P 节点状态和网络设置</p>

      <div className="space-y-6">
        {/* Node status */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">节点状态</div>
            <div className="text-xs text-muted-foreground">当前 P2P 节点运行状态</div>
          </div>
          <Badge variant={variant}>{label}</Badge>
        </div>

        {/* Error message */}
        {status === "error" && error && (
          <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
            {error}
          </div>
        )}

        {/* Peer ID */}
        {peerId && (
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">Peer ID</div>
              <div className="text-xs text-muted-foreground">本设备的 P2P 网络标识</div>
            </div>
            <code className="max-w-[240px] truncate rounded bg-muted px-2 py-1 text-xs">
              {peerId}
            </code>
          </div>
        )}

        {/* NAT status */}
        {status === "running" && natStatus && (
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">NAT 状态</div>
              <div className="text-xs text-muted-foreground">网络地址转换类型</div>
            </div>
            <span className="text-sm text-muted-foreground">{natStatus}</span>
          </div>
        )}

        {/* Connected peers */}
        {status === "running" && (
          <div className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">连接设备</div>
              <div className="text-xs text-muted-foreground">当前已连接的 P2P 设备数</div>
            </div>
            <span className="text-sm font-medium">{connectedPeers.length}</span>
          </div>
        )}

        {/* Start/Stop button */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">
              {status === "running" ? "停止节点" : "启动节点"}
            </div>
            <div className="text-xs text-muted-foreground">
              {status === "running" ? "停止后将断开所有 P2P 连接" : "启动 P2P 节点以同步笔记"}
            </div>
          </div>
          {status === "running" ? (
            <Button variant="destructive" size="sm" onClick={handleStop}>
              停止节点
            </Button>
          ) : (
            <Button size="sm" onClick={startNode} disabled={status === "starting"}>
              {status === "starting" ? "启动中..." : "启动节点"}
            </Button>
          )}
        </div>

        {/* Auto-start toggle */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">自动启动</div>
            <div className="text-xs text-muted-foreground">打开工作区时自动启动 P2P 节点</div>
          </div>
          <Switch checked={autoStartP2P} onCheckedChange={setAutoStartP2P} />
        </div>
      </div>

      {/* Stop confirmation dialog */}
      {showStopConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-[360px] rounded-lg border bg-background p-6 shadow-lg">
            <h3 className="mb-2 text-sm font-semibold">确认停止节点？</h3>
            <p className="mb-4 text-sm text-muted-foreground">
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
