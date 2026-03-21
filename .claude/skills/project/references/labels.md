# Label 体系

创建时使用 `GH_PAGER=cat` 避免交互式分页阻塞。

## 类型

```bash
gh label create "type:feature" --color "1D76DB" --description "新功能"
gh label create "type:bug" --color "D73A4A" --description "Bug 修复"
gh label create "type:chore" --color "0E8A16" --description "维护、重构、工具链"
gh label create "type:docs" --color "0075CA" --description "文档"
gh label create "type:spike" --color "D4C5F9" --description "技术调研（限时）"
```

## 优先级

```bash
gh label create "priority:critical" --color "B60205" --description "必须立即修复"
gh label create "priority:high" --color "D93F0B" --description "下个 Sprint 必须完成"
gh label create "priority:medium" --color "FBCA04" --description "尽快完成"
gh label create "priority:low" --color "C2E0C6" --description "有空再做"
```

## 工作量

```bash
gh label create "size:xs" --color "EDEDED" --description "< 1 小时"
gh label create "size:s" --color "D4C5F9" --description "1-4 小时"
gh label create "size:m" --color "BFD4F2" --description "4-8 小时"
gh label create "size:l" --color "FBCA04" --description "1-2 天"
gh label create "size:xl" --color "D93F0B" --description "> 2 天（应拆分）"
```

## 状态

```bash
gh label create "status:ready" --color "0E8A16" --description "已细化，可进入 Sprint"
gh label create "status:in-progress" --color "1D76DB" --description "开发中"
gh label create "status:blocked" --color "B60205" --description "被阻塞"
gh label create "status:review" --color "D4C5F9" --description "等待 Review"
```

## 依赖层级

```bash
gh label create "layer:L0" --color "0E8A16" --description "无依赖，可立即开始"
gh label create "layer:L1" --color "1D76DB" --description "依赖 L0 完成后可开始"
gh label create "layer:L2" --color "D93F0B" --description "依赖 L0+L1 完成后可开始"
```

## 特殊

```bash
gh label create "mvp" --color "FEF2C0" --description "MVP 必需"
gh label create "tech-debt" --color "E4E669" --description "技术债务"
gh label create "good-first-issue" --color "7057FF" --description "适合新贡献者"
```
