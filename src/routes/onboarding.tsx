import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/onboarding")({
  component: OnboardingPage,
});

function OnboardingPage() {
  return (
    <div className="flex h-screen items-center justify-center">
      <p className="text-muted-foreground">Onboarding — 待实现 (#10)</p>
    </div>
  );
}
