import { Trans } from "@lingui/react/macro";
import { createFileRoute } from "@tanstack/react-router";
import { Zap } from "lucide-react";
import { NetworkStatusCard } from "@/components/settings/NetworkStatusCard";
import { WorkspaceSyncList } from "@/components/settings/WorkspaceSyncList";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { usePreferencesStore } from "@/stores/preferencesStore";

function SyncSettingsPage() {
  const autoStartP2P = usePreferencesStore((s) => s.autoStartP2P);
  const setAutoStartP2P = usePreferencesStore((s) => s.setAutoStartP2P);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold tracking-tight">
          <Trans>同步</Trans>
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          <Trans>P2P 网络状态与工作区同步</Trans>
        </p>
      </div>

      <div className="space-y-4">
        {/* P2P Network Status */}
        <div className="rounded-xl border bg-card">
          <div className="px-5 py-4">
            <h3 className="text-sm font-medium">
              <Trans>P2P 网络</Trans>
            </h3>
            <p className="mt-0.5 text-xs text-muted-foreground">
              <Trans>节点运行状态与连接信息</Trans>
            </p>
          </div>
          <Separator />
          <div className="px-5 py-4">
            <NetworkStatusCard />
          </div>
        </div>

        {/* Workspace Sync List */}
        <WorkspaceSyncList />

        {/* Auto-start Setting */}
        <div className="rounded-xl border bg-card">
          <div className="px-5 py-3">
            <div className="flex items-center justify-between py-2">
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
                  <Zap className="h-4 w-4 text-muted-foreground" />
                </div>
                <div>
                  <div className="text-sm">
                    <Trans>开机自动启动网络</Trans>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    <Trans>打开工作区时自动启动 P2P 节点</Trans>
                  </div>
                </div>
              </div>
              <Switch checked={autoStartP2P} onCheckedChange={setAutoStartP2P} />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/sync")({
  component: SyncSettingsPage,
});
