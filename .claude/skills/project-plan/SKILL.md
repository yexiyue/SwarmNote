---
name: project-plan
description: Parse milestone documents and create GitHub Milestone with Issues. Use when the user has finished writing version docs and wants to break them down into actionable GitHub Issues.
---

读取版本文档，创建 GitHub Milestone 和 Issues。

前置条件：`gh` CLI 已认证。`milestones/<version>/` 下有 README.md 和 features/*.md。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 输入

用户提供版本号（如 `v0.1.0`）。

## 步骤

### 1. 读取文档

读取 `milestones/<version>/README.md` 和 `milestones/<version>/features/*.md`，理解版本目标和所有功能。

### 2. 创建 Milestone

```bash
GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --method POST \
  --field title="<version>" \
  --field description="<版本目标，从 README.md 提取>" \
  --field due_on="<截止日期，如有>"
```

### 3. 拆分并创建 Issues

对每个 feature 文档：
- 将功能拆分为可独立开发的任务（每个 2-8 小时）
- 确定 labels（type + priority + size），参考 `project/references/labels.md`
- 查重避免重复：
  ```bash
  GH_PAGER=cat gh issue list --search "in:title <标题关键词>" --json number -q '.[].number'
  ```
- 创建 Issue：
  ```bash
  GH_PAGER=cat gh issue create \
    --title "<功能简称>: <任务描述>" \
    --label "<labels>" \
    --milestone "<version>" \
    --body "$(cat <<'EOF'
  ## 描述
  <描述>

  ## 验收标准
  - [ ] 条件 1
  - [ ] 条件 2

  ## 技术备注
  <技术要点>

  ## 原始文档
  [查看 Feature 文档](<相对路径>)
  EOF
  )"
  ```

Issue 标题格式：`<功能简称>: <任务描述>`，如 `editor: 集成 BlockNote 编辑器`。

### 4. 关联依赖

创建完所有 Issue 后，对有依赖关系的 Issue 添加评论：
```bash
GH_PAGER=cat gh issue comment <issue-number> --body "Depends on #<blocker-number>"
```

### 5. 更新文档

将创建的 Issue 编号回填到 `milestones/<version>/README.md` 的功能清单表格中。

### 6. 输出汇总

```
| 功能 | 任务 | Issue | Labels |
|------|------|-------|--------|
| 编辑器 | 集成 BlockNote | #12 | type:feature, priority:high, size:m |
```

## 共享资源

- Label 定义：`project/references/labels.md`
- Issue 规范：`project/references/conventions.md`

以上路径相对于 `.claude/skills/`。
