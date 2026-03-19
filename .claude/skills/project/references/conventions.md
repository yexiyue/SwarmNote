# 项目管理约定

## 目录结构

默认 milestone 文档目录为 `milestones/`（位于项目根目录），可通过参数覆盖。

```
milestones/
├── v0.1.0/
│   ├── README.md               ← 版本目标、范围、验收标准
│   ├── features/
│   │   ├── editor.md           ← 单个功能的需求 + 技术方案
│   │   └── p2p-sync.md
│   └── design/                 ← 设计稿（Excalidraw / 截图 / Figma 链接）
├── v0.2.0/
│   └── ...
```

## 版本号

遵循 [SemVer](https://semver.org/)：
- `MAJOR` — 不兼容的 API 变更
- `MINOR` — 向后兼容的功能新增
- `PATCH` — 向后兼容的 Bug 修复

## Issue 规范

标题格式：`<功能简称>: <描述>`，如 `editor: 集成 BlockNote 编辑器`

Issue body 结构：
```markdown
## 描述
做什么，为什么做。

## 验收标准
- [ ] 条件 1
- [ ] 条件 2

## 技术备注
实现要点（可选）。

## 原始文档
[查看 Feature 文档](链接到 milestones/ 中的 feature 文档)
```

## 依赖关系

### Issue 级别

- Issue 正文中用 `Depends on #N` 标注前置依赖
- PR 关联 Issue 用 `Closes #N`

### Labels 分层

用 `layer:` 前缀标签标注功能所在的依赖层级，便于过滤和排序：

| Label | 含义 | 说明 |
|-------|------|------|
| `layer:L0` | 无依赖，可立即开始 | 基础设施类任务 |
| `layer:L1` | 依赖 L0 | L0 完成后可开始 |
| `layer:L2` | 依赖 L0 + L1 | 需等待大部分功能完成 |

### GitHub Projects 自定义字段

在 GitHub Projects 中添加 `Layer` 单选字段（`L0` / `L1` / `L2`），用于：
- Board 视图按 Layer 分列展示
- Table 视图按 Layer 分组排序
- 直观看到哪些任务可以并行、哪些需要等待

## Sprint 规范

- 周期：1-2 周
- 命名：`Sprint 1`, `Sprint 2`, ...（使用 Milestone）
- Sprint Goal 写在 Milestone description 中

## Definition of Done

Issue 关闭前必须满足：
- 代码已实现且功能正常
- 所有验收标准已满足
- 测试通过（如有）
- 无 lint / 编译错误
- 已自我 Review（检查 diff）
- 文档已更新（如有用户可见变更）
- 关闭时引用对应的 commit 或 PR
