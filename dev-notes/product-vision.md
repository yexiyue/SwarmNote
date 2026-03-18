# SwarmNote 产品设计

## 核心定位

**无服务器的 P2P 笔记同步——数据完全本地，设备间免费同步，离线自动合并。**

做 Obsidian Sync 的去中心化替代品。不需要付费、不需要云服务、不需要折腾 iCloud/Git，打开就能在设备间同步笔记，CRDT 保证零冲突合并。

### 差异化一句话

> 你的笔记在你的设备间自动流转，不经过任何服务器，免费且开源。

### 为什么这个方向？

**P2P 本地优先是真实且增长的需求：**

- 68% 的消费者担心在线隐私
- Obsidian 证明了本地优先笔记有大量用户，但同步要付费（$8/月）或折腾
- GDPR 等数据主权法规推动本地优先架构
- **没有竞品做到了「无服务器的 P2P 笔记同步 + CRDT 自动合并」**

**为什么不以 AI 为核心：**

- AI + 笔记市场拥挤且失败率高（Mem.ai 融资 $2350万后基本失败）
- 「AI 自动整理笔记」没有产品做成功过
- 开发者已经有 Claude Code / Cursor，不需要专门的 AI 笔记工具
- AI 作为可选增强更合理，不应作为产品灵魂

### 与现有方案对比

| 维度 | Obsidian + Sync | Obsidian + iCloud/Git | Syncthing + 编辑器 | **SwarmNote** |
|------|----------------|----------------------|-------------------|---------------|
| 同步方式 | 付费云服务 ($8/月) | 手动折腾，冲突频繁 | 文件级同步 | **P2P 免费，CRDT 字符级合并** |
| 冲突处理 | Obsidian 处理 | 生成冲突副本 | 生成冲突副本 | **CRDT 自动合并，零冲突** |
| 离线能力 | 完整 | 完整但合并痛苦 | 完整但冲突 | **完整 + 零冲突自动合并** |
| 数据主权 | Obsidian 服务器 | iCloud/GitHub | 本地 | **完全本地，不经过第三方** |
| 设置成本 | 注册+付费 | 配置 Git/iCloud | 配置 Syncthing | **打开即用，局域网自动发现** |
| 实时协作 | 不支持 | 不支持 | 不支持 | **CRDT 实时协作编辑** |

### 目标用户

先服务自己：3+ 台设备的笔记用户，不想为同步付费，不想折腾文件同步工具，关注数据隐私。

---

## Swarm 生态

SwarmNote 是 Swarm 生态的一部分：

```
Swarm 生态
├── SwarmDrop   v0.4.4  — 点对点文件传输（已完成）
├── SwarmNote   v0.1.0  — P2P 笔记同步（开发中）
└── swarm-p2p-core      — P2P 网络 SDK（已完成）
```

SwarmDrop 和 SwarmNote 共享：
- **设备信任网络**：配对一次，两个应用都能用
- **身份体系**：Ed25519 密钥对 + Stronghold 加密存储
- **P2P 基础设施**：swarm-p2p-core（mDNS + DHT + NAT 穿透 + 加密传输）

SwarmDrop = 「发给你」（一次性传输）
SwarmNote = 「一起编辑」（持续同步 + 协作）

---

## 功能路线图

### MVP（2 周）—— P2P 同步跑通

**验收标准**：两台电脑打开 SwarmNote → 自动发现 → A 编辑 → B 秒级看到 → 关掉 B → A 继续编辑 → 重开 B → 自动追上所有变更。

| 功能 | 说明 |
|------|------|
| 基础 Markdown 编辑器 | CodeMirror 6 + y-codemirror.next |
| 文档列表 | 创建/删除/重命名，按修改时间排序 |
| SQLite 持久化 | yrs 二进制存储 |
| 局域网 P2P 同步 | mDNS 自动发现 + yrs CRDT 增量/全量同步 |
| 离线合并 | CRDT 离线编辑后重连零冲突合并 |
| 设备标识 | 设备昵称 + 连接状态 |

**前置工作**：swarm-p2p-core 集成 GossipSub。

**明确不做**：AI、加密、跨网络、协作光标、搜索。

