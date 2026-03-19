# 文件树 + CRUD

## 用户故事

作为用户，我希望在侧边栏中看到我所有的笔记和文件夹，以便快速浏览和管理笔记。

## 需求描述

侧边栏展示工作区的文件树结构，支持文件夹嵌套、新建/删除/重命名笔记和文件夹操作。

## 交互设计

### 用户操作流程

- **浏览**：侧边栏展示树形结构，文件夹可展开/折叠
- **新建笔记**：工具栏按钮或 Ctrl+N 快捷键，在当前文件夹下新建
- **新建文件夹**：工具栏按钮或右键菜单
- **删除**：右键菜单 → 确认对话框 → 删除文件/文件夹
- **重命名**：右键菜单 → 行内编辑 → 回车确认
- **移动**：拖拽文件/文件夹到其他文件夹（P1，可推迟）
- **点击打开**：单击笔记在编辑区打开

### 关键页面 / 组件

- `Sidebar` — 侧边栏容器
- `FileTree` — 文件树组件
- `FileTreeItem` — 文件/文件夹节点
- `FileTreeContextMenu` — 右键菜单
- `NewItemInput` — 行内新建/重命名输入框

### 文件显示规则

- 仅展示 `.md` 文件和文件夹
- 隐藏 `.swarmnote/` 目录
- 隐藏资源目录（与 .md 同名的文件夹，如 `笔记.md` 对应的 `笔记/` 目录）
- 按文件夹在前、文件在后排序，同级按名称排序

## 技术方案

### 前端

- 自定义树组件（基于 shadcn/ui 的基础组件构建）
- Zustand store 管理文件树状态（展开/折叠、选中、编辑模式）
- 拖拽功能可使用 `@dnd-kit/core`（P1）

### 后端

- `#[tauri::command] fn list_workspace_files(path)` — 递归读取工作区目录结构，返回文件树 JSON
- `#[tauri::command] fn create_document(parent_path, title)` — 创建新 .md 文件，写入数据库
- `#[tauri::command] fn create_folder(parent_path, name)` — 创建新文件夹
- `#[tauri::command] fn delete_item(path)` — 删除文件或文件夹（含确认）
- `#[tauri::command] fn rename_item(old_path, new_name)` — 重命名文件或文件夹
- `#[tauri::command] fn move_item(from_path, to_path)` — 移动文件或文件夹（P1）

### 数据结构

```typescript
interface FileTreeNode {
  id: string;
  name: string;
  type: "file" | "folder";
  path: string;         // 相对于工作区的路径
  children?: FileTreeNode[];
}
```

## 验收标准

- [ ] 侧边栏正确展示工作区目录的文件树结构
- [ ] 文件夹可展开/折叠
- [ ] 可新建笔记（工具栏按钮 + Ctrl+N）
- [ ] 可新建文件夹
- [ ] 可删除笔记和文件夹（带确认对话框）
- [ ] 可行内重命名笔记和文件夹
- [ ] 点击笔记在编辑区打开
- [ ] 隐藏 `.swarmnote/` 和资源目录
- [ ] 排序正确（文件夹在前，按名称排序）

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- 拖拽移动功能是否在 v0.1.0 实现？
  - 建议作为 P1，时间允许再做
- 大量文件（100+ 笔记）时的性能？
  - 可延迟加载，但 v0.1.0 暂不需要优化
