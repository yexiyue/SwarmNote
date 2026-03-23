import { Trans } from "@lingui/react/macro";
import { FolderOpen, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export function SelectWorkspace() {
  const { selectAndOpenWorkspace, isLoading, error } = useWorkspaceStore();

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-4">
      <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-muted">
        <FolderOpen className="h-7 w-7 text-muted-foreground" />
      </div>
      <h2 className="text-lg font-semibold text-foreground">
        <Trans>选择工作区</Trans>
      </h2>
      <p className="max-w-sm text-center text-sm text-muted-foreground">
        <Trans>选择一个文件夹作为你的工作区，所有笔记将保存在该目录中。</Trans>
      </p>
      {error && (
        <p className="max-w-sm text-center text-sm text-destructive">
          <Trans>打开工作区失败，请重试或选择其他目录。</Trans>
        </p>
      )}
      <Button
        className="gap-1.5 rounded-lg px-5 py-2.5"
        onClick={selectAndOpenWorkspace}
        disabled={isLoading}
      >
        {isLoading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <FolderOpen className="h-4 w-4" />
        )}
        <Trans>选择文件夹</Trans>
      </Button>
    </div>
  );
}
