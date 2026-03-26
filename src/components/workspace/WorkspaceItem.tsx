import { FolderOpen } from "lucide-react";

import type { RecentWorkspace } from "@/commands/workspace";

interface WorkspaceItemProps {
  workspace: RecentWorkspace;
  onClick: (path: string) => void;
}

export function WorkspaceItem({ workspace, onClick }: WorkspaceItemProps) {
  const timeAgo = formatRelativeTime(workspace.last_opened_at);

  return (
    <button
      type="button"
      onClick={() => onClick(workspace.path)}
      className="flex w-full items-center gap-3 rounded-lg border border-border bg-card p-3 text-left transition-colors hover:bg-accent"
    >
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
        <FolderOpen className="h-5 w-5 text-muted-foreground" />
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="text-sm font-medium text-foreground">{workspace.name}</span>
        <span className="truncate text-xs text-muted-foreground">{workspace.path}</span>
      </div>
      <span className="shrink-0 text-xs text-muted-foreground">{timeAgo}</span>
    </button>
  );
}

function formatRelativeTime(isoString: string): string {
  const date = new Date(isoString);
  const now = Date.now();
  const diffMs = now - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  if (diffMin < 1) return "刚刚";
  if (diffMin < 60) return `${diffMin} 分钟前`;
  const diffHour = Math.floor(diffMin / 60);
  if (diffHour < 24) return `${diffHour} 小时前`;
  const diffDay = Math.floor(diffHour / 24);
  if (diffDay < 30) return `${diffDay} 天前`;
  return date.toLocaleDateString();
}
