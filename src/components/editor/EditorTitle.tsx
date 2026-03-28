import { type KeyboardEvent, useCallback, useRef } from "react";
import { useEditorStore } from "@/stores/editorStore";
import { useFileTreeStore } from "@/stores/fileTreeStore";

export function EditorTitle() {
  const title = useEditorStore((s) => s.title);
  const relPath = useEditorStore((s) => s.relPath);
  const updateTitle = useEditorStore((s) => s.updateTitle);
  const rename = useFileTreeStore((s) => s.rename);
  const inputRef = useRef<HTMLInputElement>(null);

  const commitTitle = useCallback(async () => {
    const raw = inputRef.current?.value.trim() || "Untitled";
    const newTitle = raw.endsWith(".md") ? raw : `${raw}.md`;
    if (newTitle === title) return;

    updateTitle(newTitle);
    if (relPath) {
      await rename(relPath, newTitle);
    }
  }, [title, relPath, updateTitle, rename]);

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      inputRef.current?.blur();
    }
  };

  return (
    <input
      ref={inputRef}
      className="w-full border-none bg-transparent text-3xl font-bold text-foreground outline-none placeholder:text-muted-foreground"
      placeholder="Untitled"
      defaultValue={title.replace(/\.md$/i, "")}
      onBlur={commitTitle}
      onKeyDown={handleKeyDown}
    />
  );
}
