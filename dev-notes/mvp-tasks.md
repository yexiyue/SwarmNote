# MVP 任务拆解

## 目标

两台电脑局域网内自动发现、同步笔记、离线编辑后重连自动合并。

## Phase 0：swarm-p2p-core 集成 GossipSub

在开始 SwarmNote 开发之前，先给 swarm-p2p-core 添加 GossipSub 支持：

- [ ] 新增 GossipSub behaviour
- [ ] NetClient 新增 pub/sub API：subscribe / unsubscribe / publish
- [ ] NodeEvent 新增事件：GossipMessage { topic, data, source }
- [ ] NodeConfig 新增 GossipSub 配置项（可选启用）
- [ ] SwarmDrop 回归测试确保不影响现有功能

---

## Phase 1：MVP

### Week 1：单机可用 + P2P 基础

**1. 项目基础设施（Day 1）**

- [ ] 前端依赖：yjs, @blocknote/core, @blocknote/react, @blocknote/mantine, zustand, tailwindcss, shadcn/ui
- [ ] Rust 依赖：yrs, rusqlite (bundled), swarm-p2p-core (git submodule)
- [ ] Tailwind CSS + shadcn/ui 初始化

**2. SQLite 持久化层（Day 1-2）**

- [ ] 数据库初始化（Tauri app_data_dir）
- [ ] documents 表：doc_id, title, yrs_state (BLOB), state_vector (BLOB), created_at, updated_at
- [ ] 基础 CRUD：create_doc, get_doc, list_docs, delete_doc, rename_doc
- [ ] yrs Update 追加写入 + 定期合并压缩

**3. 编辑器（Day 2-3）**

- [ ] BlockNote 编辑器初始化（@blocknote/react + @blocknote/mantine）
- [ ] yjs collaboration 配置（Y.Doc + XmlFragment）
- [ ] 前端 Y.Doc ←→ Rust yrs::Doc 的 IPC 通道
- [ ] 自动保存：编辑后 debounce 500ms 写入 SQLite

**4. 文档管理 UI（Day 3-4）**

- [ ] 左侧文档列表（按修改时间排序）
- [ ] 新建/删除/重命名文档
- [ ] Zustand store：当前文档、文档列表、连接状态

**5. P2P 网络层（Day 4-5）**

- [ ] swarm-p2p-core 集成，定义 SyncRequest / SyncResponse 协议消息
- [ ] mDNS 局域网自动发现
- [ ] 全量同步：新节点连接后交换 state_vector，互发缺失 updates
- [ ] 增量同步：本地编辑 → GossipSub 广播
- [ ] Tauri 事件桥接：网络事件 → 前端 UI

### Week 2：同步体验打磨

**6. 离线合并（Day 6-7）**

- [ ] 启动时加载所有文档的 yrs state
- [ ] 重连后自动全量同步（双向 state_vector 交换）
- [ ] 合并后更新 SQLite + 通知前端刷新

**7. 设备与连接状态（Day 7-8）**

- [ ] 设备昵称（默认主机名）
- [ ] 状态栏：已连接设备数 + 名称列表
- [ ] 同步指示：已同步 / 同步中 / 离线待同步

**8. 文档同步管理（Day 8-9）**

- [ ] 新建文档自动广播给已连接设备
- [ ] 删除文档同步（软删除标记）
- [ ] 文档列表同步

**9. 边缘情况 + 测试（Day 9-10）**

- [ ] 启动时自动重连已知节点
- [ ] 大文档分块传输
- [ ] 网络断开/恢复处理
- [ ] 两台真机端到端测试
- [ ] 性能目标：编辑 < 16ms，同步 < 500ms（局域网）

---

## 数据流

```
┌─────────────────────────────────┐
│          React 前端              │
│                                 │
│  CodeMirror 6                   │
│    ↕ y-codemirror.next          │
│  Y.Doc (yjs)                    │
│    ↕ Uint8Array                 │
│  invoke() / listen()            │
└────────────┬────────────────────┘
             │ Tauri IPC (binary)
┌────────────▼────────────────────┐
│        Rust 后端                 │
│                                 │
│  yrs::Doc (CRDT 引擎)           │
│    ↕                            │
│  ┌──────────┐  ┌──────────────┐ │
│  │ SQLite   │  │ swarm-p2p-   │ │
│  │ 持久化    │  │ core 网络层   │ │
│  └──────────┘  └──────────────┘ │
└─────────────────────────────────┘
```

## 同步协议

```
设备 A                              设备 B
  │                                    │
  │── mDNS 发现 ──────────────────────>│
  │<─────────────────── mDNS 响应 ─────│
  │                                    │
  │── SyncRequest(state_vector_a) ────>│
  │<── SyncResponse(missing_updates) ──│
  │── SyncResponse(missing_updates) ──>│
  │                                    │
  │═══ GossipSub 实时增量广播 ══════════│
```
