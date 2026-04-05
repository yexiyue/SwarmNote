# yrs-blocknote: Rust 端 Markdown ↔ Y.Doc (yrs) 双向转换

## 用户故事

作为开发者，我需要 Rust 后端能独立完成 Markdown 与 Y.Doc 之间的双向转换，使 P2P 同步和 .md 文件写回不依赖前端。

## 依赖

- 无依赖（L0，可独立开始）
- 被 #27（编辑器 yjs 集成）和 #28（yjs CRDT 同步）依赖

## Crate 信息

| 项 | 值 |
|---|---|
| 包名 | `yrs-blocknote`（yrs 生态命名风格，类似 yrs-warp, yrs-kvstore） |
| 位置 | `crates/yrs-blocknote/`（仓库根目录，独立于 src-tauri） |
| 版本 | `0.1.0` |
| MSRV | 1.75 |
| 定位 | 通用库，不依赖 Tauri / SwarmNote，未来可发布到 crates.io |

## 需求描述

v0.2.0 以 Y.Doc 为唯一真相源，.md 文件作为 Y.Doc 的投影。Rust 端需要能独立完成以下操作：

1. **Y.Doc → Markdown**：远端 update 到达时，即使前端未打开文档，也能将 Y.Doc 导出为 .md 写回磁盘
2. **Markdown → Y.Doc**：首次纳入同步或外部编辑器修改 .md 时，能将 Markdown 解析并注入为 Y.Doc 内容

如果缺少此能力：
- 远端 update 到达但前端未打开文档 → 无法写回 .md
- 全量同步大量文档 → 必须逐个打开编辑器才能导出
- 外部编辑器修改 .md → 无法作为 Y.Doc update 同步到其他设备
- App 在托盘后台运行 → .md 永远不更新

## 技术方案

### BlockNote Y.Doc XML Schema 参考

BlockNote 基于 Tiptap/ProseMirror 构建，yjs 集成通过 `y-prosemirror`（v1.3.7）实现。转换规则：

- `ProseMirror Node` → `Y.XmlElement(node.type.name)`，attrs 全部写入（含默认值）
- `ProseMirror Text` → `Y.XmlText`，marks 作为 delta attributes

#### 文档根结构

```
XmlFragment("document-store")           ← 根，名称在 useCreateBlockNote 中指定
│
└── XmlElement("blockGroup")            ← 唯一子节点，包裹所有 block
     │
     ├── XmlElement("blockContainer")   ← 每个 block 一个
     │   attrs: { id, ...blockProps }   ← ⚠️ 会复制 blockContent 的全部 props
     │   │
     │   ├── XmlElement("<blockType>")  ← 如 "paragraph", "heading", ...
     │   │   attrs: { ...blockProps }   ← 和 blockContainer 上的 props 重复
     │   │   children:
     │   │     └── Y.XmlText            ← inline 内容（delta 格式）
     │   │
     │   └── XmlElement("blockGroup")   ← [可选] 子块/嵌套（缩进列表等）
     │        └── XmlElement("blockContainer") → ...递归
     │
     └── ...more blockContainer...
```

#### Block 类型映射

| BlockNote type | XmlElement name | 内容类型 | 特有 props（XML attrs） |
|---|---|---|---|
| `paragraph` | `"paragraph"` | inline (XmlText) | `textAlignment`, `textColor`, `backgroundColor` |
| `heading` | `"heading"` | inline | 同上 + `level`（number）, `isToggleable`（bool） |
| `bulletListItem` | `"bulletListItem"` | inline | 同 paragraph |
| `numberedListItem` | `"numberedListItem"` | inline | 同上 + `start`（number, optional） |
| `checkListItem` | `"checkListItem"` | inline | 同上 + `checked`（bool） |
| `codeBlock` | `"codeBlock"` | inline | `language` |
| `image` | `"image"` | **空**（无子节点） | `url`, `caption`, `name`, `previewWidth`, `showPreview`, `textAlignment`, `backgroundColor` |
| `table` | `"table"` | table 子结构 | `textColor` |
| `divider` | `"divider"` | **空** | 无 |

> 当前项目 schema 已去掉 `audio` 和 `file` block（见 NoteEditor.tsx）。

#### Inline 样式（Y.XmlText delta attributes）

