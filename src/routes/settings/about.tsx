import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import type { DeviceInfo } from "@/commands/identity";
import { getDeviceInfo } from "@/commands/identity";

function AboutPage() {
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);

  useEffect(() => {
    getDeviceInfo().then(setDeviceInfo).catch(console.error);
  }, []);

  return (
    <div className="p-6">
      <h1 className="mb-1 text-lg font-semibold">关于</h1>
      <p className="mb-6 text-sm text-muted-foreground">SwarmNote 版本与设备信息</p>

      <div className="space-y-4">
        <div>
          <div className="text-sm font-medium">版本</div>
          <div className="text-sm text-muted-foreground">v0.2.0</div>
        </div>
        {deviceInfo ? (
          <>
            <div>
              <div className="text-sm font-medium">设备名称</div>
              <div className="text-sm text-muted-foreground">{deviceInfo.device_name}</div>
            </div>
            <div>
              <div className="text-sm font-medium">Peer ID</div>
              <div className="break-all font-mono text-xs text-muted-foreground">
                {deviceInfo.peer_id}
              </div>
            </div>
            <div>
              <div className="text-sm font-medium">操作系统</div>
              <div className="text-sm text-muted-foreground">
                {deviceInfo.os} · {deviceInfo.platform} · {deviceInfo.arch}
              </div>
            </div>
          </>
        ) : null}
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/about")({
  component: AboutPage,
});
