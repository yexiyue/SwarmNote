---
name: project-sprint-new
description: Create a new Sprint by selecting Issues from backlog and setting a Sprint Goal. Use when starting a new development iteration.
---

创建新的 Sprint，从 backlog 选择 Issue 纳入，同步更新 GitHub Project 的 Sprint 字段。

前置条件：`gh` CLI 已认证（含 `project` scope）。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

### 1. 获取 Sprint 编号

```bash
SPRINT_NUM=$(GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --jq '[.[] | select(.title | startswith("Sprint"))] | length + 1')
```

### 2. 列出 Backlog

列出版本 Milestone 中尚未分配 Sprint 的 open Issue：
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
- 确认 Sprint 周期（默认 2 周）

### 4. 创建 Sprint Milestone

```bash
DUE_DATE=$(python3 -c "from datetime import datetime, timedelta; print((datetime.utcnow()+timedelta(days=14)).strftime('%Y-%m-%dT00:00:00Z'))")

GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --method POST \
  --field title="Sprint ${SPRINT_NUM}" \
  --field description="Sprint Goal: <goal>" \
  --field due_on="$DUE_DATE"
```

### 5. 分配 Issue 到 Sprint Milestone

将选中的 Issue 加入 Sprint Milestone，并更新状态标签：
```bash
GH_PAGER=cat gh issue edit <number> --milestone "Sprint ${SPRINT_NUM}"
GH_PAGER=cat gh issue edit <number> --remove-label "status:ready" --add-label "status:in-progress"
```

### 6. 更新 GitHub Project Sprint 字段

如果有 GitHub Project，更新每个 Issue 的 Sprint 字段：

```bash
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')
# 获取 Project 信息
GH_PAGER=cat gh project field-list <PROJECT_NUM> --owner "$OWNER" --format json
# 找到 Sprint 字段 ID 和 "Sprint N" 选项 ID

# 如果 "Sprint N" 选项不存在，先添加：
GH_PAGER=cat gh project field-delete <SPRINT_FIELD_ID> ... # 需要通过 GraphQL 添加选项

# 设置每个 Issue 的 Sprint 字段
GH_PAGER=cat gh project item-edit --project-id "<PROJECT_ID>" \
  --id "<ITEM_ID>" --field-id "<SPRINT_FIELD_ID>" \
  --single-select-option-id "<SPRINT_OPTION_ID>"
```

> **注意**：如果 Sprint N 选项不在预创建的选项中，需要通过 GraphQL API 动态添加，或者提前在 `project-init` 中创建足够多的选项。

### 7. 输出确认

```
Sprint ${SPRINT_NUM} 已创建
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
```
