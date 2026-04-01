import { useCallback, useEffect, useRef, useState } from "react";
import { respondPairingRequest } from "@/commands/pairing";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useNotificationStore } from "@/stores/notificationStore";
import { DeviceInfoCard } from "./DeviceInfoCard";

interface PairingRequestPayload {
  pendingId: number;
  peerId: string;
  osInfo: { hostname: string; os: string; platform: string; arch: string };
  method: { type: "Direct" } | { type: "Code"; code: string };
  expiresAt: string;
}

interface PairingRequestDialogProps {
  data: PairingRequestPayload;
}

export function PairingRequestDialog({ data }: PairingRequestDialogProps) {
  const [responding, setResponding] = useState(false);
  const [remaining, setRemaining] = useState(() =>
    Math.max(0, Math.ceil((new Date(data.expiresAt).getTime() - Date.now()) / 1000)),
  );
  const respondedRef = useRef(false);

  const notificationId = useNotificationStore((s) => s.current?.id);

  const close = useCallback(() => {
    if (notificationId) {
      useNotificationStore.getState().respond(notificationId);
    }
  }, [notificationId]);

  const handleReject = useCallback(async () => {
    if (respondedRef.current) return;
    respondedRef.current = true;
    setResponding(true);
    try {
      await respondPairingRequest(data.pendingId, false);
    } catch (e) {
      console.error("Failed to reject pairing request:", e);
    } finally {
      setResponding(false);
      close();
    }
  }, [data.pendingId, close]);

  async function handleAccept() {
    if (respondedRef.current) return;
    respondedRef.current = true;
    setResponding(true);
    try {
      await respondPairingRequest(data.pendingId, true);
    } catch (e) {
      console.error("Failed to accept pairing request:", e);
    } finally {
      setResponding(false);
      close();
    }
  }

  // Countdown timer
  useEffect(() => {
    const interval = setInterval(() => {
      const left = Math.max(0, Math.ceil((new Date(data.expiresAt).getTime() - Date.now()) / 1000));
      setRemaining(left);
      if (left <= 0) {
        clearInterval(interval);
        handleReject();
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [data.expiresAt, handleReject]);

  return (
    <Dialog open onOpenChange={(open) => !open && handleReject()}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>收到配对请求</DialogTitle>
        </DialogHeader>

        <DeviceInfoCard
          hostname={data.osInfo.hostname}
          os={data.osInfo.os}
          platform={data.osInfo.platform}
        />

        <DialogDescription>
          该设备请求与您配对。
          <br />
          配对后可同步笔记。
        </DialogDescription>

        <div className="text-center text-xs text-muted-foreground">剩余时间：{remaining} 秒</div>

        <DialogFooter>
          <Button variant="outline" onClick={handleReject} disabled={responding}>
            拒绝
          </Button>
          <Button onClick={handleAccept} loading={responding}>
            {responding ? "处理中..." : "接受"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
