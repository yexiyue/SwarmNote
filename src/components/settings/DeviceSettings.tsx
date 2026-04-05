import { Trans } from "@lingui/react/macro";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function DeviceSettings() {
  // TODO: load from identity store once device identity is implemented
  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <Label htmlFor="device-name">
          <Trans>设备名称</Trans>
        </Label>
        <Input id="device-name" defaultValue="My-Desktop" disabled />
      </div>

      <div className="space-y-2">
        <Label>Peer ID</Label>
        <p className="rounded-md bg-muted px-3 py-2 font-mono text-xs text-muted-foreground">—</p>
      </div>
    </div>
  );
}
