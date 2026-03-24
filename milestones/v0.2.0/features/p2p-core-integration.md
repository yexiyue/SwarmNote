# swarm-p2p-core 集成

## 用户故事

作为开发者，我希望将 swarm-p2p-core 集成到 SwarmNote 后端，以便为局域网笔记同步提供 P2P 网络基础设施。

## 依赖

- 无依赖（L0，可独立开始）
- 前置条件：swarm-p2p-core GossipSub 已就绪

## 需求描述

将 swarm-p2p-core 作为 git submodule 引入 SwarmNote Rust 后端，定义笔记同步协议消息类型（SyncRequest/SyncResponse），搭建网络事件循环，通过 Tauri 事件桥接将网络事件传递给前端。

## 技术方案

### 后端

- 引入 swarm-p2p-core 为 git submodule（参考 SwarmDrop 模式）
- 定义同步协议消息：
  ```rust
  // SyncRequest — 请求同步
  enum SyncRequest {
      StateVector { doc_id: String, sv: Vec<u8> },
      FullSync { doc_id: String },
      DocList,
  }

  // SyncResponse — 同步响应
  enum SyncResponse {
      Updates { doc_id: String, updates: Vec<u8> },
      DocList { docs: Vec<DocMeta> },
  }
  ```
- 启动 P2P 节点：`swarm_p2p_core::start::<SyncRequest, SyncResponse>(keypair, config)`
- 创建 Tauri 事件桥接：网络事件 → `app.emit()` / `app.emit_to()`
- Tauri commands：`start_p2p_node`, `stop_p2p_node`, `get_connected_peers`

### 前端

- 监听 Tauri 事件：`peer-connected`, `peer-disconnected`, `sync-received`
- 在 Zustand store 中维护连接状态

## 验收标准

- [ ] swarm-p2p-core 作为 git submodule 集成到项目中
- [ ] SyncRequest/SyncResponse 协议消息定义完成
- [ ] P2P 节点可通过 Tauri command 启动/停止
- [ ] 网络事件正确桥接到 Tauri 事件系统
- [ ] `cargo clippy -- -D warnings` 无警告

## 开放问题

- NodeConfig 参数如何配置（监听端口、bootstrap 节点等）
- 密钥对复用 v0.1.0 Stronghold 中已生成的 Ed25519 keypair
