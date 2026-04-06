import { useLingui } from "@lingui/react/macro";
import { Check, WrapText } from "lucide-react";
import { useEffect, useRef } from "react";
import { NoteEditor } from "@/components/editor/NoteEditor";
import { EmptyState } from "@/components/layout/EmptyState";
import { StatusBar } from "@/components/layout/StatusBar";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { useEditorStore } from "@/stores/editorStore";
import { useUIStore } from "@/stores/uiStore";

export function EditorPane() {
  const { t } = useLingui();
  const currentDocId = useEditorStore((s) => s.currentDocId);
  const readableLineLength = useUIStore((s) => s.readableLineLength);
  const setReadableLineLength = useUIStore((s) => s.setReadableLineLength);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Sync scroll container ref to store after mount
  useEffect(() => {
    useEditorStore.getState().setScrollContainerRef(scrollRef.current);
    return () => {
      useEditorStore.getState().setScrollContainerRef(null);
    };
  }, []);

  return (
    <main className="flex min-w-0 flex-1 flex-col bg-background">
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <div ref={scrollRef} className="flex-1 overflow-auto py-10">
            {currentDocId ? <NoteEditor key={currentDocId} /> : <EmptyState />}
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem
            onClick={() => {
              const current = useUIStore.getState().readableLineLength;
              setReadableLineLength(!current);
            }}
          >
            <WrapText className="mr-2 h-4 w-4" />
            {t`可读行宽`}
            {readableLineLength && <Check className="ml-auto h-4 w-4" />}
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
      <StatusBar />
    </main>
  );
}
