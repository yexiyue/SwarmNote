import { codeBlockOptions } from "@blocknote/code-block";
import {
  BlockNoteSchema,
  createCodeBlockSpec,
  type Dictionary,
  defaultBlockSpecs,
} from "@blocknote/core";
import { zh as bnZh } from "@blocknote/core/locales";
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import { useLingui } from "@lingui/react/macro";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { confirm } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import * as Y from "yjs";
import { openYDoc, reloadYDocConfirmed, saveMedia } from "@/commands/document";
import type { Locale } from "@/i18n";
import { TauriYjsProvider } from "@/lib/TauriYjsProvider";
import { useEditorStore } from "@/stores/editorStore";
import { useUIStore } from "@/stores/uiStore";
import { useWorkspaceStore } from "@/stores/workspaceStore";
import { CustomReactImageBlock } from "./CustomImageBlock";
import { CustomReactVideoBlock } from "./CustomVideoBlock";
import { EditorTitle } from "./EditorTitle";

const schema = BlockNoteSchema.create({
  blockSpecs: {
    ...defaultBlockSpecs,
    codeBlock: createCodeBlockSpec(codeBlockOptions),
    image: CustomReactImageBlock(),
    video: CustomReactVideoBlock(),
  },
});

const bnDictMap: Record<Locale, Dictionary | undefined> = {
  zh: bnZh,
  en: undefined,
};

interface YjsContext {
  ydoc: Y.Doc;
  provider: TauriYjsProvider;
}

/**
 * Outer component: initializes Y.Doc from Rust backend, then renders the
 * inner editor once ready. Remounted per document via `key={currentDocId}`
 * in EditorPane.
 */
export function NoteEditor() {
  const docId = useEditorStore((s) => s.currentDocId);
  const relPath = useEditorStore((s) => s.relPath);
  const workspace = useWorkspaceStore((s) => s.workspace);

  const [yjsCtx, setYjsCtx] = useState<YjsContext | null>(null);

  useEffect(() => {
    if (!workspace || !docId) return;
    let cancelled = false;

    const wsId = workspace.id;

    async function init() {
      const result = await openYDoc(relPath, wsId);

      if (cancelled) return;

      // Store the stable UUID for subsequent IPC calls
      useEditorStore.getState().setDocUuid(result.doc_uuid);

      const ydoc = new Y.Doc();
      Y.applyUpdate(ydoc, new Uint8Array(result.yjs_state));

      const provider = new TauriYjsProvider(ydoc, result.doc_uuid);
      setYjsCtx({ ydoc, provider });
    }

    init();

    return () => {
      cancelled = true;
      setYjsCtx((prev) => {
        if (prev) {
          prev.provider.destroy();
          prev.ydoc.destroy();
        }
        return null;
      });
    };
  }, [docId, relPath, workspace]);

  if (!yjsCtx) {
    return (
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-4">
        <div className="h-8" />
        <div className="animate-pulse text-sm text-muted-foreground">Loading...</div>
      </div>
    );
  }

  return <NoteEditorInner ydoc={yjsCtx.ydoc} provider={yjsCtx.provider} />;
}

/**
 * Inner component: creates BlockNote in collaboration mode using the
 * initialized Y.Doc and TauriYjsProvider.
 */
