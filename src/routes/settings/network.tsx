import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/settings/network")({
  beforeLoad: () => {
    throw redirect({ to: "/settings/sync", replace: true });
  },
  component: () => null,
});
