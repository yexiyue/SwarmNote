import { PenLine, RefreshCw, Shield, Wifi } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useOnboardingStore } from "@/stores/onboardingStore";

const features = [
  { icon: RefreshCw, label: "多端协作" },
  { icon: Shield, label: "安全加密" },
  { icon: Wifi, label: "P2P 同步" },
] as const;

export function WelcomeStep() {
  const nextStep = useOnboardingStore((s) => s.nextStep);

  return (
    <>
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary">
        <PenLine className="h-8 w-8 text-white" />
      </div>

      <div className="flex flex-col items-center gap-2">
        <h1 className="text-2xl font-bold text-foreground">SwarmNote</h1>
        <p className="text-center text-sm text-muted-foreground">
          去中心化、本地优先的笔记应用，通过 P2P 网络在设备间安全同步。
        </p>
      </div>

      <div className="flex gap-6">
        {features.map(({ icon: Icon, label }) => (
          <div key={label} className="flex flex-col items-center gap-1.5">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
              <Icon className="h-5 w-5 text-muted-foreground" />
            </div>
            <span className="text-xs text-muted-foreground">{label}</span>
          </div>
        ))}
      </div>

      <Button className="w-full" size="lg" onClick={nextStep}>
        开始使用
      </Button>
    </>
  );
}
