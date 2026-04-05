import { useLingui } from "@lingui/react/macro";
import { type KeyboardEvent, useCallback, useRef } from "react";
import { renameYDoc } from "@/commands/document";
import { useEditorStore } from "@/stores/editorStore";
import { useFileTreeStore } from "@/stores/fileTreeStore";

export function EditorTitle() {
  const { t } = useLingui();
  const title = useEditorStore((s) => s.title);
  const relPath = useEditorStore((s) => s.relPath);
  const updateRelPath = useEditorStore((s) => s.updateRelPath);
  const rename = useFileTreeStore((s) => s.rename);
  const inputRef = useRef<HTMLInputElement>(null);

  const commitTitle = useCallback(async () => {
    const raw = inputRef.current?.value.trim() || t`无标题`;
    const newTitle = raw.toLowerCase().endsWith(".md") ? raw : `${raw}.md`;
    if (newTitle === title) return;

    if (relPath) {
      const newRelPath = await rename(relPath, newTitle);
      updateRelPath(newRelPath, newTitle);

      // Notify Rust YDocManager about the path change (UUID stays the same)
      const docUuid = useEditorStore.getState().docUuid;
      if (docUuid) {
        await renameYDoc(docUuid, newRelPath);
      }
    }
  }, [title, relPath, updateRelPath, rename, t]);

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      inputRef.current?.blur();
    }
  };

  return (
    <input
      ref={inputRef}
      className="w-full border-none bg-transparent text-2xl font-bold text-foreground outline-none placeholder:text-muted-foreground"
      placeholder={t`无标题`}
      defaultValue={title.replace(/\.md$/i, "")}
      onBlur={commitTitle}
      onKeyDown={handleKeyDown}
    />
  );
}
