import { RadioTower, Wifi, Zap } from "lucide-react";
import type { ConnectionType } from "@/commands/pairing";
import { cn } from "@/lib/utils";

const connectionConfig: Record<
  ConnectionType,
  {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    className: string;
  }
> = {
  lan: {
    icon: Wifi,
    label: "局域网",
    className: "bg-green-500/15 text-green-700 dark:text-green-400",
  },
  dcutr: {
    icon: Zap,
    label: "打洞",
    className: "bg-blue-500/15 text-blue-700 dark:text-blue-400",
  },
  relay: {
    icon: RadioTower,
    label: "中继",
    className: "bg-orange-500/15 text-orange-700 dark:text-orange-400",
  },
};

interface ConnectionBadgeProps {
  type: ConnectionType;
  latency?: number;
  className?: string;
}

export function ConnectionBadge({ type, latency, className }: ConnectionBadgeProps) {
  const config = connectionConfig[type];
  const Icon = config.icon;
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium",
        config.className,
        className,
      )}
    >
      <Icon className="h-3 w-3" />
      {config.label}
      {latency !== undefined && <span className="opacity-70">{latency}ms</span>}
    </span>
  );
}
