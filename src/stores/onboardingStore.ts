import { create } from "zustand";
import { persist } from "zustand/middleware";
import { createTauriStorage, waitForHydration } from "@/lib/tauriStore";

/** Step indices — keep in sync with OnboardingFlow.steps array */
const STEP_PATH_CHOICE = 2;
const STEP_COMPLETE = 4;

interface OnboardingState {
  isCompleted: boolean;
  currentStep: number;
  /** 是否在引导流程中完成了配对（不持久化，重启后重置） */
  pairedInOnboarding: boolean;
  /** 用户选择的路径：全新开始 or 添加设备 */
  userPath: "new" | "add-device" | null;
}

interface OnboardingActions {
  nextStep: () => void;
  prevStep: () => void;
  complete: () => void;
  reset: () => void;
  setPairedInOnboarding: (value: boolean) => void;
  setUserPath: (path: "new" | "add-device") => void;
}

export const useOnboardingStore = create<OnboardingState & OnboardingActions>()(
  persist(
    (set) => ({
      isCompleted: false,
      currentStep: 0,
      pairedInOnboarding: false,
      userPath: null,

      nextStep: () =>
        set((s) => {
          // PathChoiceStep + userPath "new" → skip PairingStep, jump to Complete
          if (s.currentStep === STEP_PATH_CHOICE && s.userPath === "new") {
            return { currentStep: STEP_COMPLETE };
          }
          return { currentStep: s.currentStep + 1 };
        }),

      prevStep: () => set((s) => ({ currentStep: Math.max(0, s.currentStep - 1) })),

      complete: () => set({ isCompleted: true }),

      reset: () =>
        set({ isCompleted: false, currentStep: 0, userPath: null, pairedInOnboarding: false }),

      setPairedInOnboarding: (value) => set({ pairedInOnboarding: value }),

      setUserPath: (path) => set({ userPath: path }),
    }),
    {
      name: "swarmnote-onboarding",
      storage: createTauriStorage("settings.json"),
      partialize: (state) => ({
        isCompleted: state.isCompleted,
        userPath: state.userPath,
      }),
    },
  ),
);

export const waitForOnboardingHydration = () => waitForHydration(useOnboardingStore);
