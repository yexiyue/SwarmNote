# 多工作区 per-window 状态管理

## 用户故事

作为用户，我希望能同时打开多个工作区窗口，每个窗口独立运行各自的工作区，互不干扰。

## 依赖

- 无依赖（L0，纯后端重构）
- 详见 [GitHub Issue #22](https://github.com/yexiyue/SwarmNote/issues/22)

## 需求描述

将后端的全局单例工作区状态改为 per-window 管理：
- `DbState.workspace_db` → `RwLock<HashMap<String, DatabaseConnection>>`
- `WorkspaceState` → `RwLock<HashMap<String, WorkspaceInfo>>`
- fs watcher 改为 per-window 实例
- 窗口关闭时自动清理资源
- 防止同一工作区在多个窗口中同时打开

## 技术方案

### 后端

- 状态结构改造：按窗口 label 索引的 HashMap
- 所有 workspace/fs 命令添加 `window: tauri::Window` 参数
- 新增 `open_workspace_window(path)` command
- 窗口关闭事件监听 → 清理 DB 连接、WorkspaceInfo、fs watcher
- 同一工作区路径互斥检查 → focus 已有窗口

### 前端

- 适配新的 window-scoped API
- Capabilities 更新支持动态窗口（`ws-*` 通配）

## 验收标准

- [ ] `DbState` 和 `WorkspaceState` 使用 `HashMap<String, T>` 按窗口 label 管理
- [ ] 所有 workspace/fs 相关 command 操作对应窗口的状态
- [ ] `open_workspace_window(path)` 可创建新窗口并预绑定工作区
- [ ] 窗口关闭时自动清理后端资源
- [ ] 同一工作区路径不能在多个窗口中同时打开
- [ ] `cargo clippy -- -D warnings` 无警告

## 开放问题

- 详细设计见 `openspec/changes/multi-workspace-windows/design.md`
