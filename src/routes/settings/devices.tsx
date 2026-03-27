import { createFileRoute } from "@tanstack/react-router";
import { Loader2, RefreshCw } from "lucide-react";
import { useEffect } from "react";
import { CodePairingCard } from "@/components/pairing/CodePairingCard";
import { NearbyDeviceCard } from "@/components/pairing/NearbyDeviceCard";
import { PairedDeviceCard } from "@/components/pairing/PairedDeviceCard";
import { Button } from "@/components/ui/button";
import { setupPairingListeners, usePairingStore } from "@/stores/pairingStore";

function DevicesPage() {
  const pairedDevices = usePairingStore((s) => s.pairedDevices);
  const nearbyDevices = usePairingStore((s) => s.nearbyDevices);
  const isLoading = usePairingStore((s) => s.isLoading);
  const refresh = usePairingStore((s) => s.refresh);

  useEffect(() => {
    setupPairingListeners();
    refresh();
  }, [refresh]);

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="mb-1 text-lg font-semibold">设备管理</h1>
          <p className="text-sm text-muted-foreground">管理已配对设备和发现附近设备</p>
        </div>
        <Button variant="ghost" size="icon-sm" onClick={refresh} disabled={isLoading}>
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
        </Button>
      </div>

      {/* Paired Devices */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-medium text-muted-foreground">已配对设备</h2>
        {pairedDevices.length > 0 ? (
          <div className="space-y-2">
            {pairedDevices.map((device) => (
              <PairedDeviceCard key={device.peerId} device={device} onUnpaired={refresh} />
            ))}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">暂无已配对设备</p>
        )}
      </section>

      {/* Nearby Devices */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-medium text-muted-foreground">附近设备</h2>
        {nearbyDevices.length > 0 ? (
          <div className="space-y-2">
            {nearbyDevices.map((device) => (
              <NearbyDeviceCard key={device.peer_id} device={device} onPaired={refresh} />
            ))}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">未发现附近设备</p>
        )}
      </section>

      {/* Code Pairing */}
      <section>
        <CodePairingCard />
      </section>
    </div>
  );
}

export const Route = createFileRoute("/settings/devices")({
  component: DevicesPage,
});
