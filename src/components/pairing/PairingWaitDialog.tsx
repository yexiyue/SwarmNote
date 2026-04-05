import { Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { DeviceInfoCard } from "./DeviceInfoCard";

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

        <DeviceInfoCard hostname={deviceName} os={deviceOs} />

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
