import { Loader2, Unlink } from "lucide-react";
import type { PairedDeviceInfo } from "@/commands/pairing";
import { unpairDevice } from "@/commands/pairing";
import { ConnectionBadge } from "@/components/pairing/ConnectionBadge";
import { Button } from "@/components/ui/button";
import { useAsyncAction } from "@/hooks/useAsyncAction";
import { formatDate, formatRelativeTime } from "@/lib/dateUtils";
import { cn } from "@/lib/utils";
import { DeviceInfoCard } from "./DeviceInfoCard";

interface PairedDeviceCardProps {
  device: PairedDeviceInfo;
  onUnpaired?: () => void;
}

export function PairedDeviceCard({ device, onUnpaired }: PairedDeviceCardProps) {
  const { loading, run } = useAsyncAction();
  const isOnline = device.isOnline ?? false;

  async function handleUnpair() {
    await run(async () => {
      await unpairDevice(device.peerId);
      onUnpaired?.();
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
        <div className="flex items-center gap-2">
          <span
            className={cn(
              "inline-block h-2 w-2 rounded-full",
              isOnline ? "bg-green-500" : "bg-muted-foreground/30",
            )}
          />
          {isOnline && device.connection ? (
            <ConnectionBadge type={device.connection} latency={device.rttMs} />
          ) : (
            device.rttMs != null && (
              <span className="text-xs text-muted-foreground">{device.rttMs}ms</span>
            )
          )}
        </div>
        <div className="text-xs text-muted-foreground">
          {isOnline
            ? `配对于 ${formatDate(device.pairedAt)}`
            : `最后在线 ${formatRelativeTime(device.lastSeen)}`}
        </div>
      </DeviceInfoCard>

      <Button
        variant="ghost"
        size="icon-sm"
        onClick={handleUnpair}
        disabled={loading}
        className="shrink-0 text-muted-foreground hover:text-destructive"
        title="取消配对"
      >
        {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Unlink className="h-4 w-4" />}
      </Button>
    </div>
  );
}
