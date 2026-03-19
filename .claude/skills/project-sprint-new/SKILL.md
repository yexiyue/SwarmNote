---
name: project-sprint-new
description: Create a new Sprint by selecting Issues from backlog and setting a Sprint Goal. Use when starting a new development iteration.
---

创建新的 Sprint Milestone，从 backlog 选择 Issue 纳入。

前置条件：`gh` CLI 已认证。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

### 1. 获取 Sprint 编号

```bash
SPRINT_NUM=$(GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --jq '[.[] | select(.title | startswith("Sprint"))] | length + 1')
```

### 2. 列出 Backlog

列出有 `status:ready` 标签的 open Issue：
```bash
GH_PAGER=cat gh issue list --label "status:ready" --json number,title,labels \
  --jq '.[] | "#\(.number) \(.title) [\(.labels | map(.name) | join(", "))]"'
```

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

### 5. 分配 Issue

将选中的 Issue 加入 Sprint Milestone：
```bash
GH_PAGER=cat gh issue edit <number> --milestone "Sprint ${SPRINT_NUM}"
```

### 6. 输出确认

展示 Sprint 概要：编号、Goal、截止日期、包含的 Issue 列表。
