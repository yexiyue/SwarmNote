# BlockNote 技术调研

> 调研时间：2026-03-18

## 简介

[BlockNote](https://www.blocknotejs.org/) 是一个基于 React 的块编辑器（Notion 风格），底层基于 ProseMirror 和 Tiptap。核心功能（包括协作、评论、UI 组件）均为 MIT 协议。

BlockNote 团队是 yjs、Hocuspocus、Tiptap 的核心贡献者。

## yjs 支持

**一等公民支持。** 传入 `Y.Doc` + provider 即可开启协作：

```typescript
import * as Y from "yjs";
import { useCreateBlockNote } from "@blocknote/react";

const doc = new Y.Doc();
const provider = /* 自定义 P2P provider */;

const editor = useCreateBlockNote({
  collaboration: {
    provider,
    fragment: doc.getXmlFragment("document-store"),
    user: { name: "用户名", color: "#ff0000" },
    showCursorLabels: "activity",
  },
});
```

### Provider 生态

BlockNote 支持标准 yjs provider 接口，已有的 provider：

| Provider | 传输方式 | 说明 |
|----------|---------|------|
| y-webrtc | WebRTC P2P | **最接近 SwarmNote 场景** |
| y-websocket | WebSocket | 中心化服务器 |
| y-indexeddb | 本地存储 | 离线持久化 |
| PartyKit | 托管服务 | BlockNote 官方演示用 |
| Liveblocks | 托管服务 | 商业方案 |
| Y-Sweet | Rust CRDT 引擎 | Jamsocket 开源方案 |

**SwarmNote 只需实现一个自定义 yjs provider**，将 yjs updates 通过 swarm-p2p-core GossipSub 广播。

### 服务端工具

`@blocknote/server-util` 提供 yjs ↔ BlockNote blocks 的转换能力：

- `blocksToYDoc(editor, blocks)` — 将 blocks 转为 Y.Doc
- `blocksToYXmlFragment(editor, blocks, fragment)` — 将 blocks 写入 XmlFragment

适用于初始化文档内容，但不应用于协作开始后的 rehydrate（会丢失历史）。

## 与 CodeMirror 6 对比

| | CodeMirror 6 | BlockNote |
|--|-------------|-----------|
| 编辑体验 | 代码编辑器风格 | **Notion 风格块编辑器**，更适合笔记 |
| yjs 集成 | y-codemirror.next（第三方绑定） | **内置一等支持** |
| 协作光标 | 需额外配置 | **开箱即用** |
| 富文本 | 纯文本 + Markdown | **可视化编辑 + Markdown 快捷键** |
| 服务端工具 | 无 | `@blocknote/server-util` |
| 扩展性 | CM Extension API | Block 自定义 + ProseMirror 扩展 |

## 生产案例

- 法国、德国、荷兰政府联合项目 Docs，用 BlockNote 给公务员做协作文档工具
- WordPress 7.0 实时协作基于 yjs 模型（类似架构）

## 对 SwarmNote 的决策

**采用 BlockNote 替代 CodeMirror 6**，理由：

1. 块编辑器更贴合笔记场景（vs 代码编辑器）
2. yjs 协作一等支持，协作光标开箱即用
3. 自定义 provider 接口清晰，适配 swarm-p2p-core 成本低
4. 团队是 yjs 核心贡献者，生态可靠

### 需要关注的点

- BlockNote 存储格式是 ProseMirror XML（非纯 Markdown），导出 Markdown 需要转换
- Rust 端透传 yjs 二进制 blob，不解析 XmlFragment 格式
- `@blocknote/server-util` 依赖 jsdom，仅用于 Node.js 环境，Tauri 后端不需要

## 参考链接

- [BlockNote 官方文档](https://www.blocknotejs.org/)
- [BlockNote 协作文档](https://www.blocknotejs.org/docs/features/collaboration)
- [BlockNote yjs 集成架构 (DeepWiki)](https://deepwiki.com/TypeCellOS/BlockNote/8.1-yjs-integration)
- [Y-Sweet + BlockNote 教程 (Jamsocket)](https://docs.jamsocket.com/y-sweet/tutorials/blocknote)
- [BlockNote GitHub](https://github.com/TypeCellOS/BlockNote)
