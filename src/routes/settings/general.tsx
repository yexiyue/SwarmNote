import { Trans, useLingui } from "@lingui/react/macro";
import { createFileRoute } from "@tanstack/react-router";
import { FolderOpen, Globe, Palette, WrapText } from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import type { Locale } from "@/i18n";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useUIStore } from "@/stores/uiStore";

function SettingCard({
  children,
  title,
  description,
}: {
  children: React.ReactNode;
  title: string;
  description?: string;
}) {
  return (
    <div className="rounded-xl border bg-card">
      <div className="px-5 py-4">
        <h3 className="text-sm font-medium">{title}</h3>
        {description && <p className="mt-0.5 text-xs text-muted-foreground">{description}</p>}
      </div>
      <Separator />
      <div className="px-5 py-3">{children}</div>
    </div>
  );
}

function SettingRow({
  icon: Icon,
  label,
  description,
  children,
}: {
  icon?: React.ComponentType<{ className?: string }>;
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex items-center gap-3">
        {Icon && (
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-muted">
            <Icon className="h-4 w-4 text-muted-foreground" />
          </div>
        )}
        <div>
          <div className="text-sm">{label}</div>
          {description && <div className="text-xs text-muted-foreground">{description}</div>}
        </div>
      </div>
      {children}
    </div>
  );
}

function StyledSelect({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="rounded-lg border bg-background px-3 py-1.5 text-sm outline-none transition-colors hover:border-foreground/20 focus:ring-2 focus:ring-ring"
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}

function GeneralSettingsPage() {
  const { t } = useLingui();
  const theme = useUIStore((s) => s.theme);
  const locale = useUIStore((s) => s.locale);
  const setTheme = useUIStore((s) => s.setTheme);
  const setLocale = useUIStore((s) => s.setLocale);

  const readableLineLength = useUIStore((s) => s.readableLineLength);
  const setReadableLineLength = useUIStore((s) => s.setReadableLineLength);

  const restoreLastWorkspace = usePreferencesStore((s) => s.restoreLastWorkspace);
  const setRestoreLastWorkspace = usePreferencesStore((s) => s.setRestoreLastWorkspace);

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-semibold tracking-tight">
          <Trans>通用</Trans>
        </h1>
        <p className="mt-1 text-sm text-muted-foreground">
          <Trans>管理界面外观和语言偏好</Trans>
        </p>
      </div>

      <div className="space-y-4">
        <SettingCard title={t`外观`} description={t`自定义应用的显示方式`}>
          <div className="space-y-1">
            <SettingRow icon={Globe} label={t`语言`} description={t`选择界面显示语言`}>
              <StyledSelect
                value={locale}
                onChange={(v) => setLocale(v as Locale)}
                options={[
                  { value: "zh", label: t`中文` },
                  { value: "en", label: "English" },
                ]}
              />
            </SettingRow>
            <Separator />
            <SettingRow icon={Palette} label={t`外观`} description={t`选择明亮或暗色主题`}>
              <StyledSelect
                value={theme}
                onChange={(v) => setTheme(v as "light" | "dark" | "system")}
                options={[
                  { value: "light", label: t`浅色` },
                  { value: "dark", label: t`深色` },
                  { value: "system", label: t`跟随系统` },
                ]}
              />
            </SettingRow>
            <Separator />
            <SettingRow
              icon={WrapText}
              label={t`可读行宽`}
              description={t`限制编辑器内容宽度以提升阅读体验`}
            >
              <Switch checked={readableLineLength} onCheckedChange={setReadableLineLength} />
            </SettingRow>
          </div>
        </SettingCard>

        <SettingCard title={t`启动行为`} description={t`控制应用启动时的默认行为`}>
          <SettingRow
            icon={FolderOpen}
            label={t`恢复上次工作区`}
            description={t`启动时自动打开上次使用的工作区`}
          >
            <Switch checked={restoreLastWorkspace} onCheckedChange={setRestoreLastWorkspace} />
          </SettingRow>
        </SettingCard>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/general")({
  component: GeneralSettingsPage,
});
