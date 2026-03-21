---
name: project-sprint-new
description: Create a new Sprint by selecting Issues from backlog and setting a Sprint Goal. Use when starting a new development iteration.
---

创建新的 Sprint，从 backlog 选择 Issue 纳入，通过 GitHub Project 的 Iteration 字段管理。

前置条件：`gh` CLI 已认证（含 `project` scope）。所有 `gh` 命令前加 `GH_PAGER=cat`。

**重要**：Sprint 不使用 Milestone。Milestone 只用于版本（v0.1.0、v0.2.0）。Sprint 通过 GitHub Project 的 Iteration 字段管理。

## 步骤

### 1. 获取 Project 和 Sprint 字段信息

```bash
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')
GH_PAGER=cat gh project field-list <PROJECT_NUM> --owner "@me" --format json
```

从返回的 fields 中找到 Sprint 字段（类型为 `ProjectV2IterationField`），记录：
- Sprint 字段 ID
- 当前可用的 Iteration 周期

### 2. 列出 Backlog

列出所有 open Issue（可按版本 Milestone 过滤）：
```bash
GH_PAGER=cat gh issue list --milestone "<version>" --state open \
  --json number,title,labels \
  --jq '.[] | "#\(.number) \(.title) [\(.labels | map(.name) | join(", "))]"'
```

按 Layer 分组展示，方便用户选择：
- L0（无依赖）优先推荐
- L1 需确认 L0 依赖是否已完成
- L2 需确认 L0+L1 依赖是否已完成

### 3. 用户选择

用 AskUserQuestion 让用户：
- 选择本次 Sprint 包含哪些 Issue
- 定义 Sprint Goal（一句话描述这个 Sprint 最重要的产出）
- 确认 Sprint 周期（默认 1 周）

### 4. 设置 GitHub Project Sprint Iteration 字段

通过 GraphQL API 将选中的 Issue 分配到当前 Iteration：

```bash
# 获取 Item ID
GH_PAGER=cat gh project item-list <PROJECT_NUM> --owner "@me" --format json

# 设置 Iteration 字段（需要通过 GraphQL）
GH_PAGER=cat gh api graphql -f query='
  mutation {
    updateProjectV2ItemFieldValue(input: {
      projectId: "<PROJECT_ID>"
      itemId: "<ITEM_ID>"
      fieldId: "<SPRINT_FIELD_ID>"
      value: { iterationId: "<ITERATION_ID>" }
    }) {
      projectV2Item { id }
    }
  }'
```

> **注意**：Iteration 字段的值需要通过 GraphQL 查询获取当前可用的 iteration ID，CLI 的 `item-edit` 不直接支持 Iteration 类型。

### 5. 输出确认

```
Sprint 已创建
Goal: <goal>
周期: YYYY-MM-DD ~ YYYY-MM-DD
GitHub Project: <Project URL>（按 Sprint 字段筛选查看）

包含 Issue:
  L0: #1 设计稿, #2 UI 框架, #3 状态管理
  L1: #6 Markdown 存储
  共 4 个 Issue

提示：
- 查看进度：`/project-sprint-status`
- 关闭 Sprint：`/project-sprint-close`
- Roadmap 视图可按时间轴查看 Sprint 排期
```