### Phase 2（+2 周）—— 日常可用

| 功能 | 说明 |
|------|------|
| 协作编辑 | yjs Awareness 实时光标/选区 |
| 编辑器增强 | 代码块语法高亮、快捷键、自动保存指示器 |
| 同步状态 UI | 同步中 / 已同步 / 离线待同步 / 设备在线状态 |
| 设备管理 | 已配对设备列表、连接历史 |

### Phase 3（+2 周）—— 跨网络 + 安全

| 功能 | 说明 |
|------|------|
| 跨网络同步 | 引导节点 + DHT（复用 SwarmDrop 基础设施） |
| NAT 穿透 | AutoNAT + DCUtR + Relay |
| E2E 加密 | XChaCha20-Poly1305（复用 SwarmDrop 加密方案） |
| 全文搜索 | FTS5 + jieba 中文分词 |
| 版本历史 | yrs snapshot 时间线浏览和回退 |

### Phase 4 —— 生态扩展

| 功能 | 说明 |
|------|------|
| SwarmDrop 身份共享 | 与 SwarmDrop 共用设备信任网络 |
| 移动端 | Android（Tauri mobile） |
| Obsidian vault 导入 | 批量导入 .md 文件 |
| 双向链接 + 图谱 | `[[wikilink]]` 支持 |
| 文件夹 / 标签 | 文档组织能力 |

### Phase 5 —— AI 增强（可选）

AI 不是核心卖点，而是 P2P 本地优先的加分项。学 NotebookLM 模式（用户主动发起、范围明确），不学 Mem.ai（AI 自作主张整理）。

| 功能 | 说明 |
|------|------|
| MCP Server | 暴露 list_docs / read_doc / create_doc / edit_doc 接口 |
| 语义搜索 | 用户主动问「帮我找关于 X 的笔记」 |
| 文档问答 | 上传/选中文档 → 问问题（NotebookLM 模式） |
| 按需摘要/整理 | 用户选中内容 → 「帮我总结」「帮我格式化」 |

**原则**：AI 只在用户明确要求时才行动，不主动修改文档，不自动整理。

---

## 技术方案

### MVP 技术取舍

| 选择 | 理由 |
|------|------|
| CodeMirror 6 + y-codemirror.next | Markdown 原生，yjs 官方绑定 |
| yrs（后端）+ yjs（前端） | CRDT 是产品基座 |
| swarm-p2p-core + GossipSub | 从 SwarmDrop 复用，需先集成 pub/sub |
| MVP 只做局域网（mDNS） | 2 周跑通核心循环 |
| SQLite + rusqlite | 表结构简单，raw SQL 更快 |
| shadcn/ui + Tailwind + Zustand | 和 SwarmDrop 同技术栈 |

### 从 SwarmDrop 复用清单

| 模块 | SwarmDrop 来源 | SwarmNote 用途 |
|------|---------------|---------------|
| swarm-p2p-core | libs/core/ | P2P 网络层 |
| 设备配对 | src-tauri/src/pairing/ | 设备发现与信任 |
| 身份管理 | Ed25519 + Stronghold | 设备身份 |
| E2E 加密 | XChaCha20-Poly1305 | 文档传输加密（Phase 3） |
| MCP Server | src-tauri/src/mcp/ | AI 接口（Phase 5） |
| UI 组件 | shadcn/ui + Tailwind | 前端基础组件 |
| 国际化 | Lingui | 多语言支持 |

---

## 市场调研结论

基于 2025-2026 年 AI + 笔记市场调研的关键发现：

1. **AI 自动整理笔记没有成功案例**：Mem.ai（$2350万融资）基本失败，产品完全重写
2. **成功的 AI+文档产品都是窄场景**：NotebookLM（文档问答，92% 留存）、Granola（会议纪要）
3. **用户讨厌不请自来的 AI**：42% 企业放弃 AI 项目，AI 项目失败率是普通 IT 项目的 2 倍
4. **开发者不需要 AI 笔记工具**：Claude Code / Cursor 已覆盖知识管理需求
5. **P2P 本地优先是真实差异化**：没有竞品做到无服务器 P2P 笔记同步 + CRDT 合并
