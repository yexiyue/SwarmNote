import { createFileRoute } from "@tanstack/react-router";
import { Loader2 } from "lucide-react";
import { useState } from "react";

import { AppLayout } from "@/components/layout/AppLayout";
import { EditorPane } from "@/components/layout/EditorPane";
import { EmptyState } from "@/components/layout/EmptyState";
import { SelectWorkspace } from "@/components/layout/SelectWorkspace";
import { WorkspacePicker } from "@/components/workspace/WorkspacePicker";
import { isPickerMode } from "@/lib/windowUtils";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  const { workspace, isLoading } = useWorkspaceStore();
  const [pickerMode, setPickerMode] = useState(() => isPickerMode());

  // TODO(#10): onboarding 实现后，在此添加 onboarding 路由守卫
  // if (!isOnboarded) navigate({ to: "/onboarding" });

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // Picker window: always show picker until user makes a selection
  if (pickerMode) {
    return <WorkspacePicker onSelected={() => setPickerMode(false)} />;
  }

  if (!workspace) {
    return <SelectWorkspace />;
  }

  return (
    <AppLayout>
      <EditorPane>
        <EmptyState />
      </EditorPane>
    </AppLayout>
  );
}
