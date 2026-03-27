import { Monitor } from "lucide-react";
import { useState } from "react";
import type { PeerInfo } from "@/commands/pairing";
import { requestPairing } from "@/commands/pairing";
import { Button } from "@/components/ui/button";

interface NearbyDeviceCardProps {
  device: PeerInfo;
  onPaired?: () => void;
}

export function NearbyDeviceCard({ device, onPaired }: NearbyDeviceCardProps) {
  const [pairing, setPairing] = useState(false);

  async function handlePair() {
    setPairing(true);
    try {
      const resp = await requestPairing(device.peer_id, { type: "Direct" });
      if (resp.status === "Success") {
        onPaired?.();
      }
    } catch (e) {
      console.error("Failed to pair device:", e);
    } finally {
      setPairing(false);
    }
  }

  return (
    <div className="flex items-center justify-between rounded-lg border p-4">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
          <Monitor className="h-5 w-5 text-muted-foreground" />
        </div>
        <div>
          <div className="text-sm font-medium">{device.hostname}</div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span>
              {device.os} · {device.platform}
            </span>
            {device.rtt_ms != null ? <span>· {device.rtt_ms}ms</span> : null}
            {device.connection_type ? <span>· {device.connection_type}</span> : null}
          </div>
        </div>
      </div>

      <Button
        size="sm"
        className="bg-indigo-600 text-white hover:bg-indigo-700"
        onClick={handlePair}
        disabled={pairing}
      >
        {pairing ? "配对中..." : "配对"}
      </Button>
    </div>
  );
}