| 样式 | attribute name | value |
|---|---|---|
| 粗体 | `"bold"` | `{}` |
| 斜体 | `"italic"` | `{}` |
| 下划线 | `"underline"` | `{}` |
| 删除线 | `"strike"` | `{}` |
| 行内代码 | `"code"` | `{}` |
| 链接 | `"link"` | `{ href: "..." }` |
| 文字颜色 | `"textColor"` | `{ stringValue: "<color>" }` |
| 背景色 | `"backgroundColor"` | `{ stringValue: "<color>" }` |

#### Table 子结构

```
XmlElement("table")
  attrs: { textColor }
  └── XmlElement("tableRow")
       ├── XmlElement("tableHeader")        ← 或 "tableCell"
       │   attrs: { colspan, rowspan, colwidth, backgroundColor, textColor, textAlignment }
       │   └── XmlElement("tableParagraph")
       │        └── Y.XmlText(cell content)
       └── XmlElement("tableCell")
            └── XmlElement("tableParagraph")
                 └── Y.XmlText(cell content)
```

#### 特殊节点

- **`hardBreak`**：文本内换行，作为独立 `XmlElement("hardBreak")` 穿插在 XmlText 之间
- **默认值也会写入**：`backgroundColor: "default"` 等默认值不省略，全部存为 XML attribute
- **`blockContainer` 和 `blockContent` props 重复**：BlockNote 的 `blockToNode` 会把 props 同时写到两层

#### 具体示例

BlockNote blocks:
```json
[{
  "id": "abc123",
  "type": "heading",
  "props": { "level": 2, "backgroundColor": "default", "textColor": "default", "textAlignment": "left" },
  "content": [
    { "type": "text", "text": "Hello ", "styles": { "bold": true } },
    { "type": "text", "text": "World", "styles": { "italic": true } }
  ],
  "children": [{
    "id": "def456",
    "type": "paragraph",
    "content": [{ "type": "text", "text": "Nested paragraph", "styles": {} }],
    "children": []
  }]
}]
```

对应 Y.Doc XML:
```
XmlFragment("document-store")
  └── XmlElement("blockGroup")
       └── XmlElement("blockContainer")
            attrs: { id: "abc123", backgroundColor: "default", textColor: "default",
                     textAlignment: "left", level: 2, isToggleable: false }
            ├── XmlElement("heading")
            │    attrs: { backgroundColor: "default", textColor: "default",
            │             textAlignment: "left", level: 2, isToggleable: false }
            │    └── Y.XmlText
            │         delta: [
            │           { insert: "Hello ", attributes: { bold: {} } },
            │           { insert: "World", attributes: { italic: {} } }
            │         ]
            └── XmlElement("blockGroup")
                 └── XmlElement("blockContainer")
                      attrs: { id: "def456", backgroundColor: "default",
                               textColor: "default", textAlignment: "left" }
                      └── XmlElement("paragraph")
                           attrs: { backgroundColor: "default", textColor: "default",
                                    textAlignment: "left" }
                           └── Y.XmlText
                                delta: [ { insert: "Nested paragraph" } ]
```

#### 源码参考

- BlockNote yjs 工具：`packages/core/src/yjs/utils.ts`
- Block → PM Node：`packages/core/src/api/nodeConversions/blockToNode.ts`
- PM Node → Block：`packages/core/src/api/nodeConversions/nodeToBlock.ts`
- PM 节点定义：`packages/core/src/pm-nodes/{Doc,BlockGroup,BlockContainer}.ts`
- Block schema 创建：`packages/core/src/schema/blocks/createSpec.ts`（line 153: `name: blockConfig.type`）
- y-prosemirror PM↔Yjs 映射：`y-prosemirror@1.3.7 src/plugins/sync-plugin.js`

---

### `yrs_to_markdown` — Y.Doc → Markdown 导出

遍历 yrs `XmlFragment` → `blockGroup` → `blockContainer` 树，将每个 block 映射为 Markdown：

| XmlElement 标签 | Markdown 输出 |
|---|---|
| `paragraph` | 纯文本 + `\n\n` |
| `heading [level=N]` | `#` 重复 N 次 + 空格 + 文本 |
| `bulletListItem` | `- ` + 文本（嵌套时增加缩进） |
| `numberedListItem` | `1. ` + 文本 |
| `checkListItem [checked]` | `- [ ]` / `- [x]` + 文本 |
| `codeBlock [language]` | `` ```lang `` + 文本 + `` ``` `` |
| `image [url, caption]` | `![caption](url)` |
| `table` | GFM 表格（遍历 tableRow → tableHeader/tableCell） |
| `divider` | `---` |
| `hardBreak`（inline） | `\n`（在 XmlText 间插入换行） |

