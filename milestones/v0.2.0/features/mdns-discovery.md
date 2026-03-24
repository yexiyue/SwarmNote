# mDNS 局域网发现

## 用户故事

作为用户，我希望在同一局域网内运行的 SwarmNote 设备能自动发现彼此，无需手动配置 IP 地址。

## 依赖

- swarm-p2p-core 集成（需先完成网络层搭建）

## 需求描述

利用 swarm-p2p-core 内置的 mDNS 能力，实现局域网内 SwarmNote 设备的自动发现。设备上线后自动广播存在，其他设备收到广播后自动建立连接。

## 技术方案

### 后端

- swarm-p2p-core 的 NodeConfig 启用 mDNS（默认已启用）
- 监听 `NodeEvent::PeersDiscovered` 事件，自动 `client.dial(peer_id)` 建立连接
- 监听 `NodeEvent::PeerConnected` / `PeerDisconnected` 事件
- 维护已连接 peers 列表，通过 Tauri 事件通知前端

### 前端

- 监听设备发现/连接/断开事件
- Zustand store 维护 `connectedPeers: PeerInfo[]`

## 验收标准

- [ ] 两台设备在同一局域网启动后 5 秒内自动发现对方
- [ ] 设备发现后自动建立 P2P 连接
- [ ] 设备断开后前端收到断开通知
- [ ] 同一设备重启后能重新被发现

## 开放问题

- 设备昵称如何交换（连接建立后的握手协议？）
