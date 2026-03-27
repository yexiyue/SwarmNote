import { Monitor, MoreHorizontal } from "lucide-react";
import { useState } from "react";
import type { PairedDeviceInfo } from "@/commands/pairing";
import { unpairDevice } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";

function formatRelativeTime(timestamp: number | null): string {
  if (timestamp == null) return "从未在线";
  const diff = Date.now() - timestamp;
  const minutes = Math.floor(diff / 60_000);
  if (minutes < 1) return "刚刚";
  if (minutes < 60) return `${minutes}分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}小时前`;
  const days = Math.floor(hours / 24);
  return `${days}天前`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

interface PairedDeviceCardProps {
  device: PairedDeviceInfo;
  onUnpaired?: () => void;
}

export function PairedDeviceCard({ device, onUnpaired }: PairedDeviceCardProps) {
  const [unpairing, setUnpairing] = useState(false);
  const isOnline = device.isOnline ?? false;

  async function handleUnpair() {
    setUnpairing(true);
    try {
      await unpairDevice(device.peerId);
      onUnpaired?.();
    } catch (e) {
      console.error("Failed to unpair device:", e);
    } finally {
      setUnpairing(false);
    }
  }

  return (
    <div className="flex items-center justify-between rounded-lg border p-4">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
          <Monitor className="h-5 w-5 text-muted-foreground" />
        </div>
        <div>
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">{device.hostname}</span>
            <span
              className={cn(
                "inline-block h-2 w-2 rounded-full",
                isOnline ? "bg-green-500" : "bg-gray-300",
              )}
            />
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span>
              {device.os} · {device.platform}
            </span>
            {device.rttMs != null ? <span>· {device.rttMs}ms</span> : null}
          </div>
          <div className="text-xs text-muted-foreground">
            {isOnline
              ? `配对于 ${formatDate(device.pairedAt)}`
              : `最后在线 ${formatRelativeTime(device.lastSeen)}`}
          </div>
        </div>
      </div>

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
            disabled={unpairing}
          >
            {unpairing ? "取消配对中..." : "取消配对"}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
