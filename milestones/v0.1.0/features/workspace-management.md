# 工作区管理

## 用户故事

作为用户，我希望将笔记组织在一个指定的目录中，以便管理和备份。

## 需求描述

工作区是 SwarmNote 的核心概念——一个包含所有笔记的本地目录。用户可以选择任意目录作为工作区，SwarmNote 在其下创建 `.swarmnote/` 隐藏目录存放配置和数据库。

## 交互设计

### 用户操作流程

1. 在 Onboarding 中选择工作区目录
2. 应用启动时自动加载上次使用的工作区
3. v0.1.0 暂不支持多工作区切换（后续版本添加）

### 关键页面 / 组件

- Onboarding 中的工作区选择步骤（复用 Onboarding 组件）
- 侧边栏顶部展示工作区名称

## 技术方案

### 前端

- Zustand store 存储当前工作区信息（路径、名称、ID）
- 侧边栏顶部展示工作区名称（取目录名）

### 后端

- `#[tauri::command] fn init_workspace(path)` — 创建 `.swarmnote/` 目录结构：
  - `.swarmnote/workspace.db` — SQLite 数据库
  - `.swarmnote/config.toml` — 工作区级配置
- `#[tauri::command] fn open_workspace(path)` — 打开已存在的工作区，加载数据库
- `#[tauri::command] fn get_workspace_info()` — 返回当前工作区信息

### 数据结构

```rust
struct WorkspaceInfo {
    id: String,        // UUID v7
    name: String,      // 目录名
    path: String,      // 绝对路径
    created_at: String,
}
```

### 目录结构

```
~/Notes/我的笔记/           ← 工作区根目录（用户选择）
├── .swarmnote/             ← SwarmNote 配置（隐藏）
│   ├── workspace.db        ← SQLite 数据库
│   └── config.toml         ← 工作区配置
├── 日记/                   ← 用户创建的文件夹
│   └── 2026-03-19.md
├── 项目笔记/
│   └── SwarmNote 开发.md
└── 快速笔记.md
```

## 验收标准

- [ ] 可选择任意目录作为工作区
- [ ] 选择后自动创建 `.swarmnote/` 目录和 `workspace.db`
- [ ] 应用重启后自动加载上次工作区
- [ ] 侧边栏顶部正确展示工作区名称
- [ ] 工作区路径存储在全局配置 `~/.swarmnote/` 中

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- 工作区目录不存在时（被移动/删除）如何处理？
  - 建议：提示用户重新选择或创建新工作区
