import { create } from "zustand";
import { persist } from "zustand/middleware";
import { createTauriStorage, waitForHydration } from "@/lib/tauriStore";

interface PreferencesState {
  autoStartP2P: boolean;
  restoreLastWorkspace: boolean;
}

interface PreferencesActions {
  setAutoStartP2P: (value: boolean) => void;
  setRestoreLastWorkspace: (value: boolean) => void;
}

export const usePreferencesStore = create<PreferencesState & PreferencesActions>()(
  persist(
    (set) => ({
      autoStartP2P: true,
      restoreLastWorkspace: true,

      setAutoStartP2P: (value: boolean) => set({ autoStartP2P: value }),
      setRestoreLastWorkspace: (value: boolean) => set({ restoreLastWorkspace: value }),
    }),
    {
      name: "swarmnote-preferences",
      storage: createTauriStorage("settings.json"),
    },
  ),
);

export const waitForPreferencesHydration = () => waitForHydration(usePreferencesStore);
