# 设备发现与配对

## 用户故事

作为用户，我希望局域网内的 SwarmNote 设备能自动发现彼此，并在配对确认后才同步笔记，以便只有我信任的设备才能访问我的笔记。

## 依赖

- swarm-p2p-core 集成（需要 P2P 网络层，mDNS + DHT 已内置）

## 需求描述

### 设备发现

利用 swarm-p2p-core 内置的 mDNS 能力，实现局域网内设备的自动发现。设备上线后自动广播存在，其他设备收到广播后展示在"附近设备"列表中。核心实现：

- 监听 `NodeEvent::PeersDiscovered` 事件，自动 `client.dial(peer_id)` 建立连接
- 监听 `NodeEvent::PeerConnected` / `PeerDisconnected` / `IdentifyReceived` / `PingSuccess` 事件更新设备状态
- 通过 `agent_version` 前缀过滤非 SwarmNote 设备
- 维护运行时设备列表 `DashMap<PeerId, PeerInfo>`，通过 Tauri 事件通知前端

### 设备配对

配对是笔记同步的前提条件。支持两种配对方式：

1. **Direct 模式（局域网）**：mDNS 发现设备后，点击"配对"，对方弹窗确认即可。零配置，局域网内最自然的体验。
2. **Code 模式（跨网络）**：一方生成 6 位数字配对码并发布到 DHT，另一方输入码后通过 DHT 查找对方地址，连接并发起配对请求。

配对粒度为设备级——配对一次即信任该设备，双方打开同一工作区时自动同步，无需每个工作区单独授权。

v0.2.0 安全等级：配对码仅用于地址发现和身份确认，传输层靠 libp2p Noise 协议加密。E2E 内容加密推迟到 v0.3.0。

## 交互设计

### 设备列表页

- 分两个区域：**已配对设备** 和 **附近设备**
- 已配对设备：显示设备名、OS、在线/离线状态、最后在线时间
  - 在线设备显示连接类型（LAN/Relay）和延迟
  - 操作：取消配对
- 附近设备：mDNS 发现的未配对设备
  - 操作：点击"配对" → 发起 Direct 配对请求

### Direct 配对流程

```
发起方：点击附近设备的"配对"按钮
    → 发送 PairingRequest 给对方
    → 等待中（显示"等待对方确认..."）

接收方：收到配对请求
    → 弹窗显示对方设备信息（设备名、OS）
    → "接受" / "拒绝"

接受 → 双方互存 PairedDeviceInfo → 配对成功提示
拒绝 → 发起方收到拒绝通知
```

### Code 配对流程

```
生成方：点击"生成配对码"
    → 生成 6 位数字码（5 分钟有效期）
    → 发布到 DHT
    → 界面展示配对码，等待对方输入

输入方：点击"输入配对码"
    → 输入 6 位数字码
    → DHT 查找 → 获取对方地址和设备信息
    → 展示找到的设备信息 → 确认发起配对
    → 发送 PairingRequest（附带配对码）

生成方：收到配对请求
    → 验证配对码是否有效且未过期
    → 弹窗确认
    → 接受后双方互存 → 消耗配对码（一次性使用）
```

### 取消配对

- 在已配对设备列表中，长按或右键菜单 → "取消配对"
- 二次确认弹窗："取消配对后将停止与该设备的笔记同步"
- 确认后删除本地信任记录，断开连接

## 技术方案

### 后端

#### 协议消息定义

```rust
// 扩展 AppRequest/AppResponse（swarm-p2p-core Request-Response 协议）
enum AppRequest {
    Pairing(PairingRequest),
    Sync(SyncRequest),  // 已有
}

enum AppResponse {
    Pairing(PairingResponse),
    Sync(SyncResponse),  // 已有
}

struct PairingRequest {
    device_info: DeviceInfo,       // 发起方设备信息
    method: PairingMethod,
    timestamp: i64,
}

enum PairingMethod {
    Direct,                        // 局域网直连配对
    Code { code: String },         // 配对码模式
}

enum PairingResponse {
    Success,
    Rejected,
    InvalidCode,
    Expired,
}
```

#### 设备管理器（发现层）

```rust
struct DeviceManager {
    peers: DashMap<PeerId, PeerInfo>,  // 运行时发现的所有设备
}

struct PeerInfo {
    peer_id: PeerId,
    addrs: Vec<Multiaddr>,
    agent_version: Option<String>,     // 通过 Identify 协议获得
    rtt_ms: Option<u64>,               // Ping 延迟
    is_connected: bool,
    discovered_at: i64,
    connected_at: Option<i64>,
}
```

- 事件驱动更新：`PeersDiscovered` / `PeerConnected` / `PeerDisconnected` / `IdentifyReceived` / `PingSuccess`
- 过滤 `agent_version` 仅保留 `swarmnote/` 前缀的设备

