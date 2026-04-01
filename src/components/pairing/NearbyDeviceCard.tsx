import type { Device } from "@/commands/pairing";
import { requestPairing } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import { useAsyncAction } from "@/hooks/useAsyncAction";
import { ConnectionBadge } from "./ConnectionBadge";
import { DeviceInfoCard } from "./DeviceInfoCard";

interface NearbyDeviceCardProps {
  device: Device;
  onPaired?: () => void;
}

export function NearbyDeviceCard({ device, onPaired }: NearbyDeviceCardProps) {
  const { loading, run } = useAsyncAction();

  async function handlePair() {
    await run(async () => {
      const resp = await requestPairing(device.peerId, { type: "Direct" });
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
        {device.connection && <ConnectionBadge type={device.connection} latency={device.latency} />}
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
