import { i18n } from "@lingui/core";
import { I18nProvider } from "@lingui/react";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { listen } from "@tauri-apps/api/event";
import { Loader2 } from "lucide-react";
import { useEffect, useState } from "react";

import { CommandPalette } from "@/components/layout/CommandPalette";
import { GlobalActionDialogs } from "@/components/pairing/GlobalActionDialogs";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useNotificationStore } from "@/stores/notificationStore";
import { waitForOnboardingHydration } from "@/stores/onboardingStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  useKeyboardShortcuts();

  const initFromBackend = useWorkspaceStore((s) => s.initFromBackend);
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    Promise.all([waitForOnboardingHydration(), initFromBackend()])
      .catch((err) => console.error("Hydration failed:", err))
      .finally(() => setHydrated(true));
  }, [initFromBackend]);

  // Listen for pairing request events from the Tauri backend
  useEffect(() => {
    const unlisten = listen("pairing-request-received", (event) => {
      useNotificationStore.getState().push({
        id: `pairing-${Date.now()}`,
        type: "pairing-request",
        payload: event.payload,
        timestamp: Date.now(),
      });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  if (!hydrated) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <I18nProvider i18n={i18n}>
      <TooltipProvider>
        <Outlet />
        <CommandPalette />
        <GlobalActionDialogs />
        {import.meta.env.DEV && <TanStackRouterDevtools position="bottom-right" />}
      </TooltipProvider>
    </I18nProvider>
  );
}
