import { useLingui } from "@lingui/react/macro";
import {
  FilePlus,
  FileText,
  type LucideIcon,
  PanelLeft,
  Settings as SettingsIcon,
} from "lucide-react";
import { useMemo } from "react";
import { openSettingsWindow } from "@/commands/workspace";
import { modKey } from "@/lib/utils";
import { useEditorStore } from "@/stores/editorStore";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";

export type CommandCategory = "action" | "recent";

export interface Command {
  id: string;
  label: string;
  category: CommandCategory;
  shortcut?: string;
  icon: LucideIcon;
  run: () => void | Promise<void>;
}

/**
 * Returns the full list of Command Palette entries for the current state.
 * Built as a hook so every entry's translations and handlers stay reactive.
 * Recent files come from editorStore.recentDocs for the active workspace.
 */
export function useCommands(): { actions: Command[]; recents: Command[] } {
  const { t } = useLingui();

  return useMemo<{ actions: Command[]; recents: Command[] }>(() => {
    const actions: Command[] = [
      {
        id: "new-note",
        label: t`新建笔记`,
        category: "action",
        icon: FilePlus,
        shortcut: `${modKey}N`,
        run: async () => {
          const createAndOpenFile = useFileTreeStore.getState().createAndOpenFile;
          await createAndOpenFile("", t`新建笔记`);
        },
      },
      {
        id: "toggle-sidebar",
        label: t`切换侧边栏`,
        category: "action",
        icon: PanelLeft,
        shortcut: `${modKey}B`,
        run: () => {
          useUIStore.getState().toggleSidebar();
        },
      },
      {
        id: "open-settings",
        label: t`打开设置`,
        category: "action",
        icon: SettingsIcon,
        shortcut: `${modKey},`,
        run: async () => {
          await openSettingsWindow("general");
        },
      },
    ];

    const workspaceId = useWorkspaceStore.getState().workspace?.id;
    const recentDocs = workspaceId ? (useEditorStore.getState().recentDocs[workspaceId] ?? []) : [];

    const recents: Command[] = recentDocs.map((doc) => ({
      id: `recent:${doc.id}`,
      label: doc.title.replace(/\.md$/i, ""),
      category: "recent",
      icon: FileText,
      run: () => {
        useEditorStore.getState().loadDocument(doc.id, doc.title, doc.relPath);
      },
    }));

    return { actions, recents };
  }, [t]);
}
