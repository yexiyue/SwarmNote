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
import { convertFileSrc } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef } from "react";
import { saveMedia } from "@/commands/document";
import type { Locale } from "@/i18n";
import { useEditorStore } from "@/stores/editorStore";
import { useUIStore } from "@/stores/uiStore";
import { EditorTitle } from "./EditorTitle";

const { audio: _a, file: _f, ...supportedBlockSpecs } = defaultBlockSpecs;

const DEBOUNCE_MS = 1500;

const schema = BlockNoteSchema.create({
  blockSpecs: {
    ...supportedBlockSpecs,
    codeBlock: createCodeBlockSpec(codeBlockOptions),
  },
});

const bnDictMap: Record<Locale, Dictionary | undefined> = {
  zh: bnZh,
  en: undefined,
};

export function NoteEditor() {
  const updateContent = useEditorStore((s) => s.updateContent);
  const saveContent = useEditorStore((s) => s.saveContent);
  const resolvedTheme = useUIStore((s) => s.resolvedTheme);
  const locale = useUIStore((s) => s.locale);

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveContentRef = useRef(saveContent);
  saveContentRef.current = saveContent;

  const uploadFile = useCallback(async (file: File): Promise<string> => {
    const relPath = useEditorStore.getState().relPath;
    const buffer = await file.arrayBuffer();
    const data = Array.from(new Uint8Array(buffer));
    const absPath = await saveMedia(relPath, file.name, data);
    return convertFileSrc(absPath);
  }, []);

  const dictionary = bnDictMap[locale];
  const editor = useCreateBlockNote({ schema, dictionary, uploadFile }, [locale]);

  // Load markdown content into editor on mount (one-time, non-reactive)
  useEffect(() => {
    const { markdown } = useEditorStore.getState();
    async function load() {
      const blocks = markdown
        ? await editor.tryParseMarkdownToBlocks(markdown)
        : [{ type: "paragraph" as const }];
      editor.replaceBlocks(editor.document, blocks);
    }
    load();
  }, [editor]);

  // Flush pending save on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
        debounceRef.current = null;
        saveContentRef.current();
      }
    };
  }, []);

  const handleChange = useCallback(async () => {
    const md = await editor.blocksToMarkdownLossy(editor.document);
    updateContent(md);

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      debounceRef.current = null;
      saveContentRef.current();
    }, DEBOUNCE_MS);
  }, [editor, updateContent]);

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-4">
      <EditorTitle />
      <BlockNoteView
        editor={editor}
        theme={resolvedTheme === "dark" ? "dark" : "light"}
        onChange={handleChange}
      />
    </div>
  );
}
