import { useShallow } from "zustand/react/shallow";
import { useSyncStore } from "@/stores/syncStore";

export type SyncDisplayStatus = "syncing" | "synced" | "local-only";

export interface SyncDisplayState {
  status: SyncDisplayStatus;
  completed?: number;
  total?: number;
  lastSyncedAt?: number;
}

/**
 * 聚合指定工作区的同步展示状态。
 * 使用 useShallow 避免无关工作区的 sync-progress 事件导致重渲染。
 */
export function useSyncDisplayState(workspaceUuid: string | undefined): SyncDisplayState {
  return useSyncStore(
    useShallow((s): SyncDisplayState => {
      if (!workspaceUuid) return { status: "local-only" };

      const wsActiveSyncs = Object.values(s.activeSyncs).filter(
        (a) => a.workspaceUuid === workspaceUuid,
      );

      if (wsActiveSyncs.length > 0) {
        const completed = wsActiveSyncs.reduce((sum, a) => sum + a.completed, 0);
        const total = wsActiveSyncs.reduce((sum, a) => sum + a.total, 0);
        return { status: "syncing", completed, total };
      }

      const lastResult = s.lastSyncResults[workspaceUuid];
      if (lastResult) {
        return { status: "synced", lastSyncedAt: lastResult.lastSyncedAt };
      }

      return { status: "local-only" };
    }),
  );
}
