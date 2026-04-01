import { Trans } from "@lingui/react/macro";
import { useNavigate } from "@tanstack/react-router";
import { CheckCircle, Fingerprint, Monitor } from "lucide-react";
import { useEffect, useState } from "react";
import { getDeviceInfo } from "@/commands/identity";
import { Button } from "@/components/ui/button";
import { useOnboardingStore } from "@/stores/onboardingStore";

export function CompleteStep() {
  const complete = useOnboardingStore((s) => s.complete);
  const navigate = useNavigate();
  const [deviceName, setDeviceName] = useState("");
  const [peerId, setPeerId] = useState("");

  useEffect(() => {
    getDeviceInfo().then((info) => {
      setDeviceName(info.device_name);
      setPeerId(info.peer_id.slice(0, 8));
    });
  }, []);

  function handleFinish() {
    complete();
    navigate({ to: "/" });
  }

  return (
    <>
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-green-100">
        <CheckCircle className="h-8 w-8 text-green-600" />
      </div>

      <div className="flex flex-col items-center gap-2">
        <h2 className="text-xl font-bold text-foreground">
          <Trans>准备就绪!</Trans>
        </h2>
        <p className="text-center text-sm text-muted-foreground">
          <Trans>你的设备身份已建立，可以开始使用 SwarmNote 了。</Trans>
        </p>
      </div>

      <div className="flex w-full flex-col gap-3 rounded-lg border border-border bg-muted/50 p-4">
        <div className="flex items-center gap-3">
          <Monitor className="h-4 w-4 text-muted-foreground" />
          <div className="flex flex-col">
            <span className="text-xs text-muted-foreground">
              <Trans>设备名称</Trans>
            </span>
            <span className="text-sm font-medium text-foreground">{deviceName}</span>
          </div>
        </div>
        <div className="h-px bg-border" />
        <div className="flex items-center gap-3">
          <Fingerprint className="h-4 w-4 text-muted-foreground" />
          <div className="flex flex-col">
            <span className="text-xs text-muted-foreground">
              <Trans>设备 ID</Trans>
            </span>
            <span className="font-mono text-sm text-foreground">{peerId}</span>
          </div>
        </div>
      </div>

      <Button className="w-full" size="lg" onClick={handleFinish}>
        <Trans>进入 SwarmNote</Trans>
      </Button>
    </>
  );
}
