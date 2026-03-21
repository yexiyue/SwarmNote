import { CircleCheck } from "lucide-react";

export function StatusBar() {
  return (
    <footer className="flex h-7 shrink-0 items-center justify-between border-t border-border bg-card px-4">
      <div className="flex items-center gap-3">
        <span className="text-[11px] text-muted-foreground">328 字</span>
        <span className="text-[11px] text-muted-foreground">1,024 字符</span>
      </div>
      <div className="flex items-center gap-2">
        <CircleCheck className="h-3 w-3 text-green-500" />
        <span className="text-[11px] text-muted-foreground">已保存 10:30</span>
      </div>
    </footer>
  );
}
