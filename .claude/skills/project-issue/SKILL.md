---
name: project-issue
description: 快速创建 GitHub Issue（bug、优化、技术债务等）。当开发过程中发现问题或改进点，需要记录并让协作者跟进时使用。
---

快速创建单个 GitHub Issue，适用于开发中随时发现的 bug、优化点、技术债务等。

前置条件：`gh` CLI 已认证（含 `project` scope）。所有 `gh` 命令前加 `GH_PAGER=cat`。

## 适用场景

- 开发中发现 bug 或异常行为
- 注意到可优化/重构的地方
- 需要记录技术债务
- 想为协作者创建一个可追踪的任务
- 小的 enhancement 不需要走完整的 milestone 规划流程

> 与 `/project-plan` 的区别：`/project-plan` 从 milestone 文档批量创建 Issue；`/project-issue` 是单个 Issue 的快速通道。

## 输入

用户直接描述问题或需求（自然语言），可选附带参数 `/project-issue <描述>`。

## 步骤

### 1. 收集信息

如果用户描述已经足够清晰（包含问题是什么、在哪里），直接进入步骤 2。

如果描述模糊，用 AskUserQuestion 收集：

```
问题 1：Issue 类型？
  - Bug（现有功能不正常）
  - Enhancement（改进现有功能）
  - Chore（重构、工具链、技术债务）
  - Docs（文档缺失或错误）

问题 2：优先级？
  - Critical（必须立即修复）
  - High（下个 Sprint 必须完成）
  - Medium（尽快完成）（默认）
  - Low（有空再做）
```

从用户描述中自动推断：
- **标题**：简洁动词开头（fix: / improve: / refactor: / docs:）
- **关联 Milestone**：根据当前版本自动关联（如 v0.1.0）
- **关联功能**：如果涉及某个已有 Parent Issue，标注关联

### 2. 确定 Labels

根据类型和优先级映射到已有 label 体系：

| 类型 | Label |
|------|-------|
| Bug | `type:bug` |
| Enhancement | `type:feature` |
| Chore | `type:chore` |
| Docs | `type:docs` |

| 优先级 | Label |
|--------|-------|
| Critical | `priority:critical` |
| High | `priority:high` |
| Medium | `priority:medium` |
| Low | `priority:low` |

可选附加 label：
- `tech-debt` — 技术债务类
- `good-first-issue` — 适合新贡献者
- `mvp` — MVP 必需

### 3. 构造 Issue Body

按项目约定格式：

```markdown
## 描述
<问题描述：做什么，为什么做>

## 复现步骤（Bug 类型）
1. ...
2. ...
3. 期望行为 vs 实际行为

## 验收标准
- [ ] <条件 1>
- [ ] <条件 2>

## 技术备注
<涉及的文件、模块、可能的修复方向（如果已知）>
```

根据类型调整结构：
- **Bug**：包含「复现步骤」和「期望 vs 实际」
- **Enhancement / Chore**：包含「当前行为」和「期望改进」
- **Docs**：包含「缺失/错误内容」和「建议修改」

### 4. 创建 Issue

```bash
GH_PAGER=cat gh issue create \
  --title "<title>" \
  --label "<label1>,<label2>" \
  --milestone "<version>" \
  --body "<body>"
```

### 5. 关联到 GitHub Project

```bash
OWNER=$(GH_PAGER=cat gh repo view --json owner -q '.owner.login')
PROJECT_NUM=$(GH_PAGER=cat gh project list --owner "$OWNER" --format json | jq '.projects[0].number')
GH_PAGER=cat gh project item-add $PROJECT_NUM --owner "$OWNER" --url <ISSUE_URL>
```

### 6. 可选：关联为 Sub-issue

如果 Issue 属于某个已有 Parent Issue（功能级别），询问是否关联：

```bash
PARENT_ID=$(GH_PAGER=cat gh issue view <parent_number> --json id -q '.id')
CHILD_ID=$(GH_PAGER=cat gh issue view <child_number> --json id -q '.id')
GH_PAGER=cat gh api graphql -f query='
  mutation {
    addSubIssue(input: {issueId: "'"$PARENT_ID"'", subIssueId: "'"$CHILD_ID"'"}) {
      issue { id }
    }
  }'
```

### 7. 输出确认

```
Issue 已创建 ✓

  #<number> <title>
  类型: <type>  优先级: <priority>  版本: <milestone>
  链接: <URL>

  Labels: type:bug, priority:high
  Project: 已添加到 <Project Name>

提示：
- 查看所有 Issue：`/project-status`
- 纳入 Sprint：`/project-sprint-new`
- 批量创建（从文档）：`/project-plan`
```

## 批量模式

如果用户一次描述了多个问题，逐一创建并汇总输出：

```
已创建 3 个 Issue ✓

  #21 fix: Windows 标题栏拖拽失效      type:bug    priority:high
  #22 fix: 窗口控制按钮无反应           type:bug    priority:high
  #23 improve: 启动加载状态优化          type:feature priority:medium

全部已添加到 Project，关联 Milestone v0.1.0。
```
