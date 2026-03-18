# MVP 任务拆解（2 周）

## 目标

两台电脑局域网内自动发现、同步笔记、离线编辑后重连自动合并。

## 前置工作：swarm-p2p-core 集成 GossipSub

在开始 SwarmNote 开发之前，先给 swarm-p2p-core 添加 GossipSub 支持：

- [ ] swarm-p2p-core 新增 GossipSub behaviour
- [ ] NetClient 新增 pub/sub API：subscribe(topic)、unsubscribe(topic)、publish(topic, data)
- [ ] NodeEvent 新增消息事件：GossipMessage { topic, data, source }
- [ ] NodeConfig 新增 GossipSub 配置项（可选启用）
- [ ] SwarmDrop 回归测试确保不影响现有功能

---

## Week 1：单机可用 + P2P 基础

### 1. 项目基础设施（Day 1）

- [ ] 引入前端依赖：yjs, y-codemirror.next, @uiw/react-codemirror, zustand, tailwindcss, shadcn/ui
- [ ] 引入 Rust 依赖：yrs, rusqlite (bundled), swarm-p2p-core (git submodule)
- [ ] 配置 Tailwind CSS + shadcn/ui 初始化

### 2. SQLite 持久化层（Day 1-2）

- [ ] 数据库初始化（Tauri app_data_dir）
- [ ] documents 表：doc_id, title, yrs_state (BLOB), state_vector (BLOB), created_at, updated_at
- [ ] updates 表：id, doc_id, update_data (BLOB), created_at
- [ ] 基础 CRUD：create_doc, get_doc, list_docs, delete_doc, rename_doc
- [ ] yrs Update 追加写入 + 定期合并压缩到 yrs_state

### 3. 基础编辑器（Day 2-3）

- [ ] CodeMirror 6 + Markdown 语法高亮
- [ ] y-codemirror.next 绑定 yjs Y.Doc
- [ ] 前端 Y.Doc ←→ Rust yrs::Doc 的 IPC 通道
  - 前端 → 后端：invoke("apply_update", { doc_id, update: Uint8Array })
  - 后端 → 前端：Tauri Event "doc-update" 推送远程变更
- [ ] 自动保存：编辑后 debounce 500ms 写入 SQLite

### 4. 文档管理 UI（Day 3-4）

- [ ] 左侧文档列表（按修改时间排序）
- [ ] 新建文档、删除文档、重命名文档
- [ ] 右侧编辑区域
- [ ] Zustand store：当前文档、文档列表、连接状态

### 5. P2P 网络层（Day 4-5）

- [ ] swarm-p2p-core 集成，定义 SyncRequest / SyncResponse 协议消息
- [ ] mDNS 局域网自动发现
- [ ] 全量同步：新节点连接后交换 state_vector，互发缺失的 updates
- [ ] 增量同步：本地编辑产生 update → GossipSub 广播给所有订阅节点
- [ ] Tauri 事件桥接：网络事件 → 前端 UI 更新

---

## Week 2：同步体验打磨

### 6. 离线合并（Day 6-7）

- [ ] 启动时加载 SQLite 中所有文档的 yrs state
- [ ] 重连后自动触发全量同步（双向 state_vector 交换）
- [ ] CRDT 合并后更新 SQLite + 通知前端刷新
- [ ] 测试场景：A/B 同时离线编辑同一段落 → 重连后内容完整合并

### 7. 设备标识与连接状态（Day 7-8）

- [ ] 本地生成设备昵称（默认：主机名）
- [ ] 连接状态栏：已连接 N 台设备，设备名称列表
- [ ] 同步状态指示：已同步 ✓ / 同步中... / 离线（待同步）

### 8. 文档同步管理（Day 8-9）

- [ ] 新建文档后自动广播给已连接设备
- [ ] 删除文档的同步（软删除标记，避免删后重建）
- [ ] 文档列表的同步：设备 A 新建的文档自动出现在设备 B

### 9. 边缘情况处理（Day 9-10）

- [ ] 应用启动时若有已知节点，自动尝试重连
- [ ] 大文档同步：yrs state 压缩 + 分块传输
- [ ] 网络断开/恢复的优雅处理
- [ ] 并发编辑同一文档的压力测试

### 10. 打磨与测试（Day 10）

- [ ] 两台真机端到端测试
- [ ] UI 细节打磨（加载状态、错误提示）
- [ ] 性能：编辑延迟 < 16ms，同步延迟 < 500ms（局域网）

---

## 数据流概览

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
