import { useLingui } from "@lingui/react/macro";
import { ask } from "@tauri-apps/plugin-dialog";
import { useCallback, useRef } from "react";
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

  const handleCreateFile = useCallback(
    async (parentRel: string) => {
      const relPath = await createFile(parentRel, t`新建笔记`);
      // Select the new file and enter rename mode
      setTimeout(() => {
        const node = treeRef.current?.get(relPath);
        if (node) {
          node.select();
          node.edit();
        }
      }, 100);
    },
    [createFile, t],
  );

  const handleCreateDir = useCallback(
    async (parentRel: string) => {
      const relPath = await createDir(parentRel, t`新建文件夹`);
      setTimeout(() => {
        const node = treeRef.current?.get(relPath);
        if (node) {
          node.select();
          node.edit();
        }
      }, 100);
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
          disableDrag
          disableDrop
          onRename={handleRenameSubmit}
          onActivate={handleActivate}
        >
          {(props) => (
            <FileTreeContextMenu
              node={props.node}
              onCreateFile={handleCreateFile}
              onCreateDir={handleCreateDir}
              onDelete={handleDelete}
              onRename={handleRename}
            >
              <div>
                <FileTreeNodeRenderer {...props} />
              </div>
            </FileTreeContextMenu>
          )}
        </Tree>
      </div>
    </FileTreeContextMenu>
  );
}
