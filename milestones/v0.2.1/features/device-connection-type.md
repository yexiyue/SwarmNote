# 设备连接类型识别

## 用户故事

作为 SwarmNote 用户，我希望能看到每台已连接设备的连接方式（局域网/打洞/中继），以便了解同步链路的质量和延迟来源。

## 依赖

- swarm-p2p-core 集成（v0.2.0 已完成）
- 设备发现与配对（v0.2.0 已完成）

## 需求描述

当前后端 `PeerInfo.connection_type` 字段存在但永远为 `None`，无法区分设备的实际连接方式。需要从 libp2p 事件和 multiaddr 中提取连接类型信息，分为三种：

| 类型 | 标识 | 含义 | 延迟预期 |
|------|------|------|----------|
| `lan` | 局域网 | mDNS 发现的局域网直连 | < 10ms |
| `dcutr` | 打洞 | DCUtR 协议打洞成功的直连 | 10-100ms |
| `relay` | 中继 | 通过 Relay 服务器中转 | 50-200ms |

### 现状分析

1. **`ConnectionEstablished` 事件**：当前用 `..` 丢弃了 `endpoint` 信息，无法判断连接路径
2. **`HolePunchSucceeded` 事件**：仅打日志，未更新设备的 `connection_type`
3. **mDNS 发现**：触发 dial 后不记录发现来源，无法标记为 LAN
4. **Relay 连接**：multiaddr 中有 `P2pCircuit` 协议标记但未提取

### 实现方案

#### 1. swarm-p2p-core 层

在 `NodeEvent` 中补充连接元数据：

- `PeerConnected` 事件增加 `connection_type` 字段
- 新增 `ConnectionType` 枚举：`Lan | Direct | Relay | Unknown`
- `HolePunchSucceeded` 事件携带 peer_id（当前已有）

#### 2. 连接类型判定逻辑

```text
ConnectionEstablished 触发时：
  1. 检查 endpoint 的 remote_addr 是否包含 /p2p-circuit → Relay
  2. 检查该 peer 是否来自 mDNS 发现 → Lan
  3. 否则 → Direct（DHT 直连）

HolePunchSucceeded 触发时：
  更新该 peer 的 connection_type: Relay → DCUtR
```

#### 3. DeviceManager 更新

- `set_connected()` 方法增加 `connection_type` 参数
- `PeerInfo.connection_type` 从 `Option<String>` 改为 `ConnectionType` 枚举
- 打洞成功时调用 `update_connection_type(peer_id, DCUtR)`

#### 4. 前端暴露

- `PeerInfo` 序列化时包含 `connectionType: "lan" | "dcutr" | "relay" | null`
- 前端 TypeScript 类型同步更新

## 交互设计

- 设备列表中每个在线设备显示彩色连接类型 badge
- 局域网：绿色底 + WiFi 图标 + `局域网 Xms`
- 打洞：蓝色底 + Zap 图标 + `打洞 Xms`
- 中继：橙色底 + RadioTower 图标 + `中继 Xms`
- 离线设备不显示 badge
- 设计稿：[pencil-shadcn.pen](../design/pencil-shadcn.pen)（Settings: 设备 Tab）

## 技术方案

### 前端

- 涉及组件：`PairedDeviceCard.tsx`、设备设置页
- 新增 `ConnectionBadge` 组件，根据 `connectionType` 渲染对应颜色/图标
- `pairingStore` 中 `PeerInfo` 类型增加 `connectionType` 字段

### 后端

**swarm-p2p-core (`libs/core/`)**：

```rust
// event.rs 新增
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Lan,    // mDNS 局域网直连
    Direct, // DHT 发现后直连
    Relay,  // 中继连接
    DCUtR,  // 打洞成功后的直连
}

// NodeEvent::PeerConnected 增加字段
PeerConnected {
    peer_id: PeerId,
    connection_type: ConnectionType,
}
```

**event_loop.rs (core)**：
- `ConnectionEstablished` 处理中提取 endpoint，解析 multiaddr
- mDNS 发现的 peer 记录到 HashSet，连接时查询
- 检测 `/p2p-circuit` 标识 Relay 连接

**device/mod.rs (tauri)**：
- `PeerInfo.connection_type: Option<ConnectionType>`（替换原 `Option<String>`）
- `set_connected(peer_id, connection_type)` 更新
- `HolePunchSucceeded` → `update_connection_type(peer_id, DCUtR)`

**pairing/commands.rs**：
- `PairedDeviceInfo` 序列化时包含 `connectionType`

## 验收标准

- [ ] 局域网内 mDNS 发现并连接的设备显示绿色「局域网」badge
- [ ] 通过 Relay 中继连接的设备显示橙色「中继」badge
- [ ] DCUtR 打洞成功后 badge 从「中继」自动切换为蓝色「打洞」
- [ ] badge 旁显示实时 RTT 延迟（ms）
- [ ] 设备断线后 badge 消失，重连后正确显示新的连接类型
- [ ] `PeerInfo` API 返回 `connectionType` 字段
- [ ] 连接类型变化时前端实时更新（通过事件推送）

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- swarm-p2p-core 是 git submodule，修改需要同步提交到 core 仓库，是否需要先发 core 的版本？
- DHT 直连（Direct）是否需要单独的 badge 样式，还是不显示 badge（因为很少见）？
- 连接升级（Relay → DCUtR）时是否需要 toast 通知用户？
