import { create } from "zustand";
import { persist } from "zustand/middleware";
import { activateLocale, detectLocale, type Locale } from "@/i18n";
import { createTauriStorage, waitForHydration } from "@/lib/tauriStore";

type Theme = "light" | "dark" | "system";

export const SIDEBAR_WIDTH_MIN = 200;
export const SIDEBAR_WIDTH_MAX = 480;
export const SIDEBAR_WIDTH_DEFAULT = 256;

interface UIState {
  sidebarOpen: boolean;
  /** Persisted width of the sidebar (before collapse). Clamped to [200, 480]. */
  sidebarWidth: number;
  workspacePickerOpen: boolean;
  theme: Theme;
  resolvedTheme: "light" | "dark";
  locale: Locale;
}

interface UIActions {
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  setSidebarWidth: (width: number) => void;
  setWorkspacePickerOpen: (open: boolean) => void;
  setTheme: (theme: Theme) => void;
  setLocale: (locale: Locale) => void;
}

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(resolved: "light" | "dark") {
  document.documentElement.classList.toggle("dark", resolved === "dark");
}

export const useUIStore = create<UIState & UIActions>()(
  persist(
    (set) => ({
      sidebarOpen: true,
      sidebarWidth: SIDEBAR_WIDTH_DEFAULT,
      workspacePickerOpen: false,
      theme: "light",
      resolvedTheme: "light",
      locale: detectLocale(),

      toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),

      setSidebarOpen: (open) => set({ sidebarOpen: open }),

      setSidebarWidth: (width) => {
        const clamped = Math.max(SIDEBAR_WIDTH_MIN, Math.min(SIDEBAR_WIDTH_MAX, width));
        // Skip the set when the clamped value hasn't changed — avoids firing
        // subscribers on every mousemove once the drag goes past the clamp.
        if (clamped === useUIStore.getState().sidebarWidth) return;
        set({ sidebarWidth: clamped });
      },

      setWorkspacePickerOpen: (open) => set({ workspacePickerOpen: open }),

      setTheme: (theme) => {
        const resolved = theme === "system" ? getSystemTheme() : theme;
        applyTheme(resolved);
        set({ theme, resolvedTheme: resolved });
      },

      setLocale: (locale) => {
        activateLocale(locale).then(
          () => set({ locale }),
          (err) => console.error("Failed to activate locale:", err),
        );
      },
    }),
    {
      name: "swarmnote-ui",
      storage: createTauriStorage("settings.json"),
      partialize: (state) => ({
        sidebarOpen: state.sidebarOpen,
        sidebarWidth: state.sidebarWidth,
        theme: state.theme,
        locale: state.locale,
      }),
      onRehydrateStorage: () => (state) => {
        if (state) {
          const resolved = state.theme === "system" ? getSystemTheme() : state.theme;
          applyTheme(resolved);
          state.resolvedTheme = resolved;
          activateLocale(state.locale).catch(console.error);
        }
      },
    },
  ),
);

export const waitForUiHydration = () => waitForHydration(useUIStore);

// Listen to system theme changes when in "system" mode
window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
  const { theme, setTheme } = useUIStore.getState();
  if (theme === "system") {
    setTheme("system");
  }
});
