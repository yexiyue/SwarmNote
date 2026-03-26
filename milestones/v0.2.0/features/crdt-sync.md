# yjs CRDT 同步

## 用户故事

作为用户，我希望在一台设备上编辑笔记后，其他局域网内的设备秒级看到更新。

## 依赖

- 设备配对（只同步已配对设备，需先完成配对流程）
- 编辑器 yjs 集成（需要 yjs 数据层支持）

## 需求描述

实现两种同步模式：
1. **增量同步**：本地编辑产生的 yjs updates 通过 GossipSub 实时广播给所有已连接设备
2. **全量同步**：新设备连接或重连时，通过 Request-Response 交换 state_vector，互发缺失的 updates

## 技术方案

### 同步协议

```
增量同步（实时）:
  编辑 → yjs update → GossipSub publish(topic=doc_id, data=update)
  接收 → GossipSub message → apply to Y.Doc → 写回 .md

全量同步（连接/重连时）:
  A → SyncRequest::StateVector { doc_id, sv_a } → B
  B → SyncResponse::Updates { doc_id, missing_for_a } → A
  B → SyncRequest::StateVector { doc_id, sv_b } → A
  A → SyncResponse::Updates { doc_id, missing_for_b } → B
```

### 全量同步优先级策略

全量同步（首次连接或长时间离线重连）可能涉及大量文档，需要按优先级排序，避免同步中断时用户正在操作的文档迟迟未更新：

1. **P0 — 当前打开的文档**：用户正在编辑/查看的文档最先同步，秒级完成
2. **P1 — 最近编辑的文档**：按 `updated_at` 降序，优先同步近期活跃文档
3. **P2 — 其余文档**：按字母序或创建时间，后台逐步追平

中断恢复：后端维护每篇文档的同步状态（`synced` / `syncing` / `pending`），重连后从上次断点继续，不重复已同步的文档。

### 文档同步状态

后端为每篇文档维护同步状态，通过 Tauri 事件通知前端：

```rust
enum DocSyncStatus {
    Synced,    // 已与所有已连接设备同步
    Syncing,   // 正在接收/发送 updates
    Pending,   // 排队等待同步（全量同步中尚未轮到）
    LocalOnly, // 仅本地修改，未连接任何设备
}
```

Tauri 事件：`doc-sync-status-changed { doc_id, status }`

### 后端

- GossipSub topic 设计：每个文档一个 topic（`swarmnote/doc/{doc_id}`）
- 打开文档时 subscribe topic，关闭时 unsubscribe
- 收到 GossipSub 消息 → 存储 update + 通知前端 + 写回 .md
- 新连接建立 → 按优先级交换文档的 state_vector → 互发缺失 updates
- 文档列表同步：连接后交换 DocList，发现新文档自动拉取

### 前端

- 监听同步事件，实时更新 Y.Doc
- 本地编辑 → invoke Rust → Rust 负责 GossipSub 广播

## 验收标准

- [ ] A 编辑后 B 秒级看到更新（局域网 < 500ms）
- [ ] 增量同步通过 GossipSub 正确广播和接收
- [ ] 全量同步通过 state_vector 交换正确完成
- [ ] 3 台设备同时在线，同步无遗漏
- [ ] 新建文档自动同步到其他设备
- [ ] 删除文档同步到其他设备（软删除标记）
- [ ] 全量同步按优先级排序：当前文档 → 最近编辑 → 其余文档
- [ ] 全量同步中断后重连，从断点继续而非重新开始
- [ ] 每篇文档的同步状态正确更新并通知前端

## 开放问题

- GossipSub topic 粒度：按文档 vs 按工作区
- 文档列表同步的冲突处理（同时创建同名文档）
- 大文档的 yjs state 分块传输策略