function NoteEditorInner({ ydoc, provider }: { ydoc: Y.Doc; provider: TauriYjsProvider }) {
  const { t } = useLingui();
  const resolvedTheme = useUIStore((s) => s.resolvedTheme);
  const locale = useUIStore((s) => s.locale);
  const markDirty = useEditorStore((s) => s.markDirty);
  const setCharCount = useEditorStore((s) => s.setCharCount);
  const docUuid = useEditorStore((s) => s.docUuid);

  // Stable refs for callbacks
  const markDirtyRef = useRef(markDirty);
  markDirtyRef.current = markDirty;
  const setCharCountRef = useRef(setCharCount);
  setCharCountRef.current = setCharCount;

  const wsPath = useWorkspaceStore.getState().workspace?.path ?? "";

  const uploadFile = useCallback(async (file: File): Promise<string> => {
    const relPath = useEditorStore.getState().relPath;
    const buffer = await file.arrayBuffer();
    const data = Array.from(new Uint8Array(buffer));
    // Returns workspace-relative path (e.g., "notes/my-note.assets/screenshot-af3b.png")
    return saveMedia(relPath, file.name, data);
  }, []);

  const resolveFileUrl = useCallback(
    async (url: string): Promise<string> => {
      if (
        url.startsWith("http://") ||
        url.startsWith("https://") ||
        url.startsWith("data:") ||
        url.startsWith("blob:")
      ) {
        return url;
      }
      // Convert workspace-relative path to tauri asset URL at render time
      return convertFileSrc(`${wsPath}/${url}`);
    },
    [wsPath],
  );

  const dictionary = bnDictMap[locale];
  const editor = useCreateBlockNote(
    {
      schema,
      dictionary,
      uploadFile,
      resolveFileUrl,
      collaboration: {
        provider,
        fragment: ydoc.getXmlFragment("document-store"),
        user: {
          name: "Local",
          color: "#3b82f6",
        },
      },
    },
    [locale],
  );

  // Track dirty state + debounced char count from Y.Doc updates (single listener)
  useEffect(() => {
    let charCountTimer: ReturnType<typeof setTimeout> | null = null;
    const handler = (_update: Uint8Array, origin: unknown) => {
      if (origin !== "remote") {
        markDirtyRef.current();
      }
      if (charCountTimer) clearTimeout(charCountTimer);
      charCountTimer = setTimeout(() => {
        let count = 0;
        editor._tiptapEditor.state.doc.descendants((node) => {
          if (node.isText && node.text) count += node.text.length;
        });
        setCharCountRef.current(count);
      }, 300);
    };
    ydoc.on("update", handler);
    handler(new Uint8Array(), null);
    return () => {
      ydoc.off("update", handler);
      if (charCountTimer) clearTimeout(charCountTimer);
    };
  }, [ydoc, editor]);

  // Listen for flush events from Rust (race-safe cleanup)
  useEffect(() => {
    if (!docUuid) return;
    const uuid = docUuid;

    let cancelled = false;
    const unlistenPromise = listen<{ docUuid: string }>("yjs:flushed", (event) => {
      if (!cancelled && event.payload.docUuid === uuid) {
        useEditorStore.getState().markFlushed(new Date());
      }
    });

    return () => {
      cancelled = true;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [docUuid]);

  // Apply external .md file changes (silent reload — document was not dirty)
  useEffect(() => {
    if (!docUuid) return;
    const uuid = docUuid;

    let cancelled = false;
    const unlistenPromise = listen<{ docUuid: string; update: number[] }>(
      "yjs:external-update",
      (event) => {
        if (!cancelled && event.payload.docUuid === uuid) {
          Y.applyUpdate(ydoc, new Uint8Array(event.payload.update), "remote");
        }
      },
    );

    return () => {
      cancelled = true;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [docUuid, ydoc]);

  // Handle external .md file changes when document has unsaved edits
  useEffect(() => {
    if (!docUuid) return;
    const uuid = docUuid;

    let cancelled = false;
    const unlistenPromise = listen<{ docUuid: string; relPath: string }>(
      "yjs:external-conflict",
      async (event) => {
        if (cancelled || event.payload.docUuid !== uuid) return;
        const confirmed = await confirm(
          t`"${event.payload.relPath}" 已被外部修改。是否重新加载？当前未保存的编辑将丢失。`,
          { title: t`文件已修改`, kind: "warning" },
        );
        if (confirmed && !cancelled) {
          await reloadYDocConfirmed(event.payload.docUuid);
        }
      },
    );

    return () => {
      cancelled = true;
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [docUuid, t]);

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-4">
      <EditorTitle />
      <BlockNoteView editor={editor} theme={resolvedTheme === "dark" ? "dark" : "light"} />
    </div>
  );
}
