import { Trans } from "@lingui/react/macro";
import { ChevronRight, MonitorSmartphone, Sparkles } from "lucide-react";
import { useOnboardingStore } from "@/stores/onboardingStore";

export function PathChoiceStep() {
  const nextStep = useOnboardingStore((s) => s.nextStep);
  const setUserPath = useOnboardingStore((s) => s.setUserPath);

  function handleChoose(path: "new" | "add-device") {
    setUserPath(path);
    nextStep();
  }

  return (
    <>
      <div className="flex flex-col items-center gap-2">
        <h2 className="text-xl font-bold text-foreground">
          <Trans>你是如何开始的？</Trans>
        </h2>
        <p className="text-center text-sm text-muted-foreground">
          <Trans>选择最适合你当前情况的选项</Trans>
        </p>
      </div>

      <div className="flex w-full flex-col gap-3">
        {/* 全新开始 */}
        <button
          type="button"
          className="flex w-full cursor-pointer items-center gap-4 rounded-xl border border-border bg-card p-4 text-left transition-colors hover:bg-muted/50"
          onClick={() => handleChoose("new")}
        >
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
            <Sparkles className="h-5 w-5 text-muted-foreground" />
          </div>
          <div className="flex flex-1 flex-col">
            <span className="text-sm font-medium text-foreground">
              <Trans>全新开始</Trans>
            </span>
            <span className="text-xs text-muted-foreground">
              <Trans>这是我的第一台 SwarmNote 设备</Trans>
            </span>
          </div>
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
        </button>

        {/* 添加设备 */}
        <button
          type="button"
          className="flex w-full cursor-pointer items-center gap-4 rounded-xl border border-primary/50 bg-card p-4 text-left transition-colors hover:bg-muted/50"
          onClick={() => handleChoose("add-device")}
        >
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
            <MonitorSmartphone className="h-5 w-5 text-primary" />
          </div>
          <div className="flex flex-1 flex-col">
            <span className="text-sm font-medium text-foreground">
              <Trans>添加设备</Trans>
            </span>
            <span className="text-xs text-muted-foreground">
              <Trans>我已有其他设备，想要同步笔记</Trans>
            </span>
          </div>
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
        </button>
      </div>
    </>
  );
}
