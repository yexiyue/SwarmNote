---
name: project-init
description: Initialize GitHub project management infrastructure with standardized labels and milestones directory. Use when setting up a new project or resetting labels.
---

初始化项目的 GitHub 管理基础设施。

前置条件：`gh` CLI 已安装并认证。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 步骤

1. **验证认证**：
   ```bash
   GH_PAGER=cat gh auth status
   ```

2. **删除 GitHub 默认 labels**：
   ```bash
   GH_PAGER=cat gh label list --json name -q '.[].name' | xargs -I {} GH_PAGER=cat gh label delete {} --yes
   ```

3. **创建 Label 体系**，读取 `project/references/labels.md` 获取完整命令列表，依次执行所有 `gh label create`。Label 分类：
   - `type:` — feature / bug / chore / docs / spike
   - `priority:` — critical / high / medium / low
   - `size:` — xs / s / m / l / xl
   - `status:` — ready / in-progress / blocked / review
   - 特殊 — mvp / tech-debt / good-first-issue

4. **创建 milestones 目录**（如不存在）：`mkdir -p milestones/`

5. **输出确认**：展示创建的 labels 数量和 milestones 目录状态

## 共享资源

- Labels 完整定义：`project/references/labels.md`
- 项目约定：`project/references/conventions.md`

以上路径相对于 `.claude/skills/`。
