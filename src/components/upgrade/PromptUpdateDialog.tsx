import { Trans } from "@lingui/react/macro";
import { Download, Loader2 } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useUpgradeStore } from "@/stores/upgradeStore";

interface PromptUpdateDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function PromptUpdateDialog({ open, onOpenChange }: PromptUpdateDialogProps) {
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

  const handleUpdate = async () => {
    onOpenChange(false);
    await startDownload();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="size-5 text-primary" />
            <Trans>发现新版本</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>
              新版本 {latestVersion} 可用，当前版本 {currentVersion}
            </Trans>
          </DialogDescription>
        </DialogHeader>

        {releaseNotes && (
          <div className="rounded-lg bg-muted p-3">
            <p className="mb-1.5 text-xs font-medium text-muted-foreground">
              <Trans>更新内容</Trans>
            </p>
            <div className="max-h-40 overflow-y-auto">
              <pre className="whitespace-pre-wrap font-sans text-xs text-foreground">
                {releaseNotes}
              </pre>
            </div>
          </div>
        )}

        {isDownloading && progress && (
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

        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isDownloading || isReady}
          >
            <Trans>稍后提醒</Trans>
          </Button>
          <Button onClick={handleUpdate} disabled={isDownloading || isReady}>
            {isDownloading ? (
              <>
                <Loader2 className="mr-2 size-4 animate-spin" />
                <Trans>下载中...</Trans>
              </>
            ) : isReady ? (
              <Trans>正在重启...</Trans>
            ) : (
              <Trans>立即更新</Trans>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
