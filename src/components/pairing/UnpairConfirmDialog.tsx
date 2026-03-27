import { Loader2, Unlink } from "lucide-react";
import { useState } from "react";
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
  const [unpairing, setUnpairing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleConfirm() {
    setUnpairing(true);
    setError(null);
    try {
      await unpairDevice(peerId);
      onConfirm();
    } catch (e) {
      console.error("Failed to unpair device:", e);
      setError("取消配对失败");
    } finally {
      setUnpairing(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false}>
        <DialogHeader className="items-center text-center">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
            <Unlink className="h-6 w-6 text-destructive" />
          </div>
          <DialogTitle>取消配对</DialogTitle>
          <DialogDescription className="text-center">
            确定要取消与 {deviceName} 的配对吗？
            <br />
            取消配对后将停止与该设备的笔记同步。
          </DialogDescription>
        </DialogHeader>

        {error ? <p className="text-center text-xs text-destructive">{error}</p> : null}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={unpairing}>
            取消
          </Button>
          <Button variant="destructive" onClick={handleConfirm} disabled={unpairing}>
            {unpairing ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                取消配对中...
              </>
            ) : (
              "确认取消"
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
