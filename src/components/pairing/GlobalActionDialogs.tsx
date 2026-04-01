import type React from "react";
import { useNotificationStore } from "@/stores/notificationStore";
import { PairingRequestDialog } from "./PairingRequestDialog";

// biome-ignore lint/suspicious/noExplicitAny: dialog registry accepts heterogeneous payload types
const dialogs: Record<string, React.ComponentType<{ data: any; notificationId: string }>> = {
  "pairing-request": PairingRequestDialog,
};

export function GlobalActionDialogs() {
  const current = useNotificationStore((s) => s.current);

  if (!current) return null;

  const DialogComponent = dialogs[current.type];
  if (!DialogComponent) return null;

  return <DialogComponent data={current.payload} notificationId={current.id} />;
}
