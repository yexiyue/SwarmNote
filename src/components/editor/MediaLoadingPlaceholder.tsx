import { Trans } from "@lingui/react/macro";
import { ImageIcon, RefreshCw, VideoIcon } from "lucide-react";

import type { LoadState } from "./useMediaLoader";

interface MediaLoadingPlaceholderProps {
  type: "image" | "video";
  fileName: string;
  state: LoadState;
  onRetry: () => void;
}

export function MediaLoadingPlaceholder({
  type,
  fileName,
  state,
  onRetry,
}: MediaLoadingPlaceholderProps) {
  const Icon = type === "image" ? ImageIcon : VideoIcon;

  if (state === "failed") {
    return (
      <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-muted-foreground/30 bg-muted/30 px-4 py-8">
        <Icon className="h-8 w-8 text-muted-foreground/50" />
        <p className="text-xs text-muted-foreground">{fileName}</p>
        <button
          type="button"
          className="flex items-center gap-1.5 rounded-md border bg-background px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted"
          onClick={onRetry}
        >
          <RefreshCw className="h-3 w-3" />
          <Trans>点击重试</Trans>
        </button>
      </div>
    );
  }

  // loading / retrying → skeleton pulse
  return (
    <div className="flex flex-col items-center justify-center gap-2 rounded-lg bg-muted/40 px-4 py-8 animate-pulse">
      <Icon className="h-8 w-8 text-muted-foreground/40" />
      <p className="text-xs text-muted-foreground/60">{fileName}</p>
    </div>
  );
}
