import { Loader2, Search } from "lucide-react";
import { useState } from "react";
import { getDeviceByCode } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface InputCodeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDeviceFound: (
    peerId: string,
    osInfo: { hostname: string; os: string; platform: string; arch: string },
  ) => void;
}

export function InputCodeDialog({ open, onOpenChange, onDeviceFound }: InputCodeDialogProps) {
  const [code, setCode] = useState("");
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function handleCodeChange(value: string) {
    setCode(value.replace(/\D/g, "").slice(0, 6));
    setError(null);
  }

  async function handleSearch() {
    if (code.length < 6) return;
    setSearching(true);
    setError(null);
    try {
      const result = await getDeviceByCode(code);
      onDeviceFound(result.peerId, result.osInfo);
    } catch (e) {
      console.error("Failed to find device by code:", e);
      setError("未找到设备，请检查配对码是否正确");
    } finally {
      setSearching(false);
    }
  }

  function handleClose(nextOpen: boolean) {
    if (!nextOpen) {
      setCode("");
      setError(null);
      setSearching(false);
    }
    onOpenChange(nextOpen);
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>输入配对码</DialogTitle>
          <DialogDescription>输入对方设备上显示的 6 位数字</DialogDescription>
        </DialogHeader>

        <div className="flex justify-center py-2">
          <input
            type="text"
            inputMode="numeric"
            value={code}
            onChange={(e) => handleCodeChange(e.target.value)}
            placeholder="000000"
            maxLength={6}
            className="w-48 rounded-lg border bg-muted px-4 py-3 text-center font-mono text-2xl tracking-[0.3em] outline-none focus:ring-2 focus:ring-ring"
          />
        </div>

        {error ? <p className="text-center text-xs text-destructive">{error}</p> : null}

        <DialogFooter>
          <Button variant="outline" onClick={() => handleClose(false)} disabled={searching}>
            取消
          </Button>
          <Button onClick={handleSearch} disabled={code.length < 6 || searching}>
            {searching ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                查找中...
              </>
            ) : (
              <>
                <Search className="h-4 w-4" />
                查找设备
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