Inline delta → Markdown：

| delta attributes | 输出 |
|---|---|
| `{ bold: {} }` | `**text**` |
| `{ italic: {} }` | `*text*` |
| `{ code: {} }` | `` `text` `` |
| `{ strike: {} }` | `~~text~~` |
| `{ underline: {} }` | `<u>text</u>`（Markdown 无原生语法） |
| `{ link: { href } }` | `[text](href)` |
| 多个同时存在 | 嵌套包裹，如 `***bold italic***` |

### `markdown_to_yrs` — Markdown → Y.Doc 导入

使用 `pulldown-cmark` 解析 Markdown AST，构建 BlockNote XML schema：

1. 创建 `XmlFragment` → 插入根 `XmlElement("blockGroup")`
2. 遍历 AST 事件，为每个 block 级元素创建 `blockContainer` + 对应 `blockContent`
3. Props 双写到 `blockContainer` 和 `blockContent`（含默认值 `"default"`）
4. 为每个 `blockContainer` 生成唯一 `id`（默认 nanoid，`uuid` feature 启用时用 UUID v7，或调用方传入闭包自定义）
5. 嵌套列表 → 递归创建子 `blockGroup`

### Crate 结构与依赖

```text
crates/yrs-blocknote/
├── Cargo.toml
├── src/
│   ├── lib.rs              — 公共 API、schema 常量、默认 props
│   ├── yrs_to_markdown.rs  — Y.Doc → Markdown
│   └── markdown_to_yrs.rs  — Markdown → Y.Doc
└── tests/
    └── roundtrip.rs        — 双向 roundtrip + 兼容性测试
```

```toml
# crates/yrs-blocknote/Cargo.toml
[package]
name = "yrs-blocknote"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "Bidirectional conversion between Markdown and BlockNote Y.Doc (yrs)"
license = "MIT"

[dependencies]
yrs = "0.21"
pulldown-cmark = { version = "0.12", default-features = false, features = ["html"] }
nanoid = "0.4"

[dependencies.uuid]
version = "1"
features = ["v7"]
optional = true

[features]
default = []
uuid = ["dep:uuid"]   # 启用后 id 生成使用 UUID v7

[dev-dependencies]
yrs = { version = "0.21", features = ["test-utils"] }
```

**ID 生成策略**：默认使用 nanoid 生成随机字符串，启用 `uuid` feature 后切换为 UUID v7。调用方也可通过闭包自定义：

```rust
// 使用默认 id 生成
let doc = yrs_blocknote::markdown_to_doc(md);

// 自定义 id 生成（SwarmNote 使用 UUID v7）
let doc = yrs_blocknote::markdown_to_doc_with(md, || Uuid::now_v7().to_string());
```

**Workspace 集成**（src-tauri/Cargo.toml）：

```toml
[workspace]
members = [".", "entity", "migration", "../crates/yrs-blocknote"]

[dependencies]
yrs-blocknote = { path = "../crates/yrs-blocknote", features = ["uuid"] }
```

## 验收标准

- [ ] `yrs_to_markdown`: Y.Doc XmlFragment → Markdown string，覆盖当前 schema 所有 block 类型
- [ ] `markdown_to_yrs`: Markdown string → Y.Doc XmlFragment，遵循 BlockNote XML schema（含 props 双写、默认值、id 生成）
- [ ] 双向 roundtrip 测试：`md → yrs → md` 对所有支持的 block 类型不丢失内容
- [ ] 与前端 BlockNote 兼容性测试：Rust 生成的 Y.Doc 能被前端 `useCreateBlockNote({ collaboration })` 正确渲染
- [ ] `cargo clippy -- -D warnings` 无警告
- [ ] `cargo test` 全部通过

## 开放问题

- BlockNote 升级改 XML schema 时需同步更新 Rust 转换器，可通过 roundtrip 测试提前发现
- Rust 导出和前端 `blocksToMarkdownLossy` 的输出可能有微小差异，建议统一由 Rust 端导出
- 嵌套列表（blockContainer 嵌套 blockGroup）的递归处理需注意深度限制
- `underline` 在标准 Markdown 中无原生语法，需用 `<u>` HTML 标签
- y-prosemirror 对 overlapping marks 会在 attribute name 后追加 hash 后缀（`markName--<8charHash>`），BlockNote 默认 marks 不涉及，但自定义 mark 可能触发
