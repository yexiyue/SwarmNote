import { Search } from "lucide-react";
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
import { ErrorMessage } from "@/components/ui/error-message";
import { useAsyncAction } from "@/hooks/useAsyncAction";

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
  const { loading, error, run, clearError } = useAsyncAction();

  function handleCodeChange(value: string) {
    setCode(value.replace(/\D/g, "").slice(0, 6));
    clearError();
  }

  async function handleSearch() {
    if (code.length < 6) return;
    await run(async () => {
      const result = await getDeviceByCode(code);
      onDeviceFound(result.peerId, result.osInfo);
    });
  }

  function handleClose(nextOpen: boolean) {
    if (!nextOpen) {
      setCode("");
      clearError();
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

        <ErrorMessage error={error} className="text-center" />

        <DialogFooter>
          <Button variant="outline" onClick={() => handleClose(false)} disabled={loading}>
            取消
          </Button>
          <Button onClick={handleSearch} disabled={code.length < 6} loading={loading}>
            {loading ? (
              "查找中..."
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
