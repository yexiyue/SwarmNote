import { RadioTower, Wifi, Zap } from "lucide-react";
import type { ConnectionType } from "@/commands/pairing";
import { cn } from "@/lib/utils";

const connectionConfig: Record<
  ConnectionType,
  {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    bgColor: string;
    textColor: string;
  }
> = {
  lan: {
    icon: Wifi,
    label: "局域网",
    bgColor: "bg-green-100",
    textColor: "text-green-600",
  },
  dcutr: {
    icon: Zap,
    label: "打洞",
    bgColor: "bg-blue-100",
    textColor: "text-blue-600",
  },
  relay: {
    icon: RadioTower,
    label: "中继",
    bgColor: "bg-amber-100",
    textColor: "text-amber-600",
  },
};

interface ConnectionBadgeProps {
  type: ConnectionType;
  latency?: number;
}

export function ConnectionBadge({ type, latency }: ConnectionBadgeProps) {
  const config = connectionConfig[type];
  const Icon = config.icon;
  return (
    <div className={cn("flex items-center gap-1 rounded-full px-1.5 py-0.5", config.bgColor)}>
      <Icon className={cn("size-2.5", config.textColor)} />
      <span className={cn("text-[10px] font-medium", config.textColor)}>{config.label}</span>
      {latency !== undefined && (
        <span className={cn("text-[10px] font-medium", config.textColor)}>{latency}ms</span>
      )}
    </div>
  );
}
