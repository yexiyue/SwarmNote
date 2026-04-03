import { Trans } from "@lingui/react/macro";
import { Loader2 } from "lucide-react";
import { useEffect } from "react";
import { useShallow } from "zustand/react/shallow";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useUpgradeStore } from "@/stores/upgradeStore";

export function ForceUpdateDialog() {
  const { status, latestVersion, currentVersion, releaseNotes, progress, startDownload } =
    useUpgradeStore(
      useShallow((s) => ({
        status: s.status,
        latestVersion: s.latestVersion,
        currentVersion: s.currentVersion,
        releaseNotes: s.releaseNotes,
        progress: s.progress,
        startDownload: s.startDownload,
      })),
    );

  const isDownloading = status === "downloading";
  const isReady = status === "ready";
  const open = status === "force-required" || isDownloading || isReady;

  // 强制更新自动开始下载
  useEffect(() => {
    if (status === "force-required") {
      startDownload();
    }
  }, [status, startDownload]);

  return (
    <Dialog open={open}>
      <DialogContent
        className="sm:max-w-md"
        onPointerDownOutside={(e) => e.preventDefault()}
        onEscapeKeyDown={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>
            <Trans>需要更新</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              当前版本 {currentVersion} 已不再支持，请更新到 {latestVersion}
            </Trans>
          </DialogDescription>
        </DialogHeader>

        {releaseNotes && (
          <div className="max-h-40 overflow-y-auto rounded-lg bg-muted p-3">
            <pre className="whitespace-pre-wrap font-sans text-xs text-muted-foreground">
              {releaseNotes}
            </pre>
          </div>
        )}

        <p className="text-sm text-muted-foreground">
          <Trans>此版本为必须更新，将自动下载并安装。</Trans>
        </p>

        {(isDownloading || isReady) && progress && (
          <div className="space-y-1.5">
            <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary transition-all duration-300"
                style={{ width: `${progress.percent}%` }}
              />
            </div>
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>{progress.percent}%</span>
              <span>{(progress.speed / 1024 / 1024).toFixed(1)} MB/s</span>
            </div>
          </div>
        )}

        <Button disabled className="w-full">
          {isReady ? (
            <Trans>正在重启...</Trans>
          ) : (
            <>
              <Loader2 className="mr-2 size-4 animate-spin" />
              <Trans>下载中...</Trans>
            </>
          )}
        </Button>
      </DialogContent>
    </Dialog>
  );
}
