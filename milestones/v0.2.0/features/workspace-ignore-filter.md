# 工作区自定义忽略规则（.swarmnote-ignore）

## 用户故事

作为笔记用户，我希望通过 `.swarmnote-ignore` 文件自定义哪些文件/目录被忽略，以便文件树只展示我关心的内容。

## 依赖

- 无依赖（文件系统模块独立优化）

## 需求描述

当前 `scan.rs` 和 `watcher.rs` 中的文件过滤逻辑硬编码（跳过隐藏目录、仅 `.md` 文件），用户无法自定义忽略规则。

引入 `ignore` crate 的 `gitignore` 模块，让用户可以在工作区根目录放置 `.swarmnote-ignore` 文件，使用 `.gitignore` 语法自定义忽略模式。同时将 `scan.rs` 和 `watcher.rs` 中重复的过滤逻辑集中到统一的 `WorkspaceFilter` 模块。

## 交互设计

- 用户在工作区根目录创建 `.swarmnote-ignore` 文件
- 使用标准 `.gitignore` 语法编写忽略规则（如 `temp/`、`drafts/*.md`）
- 文件树自动排除匹配的文件/目录
- 文件监听器也尊重忽略规则，不触发不相关的刷新

## 技术方案

### 后端

- **新增 `src-tauri/src/fs/filter.rs`**：
  - `WorkspaceFilter` 结构体，封装 `ignore::gitignore::Gitignore`
  - `new(workspace_path)` — 加载 `.swarmnote-ignore`，不存在则仅用内置规则
  - `is_ignored(path, is_dir)` — 内置规则（隐藏目录）+ gitignore 规则 + `.md` 扩展名检查
- **重构 `scan.rs`**：`scan_dir` 使用 `WorkspaceFilter` 替换硬编码过滤
- **重构 `watcher.rs`**：`is_relevant_change` 使用 `WorkspaceFilter` 替换硬编码过滤
- **依赖**：`ignore = "0.4"`（仅使用 `gitignore` 子模块）

### 前端

- 无前端改动

## 验收标准

- [ ] 无 `.swarmnote-ignore` 文件时，行为与现有完全一致
- [ ] 创建 `.swarmnote-ignore` 并写入 `temp/`，文件树中不出现 `temp/` 目录
- [ ] 监听器也遵守忽略规则，`temp/` 内文件变更不触发 `fs:tree-changed`
- [ ] `scan.rs` 和 `watcher.rs` 中不再有重复的硬编码过滤逻辑
- [ ] 所有现有测试通过 + 新增 filter 模块单元测试
- [ ] `cargo clippy -- -D warnings` 无警告

## 任务拆分建议

1. 添加 `ignore` crate 依赖
2. 新建 `filter.rs`，实现 `WorkspaceFilter`
3. 重构 `scan.rs` 使用 `WorkspaceFilter`
4. 重构 `watcher.rs` 使用 `WorkspaceFilter`
5. 补充单元测试

## 开放问题

- `.swarmnote-ignore` 文件变更时是否需要热重载？（当前方案：切换工作区或重启时重新加载）
