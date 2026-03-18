# SwarmNote 产品愿景

## 一句话定位

**P2P 笔记同步工具——设备间免费同步，离线自动合并，数据完全本地。**

## 为什么做这个

两个驱动力：

1. **技术兴趣**：深入探索 P2P 网络 + CRDT 实时协作技术，笔记应用是最佳载体
2. **做有人用的开源产品**：每个技术里程碑都打磨到「别人能用」的程度再往下走

## 核心差异

| | Obsidian + Sync | Syncthing + 编辑器 | **SwarmNote** |
|--|----------------|-------------------|---------------|
| 同步 | 付费 $8/月 | 文件级同步 | **P2P 免费，CRDT 字符级合并** |
| 冲突 | Obsidian 处理 | 生成冲突副本 | **CRDT 自动合并，零冲突** |
| 数据 | Obsidian 服务器 | 本地 | **完全本地，不经过第三方** |
| 协作 | 不支持 | 不支持 | **P2P 实时协作编辑** |
| 设置 | 注册+付费 | 配置 Syncthing | **打开即用，自动发现** |

**一句话差异**：Syncthing 同步文件遇到冲突生成副本让你手动处理，SwarmNote 用 CRDT 做到字符级自动合并——这是文件同步工具做不到的事。

## 目标用户

先服务自己：多台设备的开发者，不想为同步付费，不想折腾文件同步工具，关注数据隐私。

做到自己每天用了，再推广。

---

## 开发路线

核心原则：**技术探索驱动节奏，产品标准决定质量。每个阶段做完都是一个可发布版本。**

### Phase 0：技术基座

给 swarm-p2p-core 添加 GossipSub pub/sub 能力，为文档同步和消息广播打基础。

**产出**：swarm-p2p-core 新版本，SwarmDrop 回归测试通过。

### Phase 1：局域网 P2P 同步（MVP）

核心循环跑通：两台电脑局域网自动发现、同步笔记、离线自动合并。

- Block 编辑器（BlockNote + yjs）
- 文档列表管理
- SQLite 持久化（存储 yjs 二进制更新）
- mDNS 局域网发现 + CRDT 增量/全量同步
- 离线编辑重连自动合并
- 设备标识与连接状态

**验收**：A 编辑 → B 秒级看到 → 关掉 B → A 继续编辑 → 重开 B → 自动追上。

### Phase 2：实时协作

多人同时编辑同一文档，实时看到对方光标。

- yjs Awareness 实时光标/选区
- 编辑器增强（代码高亮、快捷键）
- 同步状态 UI（同步中/已同步/离线待同步）
- 设备管理面板

### Phase 3：跨网络 + 安全

突破局域网限制，任意网络环境都能同步。

- 引导节点 + DHT 跨网络发现
- NAT 穿透（AutoNAT + DCUtR + Relay）
- E2E 加密（XChaCha20-Poly1305）

### Phase 4：内容发布

笔记一键发布为博客/文档网站。

- 文档公开/私有标记
- 静态站点生成 + 一键推送 GitHub Pages

### Phase 5：生态

- MCP Server（让外部 AI 工具读写文档）
- 移动端（Android）
- Obsidian vault 导入
- 双向链接 + 知识图谱
- 全文搜索

---

## Swarm 生态

SwarmNote 是 Swarm 系列产品之一：

```
SwarmDrop   v0.4.4  — 点对点文件传输（已完成）
SwarmNote   v0.1.0  — P2P 笔记同步（开发中）
swarm-p2p-core      — P2P 网络 SDK（已完成，待添加 GossipSub）
```

所有产品共享 swarm-p2p-core 网络层。未来可共享设备信任网络和身份体系。

### 社区节点

用户量增长后，社区可贡献引导节点和中继节点：

- swarm-p2p-core 的 `bootstrap` 模块已有现成实现
- 提供 Docker 镜像 + 一键部署脚本
- 节点越多网络越稳，项目不需要自己维护服务器

---

## 技术选型

| 选择 | 理由 |
|------|------|
| BlockNote + yjs | Notion 风格块编辑器，yjs 协作内置一等支持 |
| yjs（前端 CRDT） | CRDT 是产品基座，离线合并必须从第一天就有，Rust 端透传二进制 |
| swarm-p2p-core + GossipSub | 从 SwarmDrop 复用，需先集成 pub/sub |
| SQLite + rusqlite | 表结构简单，raw SQL 更快 |
| shadcn/ui + Tailwind + Zustand | 和 SwarmDrop 同技术栈 |
| MVP 只做局域网（mDNS） | 最快跑通核心循环 |
