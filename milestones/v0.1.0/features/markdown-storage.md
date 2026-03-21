# Markdown 存储

## 用户故事

作为用户，我希望我的笔记以标准 Markdown 文件保存，以便用任何文本编辑器打开，数据不被锁定。

## 需求描述

SwarmNote 的核心存储格式是 Markdown 文件。BlockNote 编辑器的内容通过 BlockNote 内置的转换功能与 Markdown 双向转换。图片等资源存储在与 .md 文件同名的目录中。

## 交互设计

### 用户操作流程

- 用户在 BlockNote 中编辑 → 自动保存为 .md 文件
- 用户打开笔记 → 读取 .md 文件 → 转换为 BlockNote blocks 展示
- 资源文件（图片）通过编辑器插入时自动复制到资源目录

### 存储结构

```
工作区/
├── 日记/
│   ├── 2026-03-19.md              ← Markdown 笔记
│   └── 2026-03-19/                ← 同名资源目录
│       ├── screenshot.png
│       └── diagram.svg
└── 快速笔记.md
```

## 技术方案

### 前端

- `@blocknote/core` 提供的 `blocksToMarkdownLossy()` — blocks → Markdown
- `@blocknote/core` 提供的 `markdownToBlocks()` — Markdown → blocks
- 保存时将 blocks 转为 Markdown 传给 Rust 端
- 加载时 Rust 端返回 Markdown，前端转为 blocks

### 后端

- 文件读写使用 `std::fs` 或 `tokio::fs`
- 保存时：接收 Markdown 字符串 → 写入 .md 文件 → 计算 blake3 hash → 更新 workspace.db
- 加载时：读取 .md 文件 → 返回 Markdown 字符串
- 资源目录管理：
  - 插入图片时创建同名目录（如 `笔记.md` → `笔记/`）
  - 删除笔记时同时删除资源目录
  - 重命名笔记时同步重命名资源目录

### 注意事项

- BlockNote ↔ Markdown 转换是有损的：
  - 支持：标题、列表、代码块、引用、分割线、图片、链接、粗体、斜体
  - 不支持/丢失：文字颜色、背景色、对齐方式等高级格式
- 这是设计决策的取舍：选择 Markdown 的通用性，接受格式有损

## 验收标准

- [ ] 编辑内容保存为标准 `.md` 文件
- [ ] .md 文件可被其他 Markdown 编辑器正常打开和阅读
- [ ] 从 .md 文件加载到 BlockNote 后内容结构正确
- [ ] 图片插入后保存到同名资源目录，Markdown 中引用路径正确
- [ ] 删除笔记时同步删除资源目录
- [ ] 重命名笔记时资源目录同步重命名

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- Markdown 中的图片引用路径格式？
  - 建议使用相对路径：`![alt](./笔记名/image.png)`
- 是否需要支持从剪贴板粘贴图片？
  - 建议 v0.1.0 支持，是常见操作
