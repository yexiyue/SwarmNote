---
name: project-sprint-close
description: Close the current Sprint, handle carryover issues, and generate a retrospective. Use when a sprint period ends and the user wants to wrap up and review.
---

关闭当前 Sprint，处理 carryover，生成回顾，同步 GitHub Project 状态。

前置条件：`gh` CLI 已认证（含 `project` scope）。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

### 1. 获取当前 Sprint

```bash
GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --jq \
  '[.[] | select(.state=="open" and (.title | startswith("Sprint")))] | sort_by(.number) | last'
```

### 2. 统计完成情况

列出 open 和 closed Issues，计算完成率。

### 3. 处理未完成 Issue

用 AskUserQuestion 让用户对每个未完成的 Issue 选择：
- 移回 backlog（清除 Sprint 字段）
- 移入下一个 Sprint

移回 backlog：
```bash
GH_PAGER=cat gh issue edit <number> --milestone "" \
  --remove-label "status:in-progress" --add-label "status:ready"
```

如果有 GitHub Project，同步清除 Sprint 字段：
```bash
GH_PAGER=cat gh project item-edit --project-id "<PROJECT_ID>" \
  --id "<ITEM_ID>" --field-id "<SPRINT_FIELD_ID>" --clear
```

### 4. 关闭 Sprint Milestone

```bash
MILESTONE_NUM=$(GH_PAGER=cat gh api repos/{owner}/{repo}/milestones \
  --jq '.[] | select(.title=="Sprint N") | .number')
GH_PAGER=cat gh api repos/{owner}/{repo}/milestones/${MILESTONE_NUM} \
  --method PATCH --field state="closed"
```

### 5. 生成回顾 Issue

```bash
GH_PAGER=cat gh issue create \
  --title "Retrospective: Sprint N" \
  --label "type:docs" \
  --body "$(cat <<'EOF'
## 完成了什么？
- <已关闭 Issue 列表>

## 什么做得好？
-

## 什么可以改进？
-

## 下个 Sprint 的行动项
- [ ]

## 统计
- **计划**: X 个 Issue
- **完成**: Y 个
- **Carryover**: Z 个
- **Sprint Goal 达成**: Yes/No
EOF
)"
```

### 6. 可选：创建 Release

检查版本 Milestone 是否所有 Issue 都已关闭，如果是，询问用户是否创建 Release：
```bash
GH_PAGER=cat gh release create v<version> --title "<version>" \
  --notes "<changelog>"
```

### 7. 输出汇总

```
Sprint N 已关闭
完成率：X/Y (Z%)
Carryover：N 个 Issue（已移回 backlog / 移入 Sprint N+1）
回顾 Issue：#<number>

提示：
- 创建下一个 Sprint：`/project-sprint-new`
- 查看整体状态：`/project-status`
```
