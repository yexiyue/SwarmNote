import type { PeerInfo } from "@/commands/pairing";
import { requestPairing } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import { useAsyncAction } from "@/hooks/useAsyncAction";
import { DeviceInfoCard } from "./DeviceInfoCard";

interface NearbyDeviceCardProps {
  device: PeerInfo;
  onPaired?: () => void;
}

export function NearbyDeviceCard({ device, onPaired }: NearbyDeviceCardProps) {
  const { loading, run } = useAsyncAction();

  async function handlePair() {
    await run(async () => {
      const resp = await requestPairing(device.peer_id, { type: "Direct" });
      if (resp.status === "Success") {
        onPaired?.();
      }
    });
  }

  return (
    <div className="flex items-center justify-between rounded-lg border p-4">
      <DeviceInfoCard
        hostname={device.hostname}
        os={device.os}
        platform={device.platform}
        className="border-0 p-0"
      >
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          {device.rtt_ms != null ? <span>{device.rtt_ms}ms</span> : null}
          {device.connection_type ? <span>· {device.connection_type}</span> : null}
        </div>
      </DeviceInfoCard>

      <Button
        size="sm"
        className="bg-indigo-600 text-white hover:bg-indigo-700"
        onClick={handlePair}
        loading={loading}
      >
        {loading ? "配对中..." : "配对"}
      </Button>
    </div>
  );
}
