import { Trans } from "@lingui/react/macro";
import { FileText } from "lucide-react";

export function EmptyTreeState() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-2 px-4 py-8 text-center">
      <FileText className="h-8 w-8 text-muted-foreground/50" />
      <p className="text-sm text-muted-foreground">
        <Trans>还没有笔记</Trans>
      </p>
      <p className="text-xs text-muted-foreground/70">
        <Trans>点击上方 + 创建第一篇</Trans>
      </p>
    </div>
  );
}
