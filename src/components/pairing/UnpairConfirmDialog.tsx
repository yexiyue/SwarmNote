import { Trans } from "@lingui/react/macro";
import { Unlink } from "lucide-react";
import { unpairDevice } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ErrorMessage } from "@/components/ui/error-message";
import { useAsyncAction } from "@/hooks/useAsyncAction";

interface UnpairConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  deviceName: string;
  peerId: string;
  onConfirm: () => void;
}

export function UnpairConfirmDialog({
  open,
  onOpenChange,
  deviceName,
  peerId,
  onConfirm,
}: UnpairConfirmDialogProps) {
  const { loading, error, run } = useAsyncAction();

  async function handleConfirm() {
    await run(async () => {
      await unpairDevice(peerId);
      onConfirm();
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false}>
        <DialogHeader className="items-center text-center">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
            <Unlink className="h-6 w-6 text-destructive" />
          </div>
          <DialogTitle>
            <Trans>取消配对</Trans>
          </DialogTitle>
          <DialogDescription className="text-center">
            <Trans>确定要取消与 {deviceName} 的配对吗？</Trans>
            <br />
            <Trans>取消配对后将停止与该设备的笔记同步。</Trans>
          </DialogDescription>
        </DialogHeader>

        <ErrorMessage error={error} className="text-center" />

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={loading}>
            <Trans>取消</Trans>
          </Button>
          <Button variant="destructive" onClick={handleConfirm} loading={loading}>
            {loading ? <Trans>取消配对中...</Trans> : <Trans>确认取消</Trans>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
