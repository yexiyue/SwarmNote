import { createFileRoute } from "@tanstack/react-router";
import { AppLayout } from "@/components/layout/AppLayout";
import { EditorPane } from "@/components/layout/EditorPane";
import { EmptyState } from "@/components/layout/EmptyState";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  return (
    <AppLayout>
      <EditorPane>
        <EmptyState />
      </EditorPane>
    </AppLayout>
  );
}
