import { ChevronDown, ChevronRight, FileText, Folder, FolderOpen } from "lucide-react";
import type { NodeRendererProps } from "react-arborist";
import type { FileTreeNode as FileTreeNodeData } from "@/commands/fs";
import { cn } from "@/lib/utils";

function RenameInput({ node }: { node: NodeRendererProps<FileTreeNodeData>["node"] }) {
  return (
    <input
      // biome-ignore lint/a11y/noAutofocus: inline rename requires immediate focus
      autoFocus
      type="text"
      defaultValue={node.data.name}
      className="h-5 w-full rounded border border-primary bg-background px-1 text-[13px] outline-none"
      onFocus={(e) => e.currentTarget.select()}
      onBlur={() => node.reset()}
      onKeyDown={(e) => {
        if (e.key === "Escape") node.reset();
        if (e.key === "Enter") node.submit(e.currentTarget.value);
      }}
    />
  );
}

export function FileTreeNodeRenderer({
  node,
  style,
  dragHandle,
}: NodeRendererProps<FileTreeNodeData>) {
  const isSelected = node.isSelected;

  const handleClick = (e: React.MouseEvent) => {
    if (node.isInternal) {
      node.toggle();
    } else {
      node.select();
      node.activate();
    }
    e.stopPropagation();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleClick(e as unknown as React.MouseEvent);
  };

  return (
    <div
      ref={dragHandle}
      style={style}
      role="treeitem"
      tabIndex={-1}
      className={cn(
        "flex items-center gap-1 rounded px-2 py-[5px] text-[13px] cursor-default",
        isSelected
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground hover:bg-sidebar-accent/50",
      )}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
    >
      {node.isInternal ? (
        node.isOpen ? (
          <ChevronDown className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        )
      ) : (
        <span className="w-3.5 shrink-0" />
      )}

      {node.isInternal ? (
        node.isOpen ? (
          <FolderOpen className="h-3.5 w-3.5 shrink-0 text-primary" />
        ) : (
          <Folder className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
        )
      ) : (
        <FileText
          className={cn(
            "h-3.5 w-3.5 shrink-0",
            isSelected ? "text-primary" : "text-muted-foreground",
          )}
        />
      )}

      {node.isEditing ? (
        <RenameInput node={node} />
      ) : (
        <span className="truncate">{node.data.name}</span>
      )}
    </div>
  );
}
