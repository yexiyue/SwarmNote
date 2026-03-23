import { create } from "zustand";
import { persist } from "zustand/middleware";
import { createTauriStorage, waitForHydration } from "@/lib/tauriStore";

interface OnboardingState {
  isCompleted: boolean;
  currentStep: number;
}

interface OnboardingActions {
  nextStep: () => void;
  prevStep: () => void;
  complete: () => void;
  reset: () => void;
}

export const useOnboardingStore = create<OnboardingState & OnboardingActions>()(
  persist(
    (set) => ({
      isCompleted: false,
      currentStep: 0,

      nextStep: () => set((s) => ({ currentStep: s.currentStep + 1 })),

      prevStep: () => set((s) => ({ currentStep: Math.max(0, s.currentStep - 1) })),

      complete: () => set({ isCompleted: true }),

      reset: () => set({ isCompleted: false, currentStep: 0 }),
    }),
    {
      name: "swarmnote-onboarding",
      storage: createTauriStorage("settings.json"),
      partialize: (state) => ({
        isCompleted: state.isCompleted,
      }),
    },
  ),
);

export const waitForOnboardingHydration = () => waitForHydration(useOnboardingStore);
