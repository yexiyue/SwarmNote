import { useEffect } from "react";
import { OPEN_COMMAND_PALETTE } from "@/components/layout/CommandPalette";
import { isMac } from "@/lib/utils";
import { useUIStore } from "@/stores/uiStore";

export function useKeyboardShortcuts() {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const mod = isMac ? e.metaKey : e.ctrlKey;
      if (!mod) return;

      const { toggleSidebar, toggleSettings } = useUIStore.getState();

      switch (e.key.toLowerCase()) {
        case "b":
          e.preventDefault();
          toggleSidebar();
          break;
        case "n":
          e.preventDefault();
          // TODO: create new note
          break;
        case "s":
          e.preventDefault();
          // TODO: save current note
          break;
        case "p":
          e.preventDefault();
          document.dispatchEvent(new CustomEvent(OPEN_COMMAND_PALETTE));
          break;
        case ",":
          e.preventDefault();
          toggleSettings();
          break;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);
}
