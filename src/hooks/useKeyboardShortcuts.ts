import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { useUIStore } from "@/stores/uiStore";

export function useKeyboardShortcuts() {
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const navigate = useNavigate();

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const mod = navigator.platform.includes("Mac") ? e.metaKey : e.ctrlKey;
      if (!mod) return;

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
          document.dispatchEvent(new CustomEvent("open-command-palette"));
          break;
        case ",":
          e.preventDefault();
          navigate({ to: "/settings" });
          break;
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [toggleSidebar, navigate]);
}
