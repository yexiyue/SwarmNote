import { Monitor } from "lucide-react";
import { useEffect, useState } from "react";
import { getDeviceInfo, setDeviceName } from "@/commands/identity";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useOnboardingStore } from "@/stores/onboardingStore";

export function DeviceNameStep() {
  const nextStep = useOnboardingStore((s) => s.nextStep);
  const prevStep = useOnboardingStore((s) => s.prevStep);
  const [name, setName] = useState("");
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    getDeviceInfo().then((info) => {
      setName(info.device_name);
      setIsLoading(false);
    });
  }, []);

  async function handleNext() {
    const trimmed = name.trim();
    if (!trimmed) return;
    await setDeviceName(trimmed);
    nextStep();
  }

  return (
    <>
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-muted">
        <Monitor className="h-8 w-8 text-muted-foreground" />
      </div>

      <div className="flex flex-col items-center gap-2">
        <h2 className="text-xl font-bold text-foreground">设备名称</h2>
        <p className="text-center text-sm text-muted-foreground">
          为你的设备取个名字，方便在 P2P 网络中识别。
        </p>
      </div>

      <div className="w-full">
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="我的设备"
          disabled={isLoading}
          className="text-center"
          onKeyDown={(e) => e.key === "Enter" && handleNext()}
        />
      </div>

      <div className="flex w-full gap-3">
        <Button variant="outline" className="flex-1" onClick={prevStep}>
          上一步
        </Button>
        <Button className="flex-1" onClick={handleNext} disabled={!name.trim()}>
          下一步
        </Button>
      </div>
    </>
  );
}
