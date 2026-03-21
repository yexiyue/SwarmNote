import { FileText, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { modKey } from "@/lib/utils";

export function EmptyState() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4">
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-muted">
        <FileText className="h-7 w-7 text-muted-foreground" />
      </div>
      <h2 className="text-lg font-semibold text-foreground">还没有笔记</h2>
      <p className="text-sm text-muted-foreground">创建你的第一篇笔记，开始记录想法</p>
      <Button className="gap-1.5 rounded-lg px-5 py-2.5">
        <Plus className="h-4 w-4" />
        新建笔记
      </Button>
      <p className="text-xs text-muted-foreground">或按 {modKey}N 快速创建</p>
    </div>
  );
}
