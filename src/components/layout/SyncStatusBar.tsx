import { useLingui } from "@lingui/react/macro";

import { openSettingsWindow } from "@/commands/workspace";
import { useSyncDisplayState } from "@/hooks/useSyncDisplayState";
import { useNetworkStore } from "@/stores/networkStore";

interface SyncStatusBarProps {
  workspaceUuid: string | undefined;
}

export function SyncStatusBar({ workspaceUuid }: SyncStatusBarProps) {
  const { t } = useLingui();
  const nodeStatus = useNetworkStore((s) => s.status);
  const nodeLoading = useNetworkStore((s) => s.loading);
  const connectedPeerCount = useNetworkStore(
    (s) => s.devices.filter((d) => d.status === "online").length,
  );
  const syncState = useSyncDisplayState(workspaceUuid);

  let dotClass: string;
  let label: string;

  if (nodeLoading) {
    dotClass = "animate-pulse bg-yellow-500";
    label = t`连接中...`;
  } else if (nodeStatus === "error") {
    dotClass = "bg-red-500";
    label = t`连接失败`;
  } else if (nodeStatus !== "running") {
    dotClass = "bg-gray-400";
    label = t`网络未启动`;
  } else if (connectedPeerCount === 0) {
    dotClass = "bg-green-500";
    label = t`已连接`;
  } else if (syncState.status === "syncing") {
    dotClass = "bg-green-500";
    const peerCount = connectedPeerCount;
    label = t`${peerCount} 台 · 同步中 ${syncState.completed}/${syncState.total}`;
  } else if (syncState.status === "synced") {
    dotClass = "bg-green-500";
    const peerCount = connectedPeerCount;
    label = t`${peerCount} 台设备在线 · 已同步`;
  } else {
    dotClass = "bg-green-500";
    const peerCount = connectedPeerCount;
    label = peerCount > 0 ? t`${peerCount} 台设备在线` : t`已连接`;
  }

  return (
    <div className="border-t border-sidebar-border px-1 pt-2">
      <button
        type="button"
        className="flex w-full items-center gap-1.5 rounded-sm px-1 py-1 text-left hover:bg-sidebar-accent"
        onClick={() => openSettingsWindow("sync")}
      >
        <span className={`inline-block h-2 w-2 shrink-0 rounded-full ${dotClass}`} />
        <span className="truncate text-xs text-muted-foreground">{label}</span>
      </button>
    </div>
  );
}
