---
name: project-init
description: Initialize GitHub project management infrastructure with standardized labels, GitHub Project board, and milestones directory. Use when setting up a new project or resetting labels.
---

初始化项目的 GitHub 管理基础设施：Labels + GitHub Project + 目录结构。

前置条件：`gh` CLI 已安装并认证（需包含 `project` scope）。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

### 1. 验证认证

```bash
GH_PAGER=cat gh auth status
```

检查是否包含 `project` scope。如果没有，提示用户执行：
```bash
gh auth refresh -h github.com -s read:project,project
```

### 2. 删除 GitHub 默认 labels

```bash
GH_PAGER=cat gh label list --json name -q '.[].name' | xargs -I {} GH_PAGER=cat gh label delete {} --yes
```

### 3. 创建 Label 体系

读取 `project/references/labels.md` 获取完整命令列表，依次执行所有 `gh label create`。Label 分类：
- `type:` — feature / bug / chore / docs / spike
- `priority:` — critical / high / medium / low
- `size:` — xs / s / m / l / xl
- `status:` — ready / in-progress / blocked / review
- `layer:` — L0（无依赖） / L1（依赖 L0） / L2（依赖 L0+L1）
- 特殊 — mvp / tech-debt / good-first-issue

### 4. 创建 GitHub Project

```bash
# 获取仓库名作为 Project 标题
REPO_NAME=$(GH_PAGER=cat gh repo view --json name -q '.name')
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')

# 创建 Project
GH_PAGER=cat gh project create --owner "$OWNER" --title "$REPO_NAME" --format json
```

记录返回的 Project number 和 ID，后续步骤需要。

### 5. 添加 Project 自定义字段

```bash
PROJECT_NUM=<上一步返回的 number>

# Layer 字段（依赖层级）
GH_PAGER=cat gh project field-create "$PROJECT_NUM" --owner "$OWNER" \
  --name "Layer" --data-type "SINGLE_SELECT" \
  --single-select-options "L0,L1,L2"

# Sprint 字段（开发迭代）
GH_PAGER=cat gh project field-create "$PROJECT_NUM" --owner "$OWNER" \
  --name "Sprint" --data-type "SINGLE_SELECT" \
  --single-select-options "Sprint 1,Sprint 2,Sprint 3,Sprint 4,Sprint 5"
```

> **注意**：GitHub CLI 不支持创建 Iteration 类型字段，这里用 SINGLE_SELECT 代替。如需日期范围功能，用户可在 Web UI 中手动改为 Iteration 类型。

### 6. 提示配置 Workflows（手动）

GitHub Projects Workflows 只能通过 Web UI 配置，输出以下提示：

```
请到 GitHub Project 页面手动开启自动化 Workflows：
1. 打开 Project → 右上角 ... → Workflows
2. 开启以下规则：
   - Item added to project → Status = Todo
   - Item closed → Status = Done
   - Item reopened → Status = In Progress
   - Pull request merged → Status = Done
```

### 7. 创建目录结构

```bash
mkdir -p milestones/
```

### 8. 输出确认

展示：
- 创建的 labels 数量
- GitHub Project 链接
- 自定义字段列表
- milestones 目录状态
- Workflows 手动配置提醒

## 共享资源

- Labels 完整定义：`project/references/labels.md`
- 项目约定：`project/references/conventions.md`

以上路径相对于 `.claude/skills/`。
