import { create } from "zustand";
import { persist } from "zustand/middleware";
import { activateLocale, detectLocale, type Locale } from "@/i18n";
import { createTauriStorage, waitForHydration } from "@/lib/tauriStore";

type Theme = "light" | "dark" | "system";

interface UIState {
  sidebarOpen: boolean;
  settingsOpen: boolean;
  theme: Theme;
  resolvedTheme: "light" | "dark";
  locale: Locale;
}

interface UIActions {
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  toggleSettings: () => void;
  setSettingsOpen: (open: boolean) => void;
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
      settingsOpen: false,
      theme: "light",
      resolvedTheme: "light",
      locale: detectLocale(),

      toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),

      setSidebarOpen: (open) => set({ sidebarOpen: open }),

      toggleSettings: () => set((s) => ({ settingsOpen: !s.settingsOpen })),

      setSettingsOpen: (open) => set({ settingsOpen: open }),

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
