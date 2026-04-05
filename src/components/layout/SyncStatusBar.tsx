import { useLingui } from "@lingui/react/macro";

import { openSettingsWindow } from "@/commands/workspace";
import { useSyncDisplayState } from "@/hooks/useSyncDisplayState";
import { computeSyncDisplay, type SyncDisplayLabel, syncDotClass } from "@/lib/syncDisplay";
import { useNetworkStore } from "@/stores/networkStore";

interface SyncStatusBarProps {
  workspaceUuid: string | undefined;
}

/**
 * Turn a structured `SyncDisplayLabel` into a localized human-readable string.
 * Keeping the translation here (rather than in `computeSyncDisplay`) preserves
 * the purity of the state machine while still participating in Lingui i18n.
 */
function useSyncLabelText(label: SyncDisplayLabel): string {
  const { t } = useLingui();
  switch (label.kind) {
    case "connecting":
      return t`连接中...`;
    case "error":
      return t`连接失败`;
    case "stopped":
      return t`网络未启动`;
    case "waiting-for-peers":
      return t`等待设备连接`;
    case "syncing": {
      const { peerCount, completed, total } = label;
      return t`${peerCount} 台 · 同步中 ${completed}/${total}`;
    }
    case "synced": {
      const { peerCount } = label;
      return t`${peerCount} 台设备在线 · 已同步`;
    }
    case "online": {
      const { peerCount } = label;
      return t`${peerCount} 台设备在线`;
    }
  }
}

export function SyncStatusBar({ workspaceUuid }: SyncStatusBarProps) {
  const nodeStatus = useNetworkStore((s) => s.status);
  const nodeLoading = useNetworkStore((s) => s.loading);
  const connectedPeerCount = useNetworkStore(
    (s) => s.devices.filter((d) => d.status === "online").length,
  );
  const syncState = useSyncDisplayState(workspaceUuid);

  const { dot, label } = computeSyncDisplay({
    nodeLoading,
    nodeStatus,
    connectedPeerCount,
    syncState,
  });

  const labelText = useSyncLabelText(label);

  return (
    <div className="border-t border-sidebar-border px-1 pt-2">
      <button
        type="button"
        className="flex w-full items-center gap-1.5 rounded-sm px-1 py-1 text-left hover:bg-sidebar-accent"
        onClick={() => openSettingsWindow("sync")}
      >
        <span className={`inline-block h-2 w-2 shrink-0 rounded-full ${syncDotClass(dot)}`} />
        <span className="truncate text-xs text-muted-foreground">{labelText}</span>
      </button>
    </div>
  );
}
