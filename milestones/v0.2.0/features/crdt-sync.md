# yjs CRDT 同步

## 用户故事

作为用户，我希望在一台设备上编辑笔记后，其他局域网内的设备秒级看到更新。

## 依赖

- mDNS 局域网发现（需先建立 P2P 连接）
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

### 后端

- GossipSub topic 设计：每个文档一个 topic（`swarmnote/doc/{doc_id}`）
- 打开文档时 subscribe topic，关闭时 unsubscribe
- 收到 GossipSub 消息 → 存储 update + 通知前端 + 写回 .md
- 新连接建立 → 交换所有已打开文档的 state_vector → 互发缺失 updates
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

## 开放问题

- GossipSub topic 粒度：按文档 vs 按工作区
- 文档列表同步的冲突处理（同时创建同名文档）
- 大文档的 yjs state 分块传输策略
