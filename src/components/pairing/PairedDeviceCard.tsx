import { MoreHorizontal } from "lucide-react";
import type { PairedDeviceInfo } from "@/commands/pairing";
import { unpairDevice } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
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
              isOnline ? "bg-green-500" : "bg-gray-300",
            )}
          />
          <span className="text-xs text-muted-foreground">
            {device.rttMs != null ? `${device.rttMs}ms` : null}
          </span>
        </div>
        <div className="text-xs text-muted-foreground">
          {isOnline
            ? `配对于 ${formatDate(device.pairedAt)}`
            : `最后在线 ${formatRelativeTime(device.lastSeen)}`}
        </div>
      </DeviceInfoCard>

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon-sm">
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem
            className="text-destructive focus:text-destructive"
            onClick={handleUnpair}
            disabled={loading}
          >
            {loading ? "取消配对中..." : "取消配对"}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
