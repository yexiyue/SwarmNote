---
name: project-sprint-status
description: Show current Sprint progress with completion stats, in-progress issues, and blockers. Use when checking how the current sprint is going.
---

查看当前活跃 Sprint 的进度报告，含 GitHub Project 链接。

前置条件：`gh` CLI 已认证。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

### 1. 找到当前 Sprint

```bash
GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --jq \
  '[.[] | select(.state=="open" and (.title | startswith("Sprint")))] | sort_by(.number) | last'
```

### 2. 列出 Issue 状态

```bash
MILESTONE="Sprint N"
echo "=== 进行中 ===" && GH_PAGER=cat gh issue list --milestone "$MILESTONE" --state open \
  --json number,title,labels,assignees \
  -q '.[] | "#\(.number) \(.title) @\(.assignees | map(.login) | join(","))"'
echo "=== 已完成 ===" && GH_PAGER=cat gh issue list --milestone "$MILESTONE" --state closed \
  --json number,title \
  -q '.[] | "#\(.number) \(.title)"'
```

### 3. 检查阻塞项

```bash
GH_PAGER=cat gh issue list --label "status:blocked" --state open \
  --json number,title -q '.[] | "#\(.number) \(.title)"'
```

### 4. 获取 GitHub Project 链接

```bash
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')
GH_PAGER=cat gh project list --owner "$OWNER" --format json --jq '.projects[0].url'
```

### 5. 输出进度报告

格式：
```
Sprint N — <Sprint Goal>
截止日期：YYYY-MM-DD
GitHub Project: <Project URL>

进度：X/Y (Z%)
████████░░ Z%

已完成：
  ✓ #1 任务A
  ✓ #2 任务B

进行中：
  ◉ #3 任务C @assignee
  ◉ #4 任务D

阻塞：
  ✖ #5 任务E — 原因

提示：
- 关闭 Sprint：`/project-sprint-close`
- 查看整体状态：`/project-status`
```
