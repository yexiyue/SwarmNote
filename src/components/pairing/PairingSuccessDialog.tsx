import { CircleCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface PairingSuccessDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  deviceName: string;
}

export function PairingSuccessDialog({
  open,
  onOpenChange,
  deviceName,
}: PairingSuccessDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false}>
        <DialogHeader className="items-center text-center">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-green-100 dark:bg-green-900/30">
            <CircleCheck className="h-6 w-6 text-green-600 dark:text-green-400" />
          </div>
          <DialogTitle>配对成功</DialogTitle>
          <DialogDescription className="text-center">
            已与 {deviceName} 配对
            <br />
            现在可以同步笔记了
          </DialogDescription>
        </DialogHeader>

        <Button className="w-full" onClick={() => onOpenChange(false)}>
          完成
        </Button>
      </DialogContent>
    </Dialog>
  );
}
