import { createFileRoute } from "@tanstack/react-router";

import type { Locale } from "@/i18n";
import { useUIStore } from "@/stores/uiStore";

function GeneralSettingsPage() {
  const theme = useUIStore((s) => s.theme);
  const locale = useUIStore((s) => s.locale);
  const setTheme = useUIStore((s) => s.setTheme);
  const setLocale = useUIStore((s) => s.setLocale);

  return (
    <div className="p-6">
      <h1 className="mb-1 text-lg font-semibold">通用</h1>
      <p className="mb-6 text-sm text-muted-foreground">外观和语言设置</p>

      <div className="space-y-6">
        {/* Language */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">语言</div>
            <div className="text-xs text-muted-foreground">选择界面显示语言</div>
          </div>
          <select
            value={locale}
            onChange={(e) => setLocale(e.target.value as Locale)}
            className="rounded-md border bg-background px-3 py-1.5 text-sm"
          >
            <option value="zh">中文</option>
            <option value="en">English</option>
          </select>
        </div>

        {/* Theme */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">外观</div>
            <div className="text-xs text-muted-foreground">选择明亮或暗色主题</div>
          </div>
          <select
            value={theme}
            onChange={(e) => setTheme(e.target.value as "light" | "dark" | "system")}
            className="rounded-md border bg-background px-3 py-1.5 text-sm"
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
            <option value="system">跟随系统</option>
          </select>
        </div>
      </div>
    </div>
  );
}

export const Route = createFileRoute("/settings/general")({
  component: GeneralSettingsPage,
});
