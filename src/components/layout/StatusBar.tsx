import { useEditorStore } from "@/stores/editorStore";

export function StatusBar() {
  const relPath = useEditorStore((s) => s.relPath);

  return (
    <footer className="flex h-7 shrink-0 items-center justify-between border-t border-border bg-card px-4">
      <div className="flex items-center gap-3">
        {relPath && <span className="text-[11px] text-muted-foreground">{relPath}</span>}
      </div>
      <div />
    </footer>
  );
}
