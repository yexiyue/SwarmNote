import { Trans, useLingui } from "@lingui/react/macro";
import { Monitor, Moon, Sun } from "lucide-react";
import { Label } from "@/components/ui/label";
import { type Locale, locales } from "@/i18n";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

const localeOptions = (Object.entries(locales) as [Locale, string][]).map(([value, label]) => ({
  value,
  label,
}));

function optionBtnClass(active: boolean) {
  return cn(
    "flex items-center justify-center gap-2 rounded-lg border px-4 py-2 text-sm whitespace-nowrap transition-colors",
    active
      ? "border-primary bg-primary/10 text-primary"
      : "border-border text-muted-foreground hover:bg-muted",
  );
}

export function GeneralSettings() {
  const { t } = useLingui();
  const theme = useUIStore((s) => s.theme);
  const setTheme = useUIStore((s) => s.setTheme);
  const locale = useUIStore((s) => s.locale);
  const setLocale = useUIStore((s) => s.setLocale);

  const themeOptions = [
    { value: "light" as const, label: t`浅色`, icon: Sun },
    { value: "dark" as const, label: t`深色`, icon: Moon },
    { value: "system" as const, label: t`跟随系统`, icon: Monitor },
  ];

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <Label>
          <Trans>语言</Trans>
        </Label>
        <div className="flex gap-2">
          {localeOptions.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() => setLocale(opt.value)}
              className={optionBtnClass(locale === opt.value)}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      <div className="space-y-2">
        <Label>
          <Trans>外观</Trans>
        </Label>
        <div className="flex gap-2">
          {themeOptions.map((opt) => (
            <button
              key={opt.value}
              type="button"
              onClick={() => setTheme(opt.value)}
              className={optionBtnClass(theme === opt.value)}
            >
              <opt.icon className="h-4 w-4" />
              {opt.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
