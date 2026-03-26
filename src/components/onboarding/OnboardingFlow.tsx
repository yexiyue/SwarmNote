import { CompleteStep } from "@/components/onboarding/CompleteStep";
import { DeviceNameStep } from "@/components/onboarding/DeviceNameStep";
import { OnboardingLayout, StepTransition } from "@/components/onboarding/OnboardingLayout";
import { WelcomeStep } from "@/components/onboarding/WelcomeStep";
import { useOnboardingStore } from "@/stores/onboardingStore";

const steps = [WelcomeStep, DeviceNameStep, CompleteStep];

export function OnboardingFlow() {
  const currentStep = useOnboardingStore((s) => s.currentStep);
  const StepComponent = steps[currentStep] ?? WelcomeStep;

  return (
    <OnboardingLayout>
      <StepTransition stepKey={currentStep}>
        <StepComponent />
      </StepTransition>
    </OnboardingLayout>
  );
}
