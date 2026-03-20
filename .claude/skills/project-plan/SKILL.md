---
name: project-plan
description: Parse milestone documents and create GitHub Milestone with Issues. Use when the user has finished writing version docs and wants to break them down into actionable GitHub Issues.
---

读取版本文档，创建 GitHub Milestone、Parent/Sub Issues，并关联到 GitHub Project。

前置条件：`gh` CLI 已认证（含 `project` scope）。`milestones/<version>/` 下有 README.md 和 features/*.md。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 输入

用户提供版本号（如 `v0.1.0`）。

## 步骤

### 1. 读取文档

读取 `milestones/<version>/README.md` 和 `milestones/<version>/features/*.md`，理解版本目标、所有功能和依赖关系。

特别关注 README.md 中的：
- **依赖关系**部分 — 功能间的层级（L0/L1/L2）
- **技术选型**部分 — 确保 Issue 中引用正确的技术方案

### 2. 创建 Milestone

```bash
GH_PAGER=cat gh api repos/{owner}/{repo}/milestones --method POST \
  --field title="<version>" \
  --field description="<版本目标，从 README.md 提取>" \
  --field due_on="<截止日期，如有>"
```

### 3. 检测 GitHub Project

```bash
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')
GH_PAGER=cat gh project list --owner "$OWNER" --format json
```

如果存在 Project，记录 Project number 和 ID，获取字段信息：
```bash
GH_PAGER=cat gh project field-list <PROJECT_NUM> --owner "$OWNER" --format json
```

记录 Layer 字段 ID 和各选项 ID（L0/L1/L2）。

### 4. 创建 Parent Issues（Feature 级别）

每个 feature 文档创建一个 Parent Issue。查重避免重复：
```bash
GH_PAGER=cat gh issue list --search "in:title <标题关键词>" --json number -q '.[].number'
```

创建 Issue：
```bash
GH_PAGER=cat gh issue create \
  --title "<功能简称>: <功能总述>" \
  --label "type:feature,priority:<P>,size:<S>,layer:<L>" \
  --milestone "<version>" \
  --body "$(cat <<'EOF'
## 描述
<功能总体描述>

## 依赖
Depends on #<blocker-number>（如有）

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

**创建顺序**：先 L0，再 L1，最后 L2。这样后续层的 Issue 可引用前置 Issue 编号。

### 5. 创建 Sub-issues（任务级别）

如果一个 feature 可拆分为多个独立任务（每个 2-8 小时），为 Parent Issue 创建 Sub-issues：

1. 创建子 Issue（同样的 `gh issue create` 命令）
2. 获取 Parent 和 Child 的 node ID：
```bash
PARENT_ID=$(GH_PAGER=cat gh issue view <parent_number> --json id -q '.id')
CHILD_ID=$(GH_PAGER=cat gh issue view <child_number> --json id -q '.id')
```
3. 通过 GraphQL API 关联为 Sub-issue：
```bash
GH_PAGER=cat gh api graphql -f query='
  mutation {
    addSubIssue(input: {issueId: "'"$PARENT_ID"'", subIssueId: "'"$CHILD_ID"'"}) {
      issue { id }
    }
  }'
```

> 如果一个 feature 本身就是不可再拆分的任务，不需要创建 Sub-issues。

### 6. 关联到 GitHub Project

如果检测到 Project，将所有 Issue（Parent + Sub）加入并设置 Layer 字段：

```bash
# 加入 Project
GH_PAGER=cat gh project item-add <PROJECT_NUM> --owner "$OWNER" \
  --url "https://github.com/{owner}/{repo}/issues/<number>"

# 获取 item ID
GH_PAGER=cat gh project item-list <PROJECT_NUM> --owner "$OWNER" --format json

# 设置 Layer 字段
GH_PAGER=cat gh project item-edit --project-id "<PROJECT_ID>" \
  --id "<ITEM_ID>" --field-id "<LAYER_FIELD_ID>" \
  --single-select-option-id "<OPTION_ID>"
```

### 7. 更新文档

将创建的 Issue 编号回填到 `milestones/<version>/README.md` 的功能清单表格中。

### 8. 输出汇总

```
GitHub Project: <Project URL>

| 功能 | Parent Issue | Sub-issues | Labels | Layer |
|------|-------------|------------|--------|-------|
| 编辑器 | #12 | #13, #14, #15 | type:feature, priority:high, size:l | L1 |
| SQLite | #5 | （无拆分） | type:feature, priority:high, size:l | L0 |

提示：
- 可用 `/project-sprint-new` 从 backlog 选择 Issue 创建 Sprint
- Project Board 视图：<Project URL>
- 建议按 Layer 分组查看，先完成 L0 再推进 L1
```

## 共享资源

- Label 定义：`project/references/labels.md`
- Issue 规范：`project/references/conventions.md`

以上路径相对于 `.claude/skills/`。
