import type { BlockNoteEditor } from "@blocknote/core";
import { Trans } from "@lingui/react/macro";
import { ListTree } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { cn } from "@/lib/utils";
import { useEditorStore } from "@/stores/editorStore";

interface HeadingItem {
  id: string;
  level: number;
  text: string;
}

function extractHeadings(editor: BlockNoteEditor): HeadingItem[] {
  const headings: HeadingItem[] = [];
  for (const block of editor.document) {
    if (block.type === "heading") {
      const text = (block.content as { type: string; text?: string }[] | undefined)
        ?.map((c) => c.text ?? "")
        .join("")
        .trim();
      if (text) {
        headings.push({
          id: block.id,
          level: (block.props as { level?: number })?.level ?? 1,
          text,
        });
      }
    }
  }
  return headings;
}

interface DocumentOutlineProps {
  height: number;
}

export function DocumentOutline({ height }: DocumentOutlineProps) {
  const editor = useEditorStore((s) => s.editorInstance);
  const scrollContainer = useEditorStore((s) => s.scrollContainerRef);

  const [headings, setHeadings] = useState<HeadingItem[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);
  const headingElementsRef = useRef<Map<string, IntersectionObserverEntry>>(new Map());
  const headingsRef = useRef(headings);
  headingsRef.current = headings;
  const isScrollingRef = useRef(false);

  // Extract headings from editor document
  const updateHeadings = useCallback(() => {
    if (!editor) {
      setHeadings([]);
      return;
    }
    setHeadings(extractHeadings(editor));
  }, [editor]);

  // Initial extraction + subscribe to DOM changes via MutationObserver
  useEffect(() => {
    updateHeadings();

    if (!scrollContainer) return;

    let timer: ReturnType<typeof setTimeout>;
    const mutationObserver = new MutationObserver(() => {
      clearTimeout(timer);
      timer = setTimeout(updateHeadings, 300);
    });
    mutationObserver.observe(scrollContainer, {
      childList: true,
      subtree: true,
      characterData: true,
    });
    return () => {
      clearTimeout(timer);
      mutationObserver.disconnect();
    };
  }, [scrollContainer, updateHeadings]);

  // Stable key: only recreate IntersectionObserver when heading IDs change
  const headingIds = useMemo(() => headings.map((h) => h.id).join(","), [headings]);

  // IntersectionObserver for active heading tracking
  useEffect(() => {
    if (!scrollContainer || headingIds === "") return;

    headingElementsRef.current.clear();
    const currentHeadings = headingsRef.current;

    observerRef.current = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const id = entry.target.getAttribute("data-id");
          if (id) {
            headingElementsRef.current.set(id, entry);
          }
        }

        const hs = headingsRef.current;
        let firstVisibleId: string | null = null;
        for (const heading of hs) {
          const entry = headingElementsRef.current.get(heading.id);
          if (entry?.isIntersecting) {
            firstVisibleId = heading.id;
            break;
          }
        }

        if (!firstVisibleId) {
          for (let i = hs.length - 1; i >= 0; i--) {
            const entry = headingElementsRef.current.get(hs[i].id);
            if (entry && entry.boundingClientRect.top < (entry.rootBounds?.top ?? 0)) {
              firstVisibleId = hs[i].id;
              break;
            }
          }
        }

        if (firstVisibleId && !isScrollingRef.current) {
          setActiveId(firstVisibleId);
        }
      },
      {
        root: scrollContainer,
        rootMargin: "-10% 0px -70% 0px",
        threshold: [0, 0.5, 1],
      },
    );

    for (const heading of currentHeadings) {
      const el = scrollContainer.querySelector(
        `[data-node-type="blockContainer"][data-id="${heading.id}"]`,
      );
      if (el) {
        observerRef.current.observe(el);
      }
    }

    return () => {
      observerRef.current?.disconnect();
      observerRef.current = null;
    };
  }, [headingIds, scrollContainer]);

  const handleClick = useCallback(
    (id: string) => {
      if (!scrollContainer) return;
      const el = scrollContainer.querySelector(
        `[data-node-type="blockContainer"][data-id="${id}"]`,
      );
      if (el) {
        isScrollingRef.current = true;
        setActiveId(id);
        el.scrollIntoView({ behavior: "smooth", block: "start" });
        setTimeout(() => {
          isScrollingRef.current = false;
        }, 600);
      }
    },
    [scrollContainer],
  );

  if (!editor) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
        <ListTree className="h-8 w-8 opacity-30" />
        <p className="text-xs">
          <Trans>打开文档以查看大纲</Trans>
        </p>
      </div>
    );
  }

  if (headings.length === 0) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
        <ListTree className="h-8 w-8 opacity-30" />
        <p className="px-4 text-center text-xs">
          <Trans>在文档中添加标题即可看到大纲导航</Trans>
        </p>
      </div>
    );
  }

  return (
    <nav className="flex flex-col overflow-y-auto" style={{ maxHeight: height }}>
      {headings.map((h) => (
        <button
          key={h.id}
          type="button"
          onClick={() => handleClick(h.id)}
          title={h.text}
          className={cn(
            "flex items-center truncate rounded px-2 text-left text-[13px] leading-[28px] transition-colors",
            h.level === 1 ? "pl-2 font-medium" : h.level === 2 ? "pl-5" : "pl-8",
            activeId === h.id
              ? "bg-sidebar-accent text-sidebar-accent-foreground"
              : "text-sidebar-foreground hover:bg-sidebar-accent/50",
          )}
        >
          {h.text}
        </button>
      ))}
    </nav>
  );
}
