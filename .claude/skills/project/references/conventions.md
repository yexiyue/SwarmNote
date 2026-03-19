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

- Issue 间依赖用 `Depends on #N` 评论
- PR 关联 Issue 用 `Closes #N`

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
