import { useLingui } from "@lingui/react/macro";
import { ask } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef } from "react";
import type { NodeApi } from "react-arborist";
import { Tree, type TreeApi } from "react-arborist";
import type { FileTreeNode } from "@/commands/fs";
import { useEditorStore } from "@/stores/editorStore";
import { useFileTreeStore } from "@/stores/fileTreeStore";
import { EmptyTreeState } from "./EmptyTreeState";
import { FileTreeContextMenu } from "./FileTreeContextMenu";
import { FileTreeNodeRenderer } from "./FileTreeNode";

interface FileTreeProps {
  width: number;
  height: number;
}

export function FileTree({ width, height }: FileTreeProps) {
  const { t } = useLingui();
  const tree = useFileTreeStore((s) => s.tree);
  const isLoading = useFileTreeStore((s) => s.isLoading);
  const createFile = useFileTreeStore((s) => s.createFile);
  const createDir = useFileTreeStore((s) => s.createDir);
  const deleteFile = useFileTreeStore((s) => s.deleteFile);
  const deleteDir = useFileTreeStore((s) => s.deleteDir);
  const rename = useFileTreeStore((s) => s.rename);
  const selectFile = useFileTreeStore((s) => s.selectFile);
  const loadDocument = useEditorStore((s) => s.loadDocument);

  const treeRef = useRef<TreeApi<FileTreeNode>>(null);

  // Deterministic rename-after-create: the `createFile`/`createDir` store
  // actions update `tree` asynchronously via the Rust backend + rescan, so
  // the NodeApi we want to enter edit mode on doesn't exist yet when we
  // return. We stash the desired id in a ref and let a `useEffect` fire
  // whenever `tree` changes — as soon as the node is present, we enter
  // edit mode. No `setTimeout` needed.
  const pendingEditRef = useRef<string | null>(null);

  // `tree` is used as a render-signal dependency (we don't read from it
  // directly — we use `treeRef.current?.get()` — but we must re-run on
  // every change so react-arborist has time to reconcile).
  // biome-ignore lint/correctness/useExhaustiveDependencies: see above
  useEffect(() => {
    const pending = pendingEditRef.current;
    if (!pending) return;
    const node = treeRef.current?.get(pending);
    if (node) {
      pendingEditRef.current = null;
      node.select();
      node.edit();
    }
  }, [tree]);

  const handleCreateFile = useCallback(
    async (parentRel: string) => {
      const relPath = await createFile(parentRel, t`新建笔记`);
      pendingEditRef.current = relPath;
    },
    [createFile, t],
  );

  const handleCreateDir = useCallback(
    async (parentRel: string) => {
      const relPath = await createDir(parentRel, t`新建文件夹`);
      pendingEditRef.current = relPath;
    },
    [createDir, t],
  );

  const handleDelete = useCallback(
    async (node: NodeApi<FileTreeNode>) => {
      const isFolder = node.isInternal;
      const message = isFolder
        ? t`确定要删除文件夹 "${node.data.name}" 及其所有内容吗？`
        : t`确定要删除 "${node.data.name}" 吗？`;

      const confirmed = await ask(message, {
        title: t`确认删除`,
        kind: "warning",
        okLabel: t`删除`,
        cancelLabel: t`取消`,
      });

      if (!confirmed) return;

      if (isFolder) {
        await deleteDir(node.data.id);
      } else {
        await deleteFile(node.data.id);
      }
    },
    [deleteFile, deleteDir, t],
  );

  const handleRename = useCallback((node: NodeApi<FileTreeNode>) => {
    node.edit();
  }, []);

  const handleRenameSubmit = useCallback(
    async ({ id, name: newName }: { id: string; name: string }) => {
      await rename(id, newName);
    },
    [rename],
  );

  const move = useFileTreeStore((s) => s.move);

  /**
   * react-arborist drop handler. `parentId` is the destination folder's id
   * (which is its rel_path) or `null` for the workspace root. `dragIds` are
   * the rel_paths of the nodes being moved.
   *
   * For each dragged node we compute a target path = parent + "/" + basename,
   * and reject no-op moves (target === source) and any case where the target
   * would be a descendant of the source (enforced server-side too).
   */
  const handleMove = useCallback(
    async ({ dragIds, parentId }: { dragIds: string[]; parentId: string | null }) => {
      const parentRel = parentId ?? "";
      for (const from of dragIds) {
        const basename = from.split("/").pop() ?? from;
        const to = parentRel ? `${parentRel}/${basename}` : basename;
        if (to === from) continue;
        // Fast-fail: the backend rejects folder-into-descendant too, but
        // skipping here avoids a wasted IPC round-trip.
        if (to.startsWith(`${from}/`)) continue;
        try {
          await move(from, to);
        } catch (err) {
          console.error(`Failed to move ${from} → ${to}:`, err);
        }
      }
    },
    [move],
  );

  const disableDropTarget = useCallback(
    ({
      parentNode,
      dragNodes,
    }: {
      parentNode: NodeApi<FileTreeNode> | null;
      dragNodes: NodeApi<FileTreeNode>[];
    }) => {
      // Only folders (and the root, which has parentNode === null) are valid
      // drop targets. react-arborist gives us a null parentNode when hovering
      // the root — we allow that.
      if (parentNode && !parentNode.isInternal) return true;
      // Reject dropping a folder onto one of its own descendants.
      const parentId = parentNode?.id ?? "";
      return dragNodes.some((n) => parentId.startsWith(`${n.id}/`) || parentId === n.id);
    },
    [],
  );

  const handleActivate = useCallback(
    (node: NodeApi<FileTreeNode>) => {
      if (node.isLeaf) {
        selectFile(node.data.id);
        loadDocument(node.data.id, node.data.name, node.data.id);
      }
    },
    [selectFile, loadDocument],
  );

  if (isLoading && tree.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-muted-foreground/30 border-t-primary" />
      </div>
    );
  }

  if (tree.length === 0) {
    return <EmptyTreeState />;
  }

  return (
    <FileTreeContextMenu
      node={null}
      onCreateFile={handleCreateFile}
      onCreateDir={handleCreateDir}
      onDelete={handleDelete}
      onRename={handleRename}
    >
      <div className="flex-1">
        <Tree<FileTreeNode>
          ref={treeRef}
          data={tree}
          width={width}
          height={height}
          indent={16}
          rowHeight={28}
          openByDefault={false}
          disableDrop={disableDropTarget}
          onRename={handleRenameSubmit}
          onActivate={handleActivate}
          onMove={handleMove}
        >
          {(props) => (
            <FileTreeContextMenu
              node={props.node}
              onCreateFile={handleCreateFile}
              onCreateDir={handleCreateDir}
              onDelete={handleDelete}
              onRename={handleRename}
            >
              {/* stopPropagation prevents outer ContextMenu from intercepting */}
              <div role="none" onContextMenu={(e) => e.stopPropagation()}>
                <FileTreeNodeRenderer {...props} />
              </div>
            </FileTreeContextMenu>
          )}
        </Tree>
      </div>
    </FileTreeContextMenu>
  );
}
