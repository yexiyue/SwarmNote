import { Trans, useLingui } from "@lingui/react/macro";
import { useNavigate } from "@tanstack/react-router";
import { FileText, Plus, Settings, ToggleLeft } from "lucide-react";
import { useEffect, useState } from "react";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command";
import { modKey } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

export function CommandPalette() {
  const { t } = useLingui();
  const [open, setOpen] = useState(false);
  const navigate = useNavigate();
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);

  useEffect(() => {
    function handleOpen() {
      setOpen(true);
    }
    document.addEventListener("open-command-palette", handleOpen);
    return () => document.removeEventListener("open-command-palette", handleOpen);
  }, []);

  function runCommand(fn: () => void) {
    setOpen(false);
    fn();
  }

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <Command>
        <CommandInput placeholder={t`输入命令...`} />
        <CommandList>
          <CommandEmpty>
            <Trans>没有找到匹配的命令</Trans>
          </CommandEmpty>
          <CommandGroup heading={t`操作`}>
            <CommandItem onSelect={() => runCommand(() => {})}>
              <Plus className="h-4 w-4" />
              <Trans>新建笔记</Trans>
              <CommandShortcut>{modKey}N</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(toggleSidebar)}>
              <ToggleLeft className="h-4 w-4" />
              <Trans>切换侧边栏</Trans>
              <CommandShortcut>{modKey}B</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => navigate({ to: "/settings" }))}>
              <Settings className="h-4 w-4" />
              <Trans>打开设置</Trans>
              <CommandShortcut>{modKey},</CommandShortcut>
            </CommandItem>
          </CommandGroup>
          <CommandGroup heading={t`最近文件`}>
            <CommandItem onSelect={() => runCommand(() => {})}>
              <FileText className="h-4 w-4" />
              2026-03-21
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => {})}>
              <FileText className="h-4 w-4" />
              2026-03-19
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
