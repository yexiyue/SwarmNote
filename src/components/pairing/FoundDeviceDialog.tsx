import { Link, Loader2, Monitor } from "lucide-react";
import { useState } from "react";
import { requestPairing } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface FoundDeviceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  peerId: string;
  osInfo: { hostname: string; os: string; platform: string; arch: string };
  code: string;
  onSuccess: () => void;
}

export function FoundDeviceDialog({
  open,
  onOpenChange,
  peerId,
  osInfo,
  code,
  onSuccess,
}: FoundDeviceDialogProps) {
  const [pairing, setPairing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleConfirm() {
    setPairing(true);
    setError(null);
    try {
      const resp = await requestPairing(peerId, { type: "Code", code });
      if (resp.status === "Success") {
        onSuccess();
      } else {
        setError(resp.reason ?? "配对被拒绝");
      }
    } catch (e) {
      console.error("Failed to request pairing:", e);
      setError("配对请求失败");
    } finally {
      setPairing(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>找到设备</DialogTitle>
        </DialogHeader>

        <div className="flex items-center gap-3 rounded-lg border p-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
            <Monitor className="h-5 w-5 text-muted-foreground" />
          </div>
          <div>
            <div className="text-sm font-medium">{osInfo.hostname}</div>
            <div className="text-xs text-muted-foreground">
              {osInfo.os} · {osInfo.platform}
            </div>
          </div>
        </div>

        <DialogDescription>确认与此设备配对？</DialogDescription>

        {error ? <p className="text-xs text-destructive">{error}</p> : null}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={pairing}>
            取消
          </Button>
          <Button onClick={handleConfirm} disabled={pairing}>
            {pairing ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                配对中...
              </>
            ) : (
              <>
                <Link className="h-4 w-4" />
                确认配对
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