#### 配对管理器

```rust
struct PairingManager {
    client: NetClient<AppRequest, AppResponse>,
    device_manager: Arc<DeviceManager>,
    paired_devices: DashMap<PeerId, PairedDeviceInfo>,
    active_code: Mutex<Option<PairingCodeInfo>>,      // 当前活跃配对码（单例）
    pending_inbound: DashMap<u64, PendingPairing>,    // 待确认的入站请求
}
```

- `generate_code(expires_in_secs)` — 生成 6 位码，SHA256 哈希后发布到 DHT
- `get_device_info(code)` — 通过 DHT 查找配对码对应的设备
- `request_pairing(peer_id, method)` — 发起配对请求
- `handle_pairing_request(pending_id, response)` — 处理入站配对请求
- `unpair(peer_id)` — 取消配对
- `is_paired(peer_id)` — 检查是否已配对（同步前校验）

#### Tauri Commands

- `generate_pairing_code(expires_in_secs) -> PairingCodeInfo`
- `get_device_by_code(code) -> DeviceInfo`
- `request_pairing(peer_id, method) -> PairingResponse`
- `respond_pairing_request(pending_id, accept: bool) -> ()`
- `get_paired_devices() -> Vec<PairedDeviceInfo>`
- `unpair_device(peer_id) -> ()`
- `get_nearby_devices() -> Vec<DeviceInfo>`

#### Tauri Events

- `pairing-request-received` — 收到配对请求（前端弹窗）
- `paired-device-added` — 配对成功
- `paired-device-removed` — 取消配对
- `nearby-devices-changed` — 附近设备列表更新

#### SQLite 存储

```sql
CREATE TABLE paired_devices (
    peer_id     TEXT PRIMARY KEY,
    hostname    TEXT NOT NULL,
    os          TEXT,
    platform    TEXT,
    arch        TEXT,
    paired_at   INTEGER NOT NULL,
    last_seen   INTEGER
);
```

使用全局数据库（`~/.swarmnote/swarmnote.db`）而非工作区数据库，因为配对是设备级的。

### 前端

#### 状态管理（Zustand）

```typescript
interface PairingStore {
  // 附近设备（mDNS 发现的未配对设备）
  nearbyDevices: DeviceInfo[];
  // 已配对设备
  pairedDevices: PairedDeviceInfo[];
  // 配对流程状态机
  pairingPhase: PairingPhase;
}

type PairingPhase =
  | { phase: "idle" }
  | { phase: "generating"; codeInfo: PairingCodeInfo }
  | { phase: "inputting" }
  | { phase: "searching"; code: string }
  | { phase: "found"; device: DeviceInfo }
  | { phase: "requesting"; peerId: string }
  | { phase: "waiting-confirmation"; peerId: string }  // Direct 模式等待对方确认
  | { phase: "success"; device: PairedDeviceInfo }
  | { phase: "error"; message: string };
```

#### UI 组件

- `DevicesPage` — 设备管理主页面（已配对 + 附近设备）
- `PairedDeviceCard` — 已配对设备卡片（状态 + 取消配对）
- `NearbyDeviceCard` — 附近设备卡片（配对按钮）
- `PairingCodeDialog` — 生成/输入配对码弹窗
- `PairingRequestDialog` — 收到配对请求的确认弹窗
- `UnpairConfirmDialog` — 取消配对二次确认

## 验收标准

- [ ] 两台设备在同一局域网启动后 5 秒内通过 mDNS 自动发现对方
- [ ] 发现的设备通过 `agent_version` 过滤，仅展示 SwarmNote 设备
- [ ] 设备断开后前端及时收到断开通知，列表更新
- [ ] 设备重启后能重新被发现
- [ ] 未配对设备显示在"附近设备"列表
- [ ] Direct 配对：点击配对 → 对方确认 → 双方成功存储信任关系
- [ ] Direct 配对：对方拒绝 → 发起方收到拒绝通知
- [ ] Code 配对：生成 6 位码 → 对方输入 → DHT 查找 → 配对成功
- [ ] Code 配对：配对码过期后无法使用
- [ ] Code 配对：配对码使用一次后自动失效
- [ ] 已配对设备持久化到 SQLite，重启后仍保留
- [ ] 取消配对后删除信任记录，停止同步
- [ ] 未配对设备无法发起或接收笔记同步
- [ ] `cargo clippy -- -D warnings` 无警告
- [ ] `pnpm lint:ci` 通过

## 开放问题

- 配对请求超时时间设多久？（建议 60 秒）
- 是否需要系统通知（当应用不在前台时收到配对请求）
- 已配对设备的设备名变更如何同步更新
- 取消配对是否需要通知对方（单方面 vs 双方同步删除）
