import { Loader2, Monitor } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface PairingWaitDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  deviceName: string;
  deviceOs: string;
  onCancel: () => void;
}

export function PairingWaitDialog({
  open,
  onOpenChange,
  deviceName,
  deviceOs,
  onCancel,
}: PairingWaitDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>等待对方确认...</DialogTitle>
        </DialogHeader>

        <div className="flex items-center gap-3 rounded-lg border p-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
            <Monitor className="h-5 w-5 text-muted-foreground" />
          </div>
          <div>
            <div className="text-sm font-medium">{deviceName}</div>
            <div className="text-xs text-muted-foreground">{deviceOs}</div>
          </div>
        </div>

        <DialogDescription className="flex items-center gap-2">
          <Loader2 className="h-4 w-4 animate-spin" />
          已发送配对请求，等待对方接受
        </DialogDescription>

        <DialogFooter>
          <Button variant="outline" onClick={onCancel}>
            取消
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
