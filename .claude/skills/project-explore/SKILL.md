---
name: project-explore
description: Guided requirement exploration for a version milestone. Use when the user wants to discuss and plan what to build for a version, then generate milestone documents (README + feature docs).
---

引导式对话，探索版本需求，生成 milestone 文档。

**这是一个探索过程，不是实现过程。** 只讨论和生成文档，不写代码。

## 输入

用户提供版本号（如 `v0.2.0`），可选附带简短描述。

## 步骤

### 1. 建立上下文

读取项目已有文档，理解背景：
- `CLAUDE.md` — 项目架构和约定
- `milestones/` — 已有版本文档（了解已完成/规划的功能）
- 如有 `dev-notes/product-vision.md` 等规划文档也一并读取

### 2. 引导讨论

逐步提问，每次 1-2 个问题，用 AskUserQuestion：

**第一轮：目标与范围**
- "这个版本要解决什么核心问题？用户完成后能做什么？"
- "有哪些功能是这个版本必须包含的？哪些明确不做？"

**第二轮：功能细化**
- 对每个功能逐个讨论优先级（P0/P1/P2）
- P0 功能深入探讨：
  - "用户操作流程是什么？"
  - "技术上倾向什么方案？有什么约束？"
  - "验收标准是什么？怎么判断做完了？"

**第三轮：风险与依赖**
- "有什么技术风险或不确定性？"
- "依赖哪些前置工作或外部条件？"
- "时间预期？"

**原则**：
- 不要一次问太多，保持对话节奏
- 用户回答模糊时追问具体细节
- 主动提出建议和方案供用户选择
- 随时根据新信息调整讨论方向

### 3. 生成文档

讨论完成后，基于模板生成文件：

- 版本文档模板：`project/templates/version-readme.md`
- 功能文档模板：`project/templates/feature.md`

生成目标：
- `milestones/<version>/README.md` — 版本目标、范围、功能清单、验收标准
- `milestones/<version>/features/<name>.md` — 每个功能一个文档
- `milestones/<version>/design/` — 创建空目录，后续放设计稿

### 4. 确认

展示生成的文件清单和关键内容摘要，让用户 review。如用户要求修改，直接编辑对应文件。

## 共享资源

- 版本文档模板：`project/templates/version-readme.md`
- 功能文档模板：`project/templates/feature.md`
- 项目约定：`project/references/conventions.md`

以上路径相对于 `.claude/skills/`。
