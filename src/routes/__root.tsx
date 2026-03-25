import { i18n } from "@lingui/core";
import { I18nProvider } from "@lingui/react";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { useEffect } from "react";

import { CommandPalette } from "@/components/layout/CommandPalette";
import { SettingsDialog } from "@/components/settings/SettingsDialog";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { isPickerMode } from "@/lib/windowUtils";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  useKeyboardShortcuts();

  const initFromBackend = useWorkspaceStore((s) => s.initFromBackend);

  useEffect(() => {
    // Picker windows start fresh — don't auto-restore the last workspace
    if (!isPickerMode()) {
      initFromBackend();
    }
  }, [initFromBackend]);

  return (
    <I18nProvider i18n={i18n}>
      <TooltipProvider>
        <Outlet />
        <CommandPalette />
        <SettingsDialog />
        {import.meta.env.DEV && <TanStackRouterDevtools position="bottom-right" />}
      </TooltipProvider>
    </I18nProvider>
  );
}
