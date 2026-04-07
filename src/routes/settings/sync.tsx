import { Trans, useLingui } from "@lingui/react/macro";
import { createFileRoute } from "@tanstack/react-router";
import { Zap } from "lucide-react";
import { NetworkStatusCard } from "@/components/settings/NetworkStatusCard";
import { SettingRow } from "@/components/settings/SettingRow";
import { WorkspaceSyncList } from "@/components/settings/WorkspaceSyncList";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { usePreferencesStore } from "@/stores/preferencesStore";

function SyncSettingsPage() {
  const { t } = useLingui();
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
        <Card>
          <CardHeader className="border-b">
            <CardTitle>
              <Trans>P2P 网络</Trans>
            </CardTitle>
            <CardDescription>
              <Trans>节点运行状态与连接信息</Trans>
            </CardDescription>
          </CardHeader>
          <CardContent>
            <NetworkStatusCard />
          </CardContent>
        </Card>

        {/* Workspace Sync List */}
        <WorkspaceSyncList />

        {/* Auto-start Setting */}
        <Card>
          <CardContent>
            <SettingRow
              icon={Zap}
              label={t`开机自动启动网络`}
              description={t`打开工作区时自动启动 P2P 节点`}
            >
              <Switch checked={autoStartP2P} onCheckedChange={setAutoStartP2P} />
            </SettingRow>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/sync")({
  component: SyncSettingsPage,
});
