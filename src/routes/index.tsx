import { createFileRoute } from "@tanstack/react-router";
import { Loader2 } from "lucide-react";

import { AppLayout } from "@/components/layout/AppLayout";
import { EditorPane } from "@/components/layout/EditorPane";
import { WorkspacePicker } from "@/components/workspace/WorkspacePicker";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  const { workspace, isLoading } = useWorkspaceStore();
  const workspacePickerOpen = useUIStore((s) => s.workspacePickerOpen);
  const setWorkspacePickerOpen = useUIStore((s) => s.setWorkspacePickerOpen);

  // 工作区窗口创建时后端已预绑定 workspace，
  // 前端只需等待 workspace:ready 事件。
  if (isLoading || !workspace) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <>
      <AppLayout>
        <EditorPane />
      </AppLayout>
      <WorkspacePicker open={workspacePickerOpen} onOpenChange={setWorkspacePickerOpen} />
    </>
  );
}
