---
name: project-status
description: Show overall project status with all milestone progress, backlog count, and blocked issues. Use when the user wants a high-level view of the project.
---

输出整体项目状态概览，含 GitHub Project 链接和 Roadmap 视图提示。

前置条件：`gh` CLI 已认证。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

### 1. 列出所有 Milestone 进度

```bash
GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --jq \
  '.[] | select(.state=="open") | "\(.title): \(.closed_issues)/\(.open_issues + .closed_issues) (\(if (.open_issues + .closed_issues) > 0 then (.closed_issues * 100 / (.open_issues + .closed_issues) | floor) else 0 end)%)"'
```

### 2. 统计 Backlog

```bash
GH_PAGER=cat gh issue list --no-milestone --state open --json number -q '. | length'
```

### 3. 检查阻塞项

```bash
GH_PAGER=cat gh issue list --label "status:blocked" --state open \
  --json number,title,milestone \
  -q '.[] | "#\(.number) \(.title) [\(.milestone.title // "no milestone")]"'
```

### 4. 获取 GitHub Project 链接

```bash
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')
GH_PAGER=cat gh project list --owner "$OWNER" --format json
```

### 5. 输出概览

格式：
```
项目状态概览
============

Milestones:
  v0.1.0:    8/12 (66%)  ████████████░░░░ 66%
  Sprint 3:  3/5  (60%)  ████████████░░░░ 60%

Backlog: 15 个未分配 Issue
阻塞:   2 个 Issue
  ✖ #23 P2P 集成 [v0.1.0]
  ✖ #31 性能测试 [Sprint 3]

GitHub Project: <Project URL>

推荐视图：
- Board 视图：按 Status 分列（Todo / In Progress / Done）
- Table 视图：按 Layer 分组，快速看依赖层级
- Roadmap 视图：按 Sprint 时间线展示进度

提示：
- 查看当前 Sprint：`/project-sprint-status`
- 创建新 Sprint：`/project-sprint-new`
```
