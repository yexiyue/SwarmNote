import { create } from "zustand";
import { persist } from "zustand/middleware";
import { createTauriStorage, waitForHydration } from "@/lib/tauriStore";

interface PreferencesState {
  autoStartP2P: boolean;
}

interface PreferencesActions {
  setAutoStartP2P: (value: boolean) => void;
}

export const usePreferencesStore = create<PreferencesState & PreferencesActions>()(
  persist(
    (set) => ({
      autoStartP2P: true,

      setAutoStartP2P: (value: boolean) => set({ autoStartP2P: value }),
    }),
    {
      name: "swarmnote-preferences",
      storage: createTauriStorage("settings.json"),
    },
  ),
);

export const waitForPreferencesHydration = () => waitForHydration(usePreferencesStore);
