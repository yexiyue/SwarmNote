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

## GitHub Project

每个项目创建一个 GitHub Project（Board 类型），统一管理所有版本的 Issues。

### 自定义字段

| 字段 | 类型 | 值 | 用途 |
|------|------|-----|------|
| **Status** | 内置 | Todo / In Progress / Done | Issue 状态流转 |
| **Layer** | 单选 | L0 / L1 / L2 | 依赖层级，决定开发顺序 |
| **Sprint** | Iteration | 自动按周期生成 | 当前所属 Sprint，支持 Roadmap 时间轴 |

> Sprint 字段使用 Iteration 类型（自带开始/结束日期），支持 Roadmap 视图按时间轴展示。CLI 设置 Iteration 字段值需通过 GraphQL API。

### 推荐视图

| 视图 | 分组/排序 | 用途 |
|------|-----------|------|
| **Board** | 按 Status 分列 | 日常看板，拖拽管理状态 |
| **Table** | 按 Layer 分组 | 看依赖层级，决定优先开发什么 |
| **Roadmap** | 按 Sprint 时间线 | 整体进度和时间规划 |

### Workflows（自动化）

以下 Workflows 需在 Web UI 中手动开启（CLI 不支持配置）：

- **Item added to project** → Status = `Todo`
- **Item closed** → Status = `Done`
- **Item reopened** → Status = `In Progress`
- **Pull request merged** → Status = `Done`

### CLI 权限

`gh` CLI 需要 `project` scope 才能操作 Projects：
```bash
gh auth refresh -h github.com -s read:project,project
```

## Issue 规范

### 标题格式

`<功能简称>: <描述>`，如 `editor: 集成 BlockNote 编辑器`

### Issue 层级

采用 Parent / Sub-issue 结构：

- **Parent Issue**：对应一个 feature，描述功能总体目标和验收标准
- **Sub-issues**：对应 feature 下的独立任务（每个 2-8 小时），通过 GraphQL API 关联

```
Parent: editor: 集成 BlockNote 编辑器 (#7)
  ├── Sub: editor: 基础 BlockNote 集成 (#13)
  ├── Sub: editor: 暗色主题适配 (#14)
  └── Sub: editor: 自动保存 + debounce (#15)
```

如果 feature 本身不可再拆分（如密钥管理），直接作为普通 Issue，不需要 Sub-issues。

### Issue body 结构

```markdown
## 描述
做什么，为什么做。

## 前置依赖
- [ ] #N
- [ ] #M
（每行一个前置 Issue，GitHub 会显示 Tasklist 进度条）

## 验收标准
- [ ] 条件 1
- [ ] 条件 2

## 技术备注
实现要点（可选）。

## 原始文档
[查看 Feature 文档](链接到 milestones/ 中的 feature 文档)
```

### 关联 Sub-issue（GraphQL）

```bash
PARENT_ID=$(GH_PAGER=cat gh issue view <parent_number> --json id -q '.id')
CHILD_ID=$(GH_PAGER=cat gh issue view <child_number> --json id -q '.id')
GH_PAGER=cat gh api graphql -f query='
  mutation {
    addSubIssue(input: {issueId: "'"$PARENT_ID"'", subIssueId: "'"$CHILD_ID"'"}) {
      issue { id }
    }
  }'
```

## 依赖关系

### Issue 级别

- Issue 正文中用 Tasklist 格式标注前置依赖：`- [ ] #N`（每行一个，GitHub 会显示进度条）
- 前置 Issue 关闭后，手动勾选对应 checkbox 更新进度
- PR 关联 Issue 用 `Closes #N`

### Labels 分层

用 `layer:` 前缀标签标注功能所在的依赖层级，便于过滤和排序：

| Label | 含义 | 说明 |
|-------|------|------|
| `layer:L0` | 无依赖，可立即开始 | 基础设施类任务 |
| `layer:L1` | 依赖 L0 | L0 完成后可开始 |
| `layer:L2` | 依赖 L0 + L1 | 需等待大部分功能完成 |

### GitHub Projects Layer 字段

与 Labels 对应，在 Project 中通过 Layer 字段实现：
- Board 视图按 Layer 分列展示
- Table 视图按 Layer 分组排序
- 直观看到哪些任务可以并行、哪些需要等待

## Sprint 规范

- 周期：1-2 周
- **不使用 Milestone 管理 Sprint**，Milestone 只用于版本（v0.1.0、v0.2.0）
- Sprint 通过 GitHub Project 的 **Iteration 字段**管理（自带开始/结束日期）
- Sprint Goal 记录在 Sprint 创建时的输出中
- Roadmap 视图可按 Iteration 时间轴展示 Sprint 排期
- CLI 设置 Iteration 字段值需通过 GraphQL API（`updateProjectV2ItemFieldValue`）

## Definition of Done

Issue 关闭前必须满足：
- 代码已实现且功能正常
- 所有验收标准已满足
- 测试通过（如有）
- 无 lint / 编译错误
- 已自我 Review（检查 diff）
- 文档已更新（如有用户可见变更）
- 关闭时引用对应的 commit 或 PR
