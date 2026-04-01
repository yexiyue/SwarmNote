import { createFileRoute } from "@tanstack/react-router";
import { Loader2, Monitor, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { type DeviceInfo, getDeviceInfo } from "@/commands/identity";
import { CodePairingCard } from "@/components/pairing/CodePairingCard";
import { NearbyDeviceCard } from "@/components/pairing/NearbyDeviceCard";
import { PairedDeviceCard } from "@/components/pairing/PairedDeviceCard";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { setupPairingListeners, usePairingStore } from "@/stores/pairingStore";

function DevicesPage() {
  const pairedDevices = usePairingStore((s) => s.pairedDevices);
  const nearbyDevices = usePairingStore((s) => s.nearbyDevices);
  const isLoading = usePairingStore((s) => s.isLoading);
  const refresh = usePairingStore((s) => s.refresh);
  const [myDevice, setMyDevice] = useState<DeviceInfo | null>(null);

  useEffect(() => {
    setupPairingListeners();
    refresh();
  }, [refresh]);

  useEffect(() => {
    getDeviceInfo()
      .then(setMyDevice)
      .catch(() => null);
  }, []);

  return (
    <div>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">设备管理</h1>
          <p className="mt-1 text-sm text-muted-foreground">管理已配对设备和发现附近设备</p>
        </div>
        <Button variant="outline" size="icon-sm" onClick={refresh} disabled={isLoading}>
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
        </Button>
      </div>

      <div className="space-y-4">
        {/* My Device Card */}
        <div className="rounded-xl border bg-card">
          <div className="flex items-center gap-4 px-5 py-4">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-primary/10">
              <Monitor className="h-5.5 w-5.5 text-primary" />
            </div>
            <div className="min-w-0 flex-1">
              <div className="truncate font-medium text-foreground">
                {myDevice?.device_name ?? "—"}
              </div>
              <div className="mt-0.5 text-xs text-muted-foreground">
                {myDevice ? `${myDevice.os} · ${myDevice.platform}` : "—"}
              </div>
            </div>
            {myDevice && (
              <code className="shrink-0 rounded-md bg-muted px-2.5 py-1 text-xs text-muted-foreground">
                {myDevice.peer_id.slice(0, 20)}…
              </code>
            )}
          </div>
        </div>
        {/* Paired Devices */}
        <div className="rounded-xl border bg-card">
          <div className="flex items-center justify-between px-5 py-3">
            <h3 className="text-sm font-medium">已配对设备</h3>
            <span className="text-xs text-muted-foreground">
              {pairedDevices.length > 0 ? `${pairedDevices.length} 台设备` : "无"}
            </span>
          </div>
          <Separator />
          <div className="px-5 py-3">
            {pairedDevices.length > 0 ? (
              <div className="space-y-2">
                {pairedDevices.map((device) => (
                  <PairedDeviceCard key={device.peerId} device={device} onUnpaired={refresh} />
                ))}
              </div>
            ) : (
              <p className="py-2 text-center text-xs text-muted-foreground">
                通过配对码连接其他设备
              </p>
            )}
          </div>
        </div>

        {/* Nearby Devices */}
        <div className="rounded-xl border bg-card">
          <div className="flex items-center justify-between px-5 py-3">
            <h3 className="text-sm font-medium">附近设备</h3>
            <span className="text-xs text-muted-foreground">
              {nearbyDevices.length > 0 ? `${nearbyDevices.length} 台设备` : "无"}
            </span>
          </div>
          <Separator />
          <div className="px-5 py-3">
            {nearbyDevices.length > 0 ? (
              <div className="space-y-2">
                {nearbyDevices.map((device) => (
                  <NearbyDeviceCard key={device.peerId} device={device} onPaired={refresh} />
                ))}
              </div>
            ) : (
              <p className="py-2 text-center text-xs text-muted-foreground">
                在局域网中发现可配对设备
              </p>
            )}
          </div>
        </div>

        {/* Code Pairing */}
        <div className="rounded-xl border bg-card">
          <div className="px-5 py-3">
            <h3 className="text-sm font-medium">配对码连接</h3>
            <p className="mt-0.5 text-xs text-muted-foreground">使用配对码与远程设备配对</p>
          </div>
          <Separator />
          <div className="px-5 py-3">
            <CodePairingCard />
          </div>
        </div>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/devices")({
  component: DevicesPage,
});
