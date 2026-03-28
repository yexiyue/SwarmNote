import { Trans } from "@lingui/react/macro";
import { FileText, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { modKey } from "@/lib/utils";
import { useFileTreeStore } from "@/stores/fileTreeStore";

export function EmptyState() {
  const createAndOpenFile = useFileTreeStore((s) => s.createAndOpenFile);

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4">
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-muted">
        <FileText className="h-7 w-7 text-muted-foreground" />
      </div>
      <h2 className="text-lg font-semibold text-foreground">
        <Trans>还没有笔记</Trans>
      </h2>
      <p className="text-sm text-muted-foreground">
        <Trans>创建你的第一篇笔记，开始记录想法</Trans>
      </p>
      <Button
        className="gap-1.5 rounded-lg px-5 py-2.5"
        onClick={() => createAndOpenFile("", "新建笔记")}
      >
        <Plus className="h-4 w-4" />
        <Trans>新建笔记</Trans>
      </Button>
      <p className="text-xs text-muted-foreground">
        <Trans>或按 {modKey}N 快速创建</Trans>
      </p>
    </div>
  );
}
