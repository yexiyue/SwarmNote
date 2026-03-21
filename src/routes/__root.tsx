import { i18n } from "@lingui/core";
import { I18nProvider } from "@lingui/react";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { CommandPalette } from "@/components/layout/CommandPalette";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  useKeyboardShortcuts();

  return (
    <I18nProvider i18n={i18n}>
      <TooltipProvider>
        <Outlet />
        <CommandPalette />
        {import.meta.env.DEV && <TanStackRouterDevtools position="bottom-right" />}
      </TooltipProvider>
    </I18nProvider>
  );
}
