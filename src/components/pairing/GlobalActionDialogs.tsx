import { useCallback, useState } from "react";
import { respondPairingRequest } from "@/commands/pairing";
import { useNotificationStore } from "@/stores/notificationStore";
import { PairingRequestDialog } from "./PairingRequestDialog";

export function GlobalActionDialogs() {
  const current = useNotificationStore((s) => s.current);
  const [responding, setResponding] = useState(false);

  const dismiss = useCallback(() => {
    if (!current) return;
    useNotificationStore.getState().respond(current.id);
    setResponding(false);
  }, [current]);

  const handlePairingRespond = useCallback(
    (pendingId: number, accept: boolean) => {
      setResponding(true);
      // fire-and-forget: 关闭弹窗不等后端结果
      respondPairingRequest(pendingId, accept).catch((e) => {
        console.error(`Failed to ${accept ? "accept" : "reject"} pairing:`, e);
      });
      dismiss();
    },
    [dismiss],
  );

  if (!current) return null;

  if (current.type === "pairing-request") {
    // biome-ignore lint/suspicious/noExplicitAny: payload type is validated by event source
    const data = current.payload as any;
    return (
      <PairingRequestDialog
        data={data}
        responding={responding}
        onAccept={() => handlePairingRespond(data.pendingId, true)}
        onReject={() => handlePairingRespond(data.pendingId, false)}
        onClose={dismiss}
      />
    );
  }

  return null;
}
