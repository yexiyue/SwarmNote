import { createFileRoute } from "@tanstack/react-router";
import { Loader2 } from "lucide-react";

import { AppLayout } from "@/components/layout/AppLayout";
import { EditorPane } from "@/components/layout/EditorPane";
import { OnboardingFlow } from "@/components/onboarding/OnboardingFlow";
import { WorkspacePicker } from "@/components/workspace/WorkspacePicker";
import { useOnboardingStore } from "@/stores/onboardingStore";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  const { workspace, isLoading } = useWorkspaceStore();
  const isCompleted = useOnboardingStore((s) => s.isCompleted);
  const workspacePickerOpen = useUIStore((s) => s.workspacePickerOpen);
  const setWorkspacePickerOpen = useUIStore((s) => s.setWorkspacePickerOpen);

  if (isLoading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!isCompleted) {
    return <OnboardingFlow />;
  }

  if (!workspace) {
    return <WorkspacePicker mode="fullscreen" />;
  }

  return (
    <>
      <AppLayout>
        <EditorPane />
      </AppLayout>
      <WorkspacePicker
        mode="dialog"
        open={workspacePickerOpen}
        onOpenChange={setWorkspacePickerOpen}
      />
    </>
  );
}
