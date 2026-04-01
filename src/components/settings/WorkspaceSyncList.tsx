import { Trans } from "@lingui/react/macro";
import { CheckCircle2, Clock, FolderSync, WifiOff } from "lucide-react";
import { useEffect, useState } from "react";
import type { RecentWorkspace } from "@/commands/workspace";
import { getRecentWorkspaces } from "@/commands/workspace";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import { useNetworkStore } from "@/stores/networkStore";

function WorkspaceSyncItem({ workspace }: { workspace: RecentWorkspace }) {
  return (
    <div className="flex items-center gap-3 py-2.5">
      <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-muted">
        <FolderSync className="h-4 w-4 text-muted-foreground" />
      </div>
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-medium">{workspace.name}</div>
        <div className="mt-0.5 flex items-center gap-1.5 text-xs text-muted-foreground">
          <CheckCircle2 className="h-3 w-3 text-muted-foreground/60" />
          <span>
            <Trans>同步未启用</Trans>
          </span>
        </div>
      </div>
      <div className="flex items-center gap-1 text-xs text-muted-foreground">
        <Clock className="h-3 w-3" />
        <span className="text-[11px]">—</span>
      </div>
    </div>
  );
}

export function WorkspaceSyncList() {
  const status = useNetworkStore((s) => s.status);
  const [recentWorkspaces, setRecentWorkspaces] = useState<RecentWorkspace[]>([]);

  useEffect(() => {
    getRecentWorkspaces()
      .then(setRecentWorkspaces)
      .catch(() => null);
  }, []);

  return (
    <div className="rounded-xl border bg-card">
      <div className="px-5 py-4">
        <h3 className="text-sm font-medium">
          <Trans>工作区同步</Trans>
        </h3>
        <p className="mt-0.5 text-xs text-muted-foreground">
          <Trans>各工作区的同步状态</Trans>
        </p>
      </div>
      <Separator />
      <div className="px-5 py-3">
        {status !== "running" ? (
          <div className="flex flex-col items-center gap-2 py-4">
            <WifiOff className="h-8 w-8 text-muted-foreground/40" />
            <p className="text-center text-sm text-muted-foreground">
              <Trans>启动 P2P 网络后即可同步工作区</Trans>
            </p>
          </div>
        ) : recentWorkspaces.length === 0 ? (
          <p className="py-3 text-center text-xs text-muted-foreground">
            <Trans>暂无工作区</Trans>
          </p>
        ) : (
          <div className={cn("space-y-0", recentWorkspaces.length > 1 && "divide-y divide-border")}>
            {recentWorkspaces.map((ws) => (
              <WorkspaceSyncItem key={ws.path} workspace={ws} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
