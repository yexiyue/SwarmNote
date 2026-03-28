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
import { useCallback, useEffect, useRef } from "react";
import type { Locale } from "@/i18n";
import { useEditorStore } from "@/stores/editorStore";
import { useUIStore } from "@/stores/uiStore";
import { EditorTitle } from "./EditorTitle";

const { image: _, video: _v, audio: _a, file: _f, ...supportedBlockSpecs } = defaultBlockSpecs;

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

  const dictionary = bnDictMap[locale];
  const editor = useCreateBlockNote({ schema, dictionary }, [locale]);

  // Load markdown content into editor on mount (one-time, non-reactive)
  useEffect(() => {
    const { markdown } = useEditorStore.getState();
    if (!markdown) return;
    async function load() {
      const blocks = await editor.tryParseMarkdownToBlocks(markdown);
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
