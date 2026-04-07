import { Trans, useLingui } from "@lingui/react/macro";
import { createFileRoute } from "@tanstack/react-router";
import { FolderOpen, Globe, Palette, WrapText } from "lucide-react";
import { SettingRow } from "@/components/settings/SettingRow";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import type { Locale } from "@/i18n";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useUIStore } from "@/stores/uiStore";

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
        <Card>
          <CardHeader className="border-b">
            <CardTitle>{t`外观`}</CardTitle>
            <CardDescription>{t`自定义应用的显示方式`}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-1">
              <SettingRow icon={Globe} label={t`语言`} description={t`选择界面显示语言`}>
                <Select value={locale} onValueChange={(v) => setLocale(v as Locale)}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="zh">{t`中文`}</SelectItem>
                    <SelectItem value="en">English</SelectItem>
                  </SelectContent>
                </Select>
              </SettingRow>
              <Separator />
              <SettingRow icon={Palette} label={t`外观`} description={t`选择明亮或暗色主题`}>
                <Select
                  value={theme}
                  onValueChange={(v) => setTheme(v as "light" | "dark" | "system")}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="light">{t`浅色`}</SelectItem>
                    <SelectItem value="dark">{t`深色`}</SelectItem>
                    <SelectItem value="system">{t`跟随系统`}</SelectItem>
                  </SelectContent>
                </Select>
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
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="border-b">
            <CardTitle>{t`启动行为`}</CardTitle>
            <CardDescription>{t`控制应用启动时的默认行为`}</CardDescription>
          </CardHeader>
          <CardContent>
            <SettingRow
              icon={FolderOpen}
              label={t`恢复上次工作区`}
              description={t`启动时自动打开上次使用的工作区`}
            >
              <Switch checked={restoreLastWorkspace} onCheckedChange={setRestoreLastWorkspace} />
            </SettingRow>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/general")({
  component: GeneralSettingsPage,
});
