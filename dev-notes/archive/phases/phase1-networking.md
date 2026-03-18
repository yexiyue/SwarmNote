# Phase 1：P2P 网络层

**目标**：建立完整的 P2P 网络基础设施，包括节点身份、设备关联、节点发现、信任认证、NAT 穿透、文件同步，为后续编辑器和加密层提供可靠的传输通道。

**完成标志**：
1. 两台设备（同一局域网或跨网络）经信任确认后，能够通过 Request-Response 同步资源文件（图片等）
2. 同一用户的多台设备能够关联，并自动同步数据

---

## 1.1 libp2p Swarm 初始化

**目标**：应用启动时创建 libp2p Swarm 并监听端口，配置基础协议。

### 依赖配置

- [ ] 添加 libp2p 依赖到 `src-tauri/Cargo.toml`
  ```toml
  libp2p = { version = "0.55", features = [
      "tcp", "noise", "yamux", "mdns",
      "request-response", "kad", "gossipsub",
      "relay", "dcutr", "autonat", "identify",
      "tokio",
  ] }
  tokio = { version = "1", features = ["full"] }
  tracing = "0.1"
  tracing-subscriber = { version = "0.3", features = ["env-filter"] }
  ```

### 模块结构

- [ ] 创建 `src-tauri/src/network/` 模块目录结构
  ```
  src-tauri/src/network/
    mod.rs              # 模块导出
    swarm.rs            # Swarm 创建与事件循环
    behaviour.rs        # NetworkBehaviour 定义
    protocol.rs         # Request-Response 协议定义
    connection.rs       # 连接生命周期管理
    trust.rs            # 节点信任管理
    sync.rs             # 同步状态管理
    error.rs            # 网络错误定义
  ```

### NetworkBehaviour 定义

- [ ] 定义 `SwarmNoteBehaviour`：组合所有子 behaviour
  ```rust
  #[derive(NetworkBehaviour)]
  struct SwarmNoteBehaviour {
      // 基础协议
      identify: identify::Behaviour,           // 节点信息交换（必须最先配置）

      // 节点发现
      mdns: mdns::tokio::Behaviour,            // 局域网发现
      kad: kad::Behaviour<MemoryStore>,        // DHT 跨网络发现

      // 数据传输
      request_response: request_response::Behaviour<SyncCodec>,
      gossipsub: gossipsub::Behaviour,         // 预留给 Phase 2

      // NAT 穿透
      relay_client: relay::client::Behaviour,
      dcutr: dcutr::Behaviour,
      autonat: autonat::Behaviour,
  }
  ```

### Swarm 创建

- [ ] 创建 Swarm 流程：
  1. 加载或生成本地密钥对（见 1.2 密钥持久化）
  2. 构建传输层：`tcp + noise + yamux`
  3. 创建各 Behaviour 实例
  4. 配置 Identify 协议（agent_version, protocols）
  5. 创建 Swarm
  6. 监听端口（默认随机，可配置固定端口）

- [ ] Swarm 事件循环跑在 tokio spawn 后台任务中
- [ ] 实现优雅关闭：收到关闭信号时清理连接

### Tauri 集成

- [ ] 实现 Tauri State：通过 `app.manage()` 共享网络状态
  ```rust
  struct NetworkState {
      swarm_tx: mpsc::Sender<SwarmCommand>,  // 发送命令到 Swarm
      status: Arc<RwLock<NetworkStatus>>,     // 当前网络状态
  }
  ```
- [ ] 实现 Tauri 命令：
  - `get_local_peer_info` → PeerId + 监听地址列表
  - `get_network_status` → 网络状态（在线/离线/受限）

**验证点**：应用启动后，日志输出本节点 PeerId 和监听地址。重启后 PeerId 保持不变。

---

## 1.2 密钥持久化与节点身份

**目标**：确保节点身份（PeerId）在重启后保持一致。

### 密钥存储

- [ ] 定义 `node_identity` 表 Entity：
  ```rust
  pub struct Model {
      pub id: i32,                    // 固定为 1（单行表）
      pub keypair: Vec<u8>,           // libp2p Ed25519 密钥对（加密存储）
      pub created_at: i64,
  }
  ```

- [ ] 密钥生命周期：
  - 首次启动：生成 Ed25519 密钥对 → 加密后存入数据库
  - 后续启动：从数据库加载密钥对 → 解密使用
  - 密钥加密：使用设备唯一标识派生的密钥加密存储

- [ ] 密钥备份与恢复（可选）：
  - 导出密钥助记词（BIP39）
  - 从助记词恢复密钥

### PeerId 与指纹

- [ ] PeerId 派生：从 Ed25519 公钥派生
- [ ] 设备指纹生成：PeerId → SHA256 截断 → 分组显示
  ```
  例：A1B2:C3D4:E5F6:G7H8
  ```
- [ ] 实现 Tauri 命令：
  - `get_device_fingerprint` → 人类可读的设备指纹

**验证点**：启动应用，记录 PeerId。关闭重启，PeerId 保持一致。

---

## 1.3 用户身份与多设备关联

**目标**：支持同一用户在多台设备上使用，设备间自动信任和同步。

### 设计原则

**节点身份 vs 用户身份**：
- 节点身份（PeerId）：每台设备唯一，用于网络层标识
- 用户身份：可跨多台设备，用于应用层标识

**去中心化多设备方案**（类似 Signal）：
- 不依赖 OAuth 等中心化服务
- 通过设备配对建立关联
- 主设备授权新设备加入

### 用户身份

- [ ] 定义 `user_identity` 表 Entity：
  ```rust
  pub struct Model {
      pub id: i32,                      // 固定为 1
      pub user_id: String,              // 用户 UUID
      pub nickname: String,             // 用户昵称
      pub avatar_color: String,         // 头像颜色（hex）
      pub created_at: i64,
      pub updated_at: i64,
  }
  ```

- [ ] 首次启动设置：
  - 生成用户 UUID
  - 引导用户设置昵称和头像颜色
  - 或选择"关联已有设备"

### 设备关联

- [ ] 定义 `linked_devices` 表 Entity：
  ```rust
  pub struct Model {
      pub peer_id: String,              // 关联设备的 PeerId
      pub device_name: String,          // 设备名称（如"MacBook Pro"）
      pub is_primary: bool,             // 是否为主设备
      pub linked_at: i64,
      pub last_seen: i64,
  }
  ```

- [ ] 设备配对流程：
  ```
  主设备 A                              新设备 B
      │                                    │
      │  1. A 生成配对码（6位数字+有效期）  │
      │     显示在屏幕上                    │
      │                                    │
      │  2. B 输入配对码                    │
      │     B → A: PairingRequest          │
      │                                    │
      │  3. A 验证配对码，弹窗确认          │
      │     "设备 B 请求关联，是否允许？"   │
      │                                    │
      │  4. A 确认后，发送用户身份数据      │
      │     A → B: PairingResponse         │
      │     （含 user_id, 用户密钥等）      │
      │                                    │
      │  5. B 保存用户身份，关联完成        │
      │     两设备互相加入 linked_devices   │
  ```

- [ ] 配对码设计：
  - 格式：6 位数字（如 `123456`）
  - 有效期：5 分钟
  - 单次使用，用后作废
  - 可选：显示为 QR 码

### 协议定义

- [ ] 扩展 Request-Response 协议：
  ```rust
  enum SyncRequest {
      // ... 文件同步消息 ...

      /// 设备配对请求
      PairingRequest {
          pairing_code: String,
          device_name: String,
          peer_id: String,
      },
  }

  enum SyncResponse {
      // ... 文件同步消息 ...

      /// 配对响应
      PairingResponse {
          success: bool,
          user_identity: Option<UserIdentityData>,
          error: Option<String>,
      },
  }
  ```

### 关联设备行为

- [ ] 关联设备自动信任：
  - `linked_devices` 中的设备自动标记为 `trusted_always`
  - 无需手动确认信任弹窗

- [ ] 关联设备数据同步：
  - 自动同步所有本地文件
  - 自动同步信任设备列表
  - 自动同步用户设置

### Tauri 命令

- [ ] 实现 Tauri 命令：
  - `get_user_profile` → 昵称 + 头像颜色 + user_id
  - `update_user_profile(nickname, color)`
  - `generate_pairing_code` → 生成配对码
  - `cancel_pairing_code` → 取消配对码
  - `input_pairing_code(code)` → 输入配对码发起关联
  - `respond_pairing_request(peer_id, accept)` → 响应配对请求
  - `list_linked_devices` → 获取关联设备列表
  - `unlink_device(peer_id)` → 解除设备关联

### 可选：OAuth 集成（后续迭代）

> 注：OAuth 作为可选功能，不是 MVP 必须。用于简化多设备场景的身份恢复。

- [ ] 支持 OAuth 登录（Google / GitHub / Apple）
- [ ] OAuth 仅用于：
  - 云端备份用户身份密钥
  - 新设备快速恢复身份（无需配对）
- [ ] OAuth **不用于**：
  - 日常认证（仍使用本地密钥）
  - 文档访问控制（仍使用 E2E 加密）

**验证点**：设备 A 生成配对码 → 设备 B 输入配对码 → A 确认 → 两设备自动同步文件，无需信任弹窗。

---

## 1.4 mDNS 局域网发现

**目标**：自动发现局域网内的 SwarmNote 节点。

- [ ] 配置 mDNS behaviour（使用 tokio runtime）
  ```rust
  let mdns = mdns::tokio::Behaviour::new(
      mdns::Config::default(),
      local_peer_id,
  )?;
  ```
- [ ] 监听 mDNS 事件：
  - `MdnsEvent::Discovered`：记录发现的节点
  - `MdnsEvent::Expired`：标记节点离线

- [ ] 发现节点处理流程：
  1. 发现新节点 → 检查是否为关联设备
  2. 是关联设备 → 自动 dial 连接
  3. 查询 `trusted_devices` 表
  4. `trusted_always` → 自动 dial 连接
  5. `blocked` → 忽略
  6. 未知 / `pending` → 通过 Tauri Event 通知前端

- [ ] 维护已发现节点列表：
  ```rust
  struct DiscoveredPeer {
      peer_id: PeerId,
      addresses: Vec<Multiaddr>,
      discovered_at: Instant,
      last_seen: Instant,
      source: DiscoverySource,  // Mdns / Dht / Manual
  }
  ```

- [ ] 实现 Tauri 命令：
  - `get_discovered_peers` → 发现的节点列表
  - `get_connected_peers` → 已连接节点列表

**验证点**：两台设备在同一局域网启动，日志显示互相发现。关联设备自动连接，非关联设备等待信任确认。

---

## 1.5 节点信任与授权

**目标**：只有经过用户确认的节点才能建立数据连接。

### 设计原则

**设备信任 ≠ 文档授权**

- 设备信任（本阶段）：允许网络连接和基础数据交换
- 文档授权（Phase 3）：通过 E2E 密钥分发控制文档访问权限

**关联设备 vs 外部设备**

- 关联设备：自动信任，无需确认
- 外部设备：需要手动信任确认

### 信任级别

| 级别 | 含义 | 行为 |
|------|------|------|
| `linked` | 关联设备 | 自动连接，完全信任，同步所有数据 |
| `trusted_always` | 始终信任 | 自动连接，不弹窗 |
| `trusted_once` | 本次信任 | 本次会话允许连接，下次重新确认 |
| `pending` | 新发现，未确认 | 不连接，等待用户操作 |
| `blocked` | 已拒绝 | 自动忽略，不提示 |

### 数据模型

- [ ] 定义 `trusted_devices` 表 Entity：
  ```rust
  pub struct Model {
      pub peer_id: String,              // libp2p PeerId
      pub user_id: Option<String>,      // 对方用户 ID（如果已知）
      pub nickname: Option<String>,     // 对方昵称
      pub device_name: Option<String>,  // 对方设备名
      pub fingerprint: String,          // PeerId 的人类可读指纹
      pub trust_level: String,          // linked / trusted_always / trusted_once / pending / blocked
      pub first_seen: i64,
      pub last_seen: i64,
      pub last_connected: Option<i64>,
  }
  ```

### 信任流程

- [ ] 双向信任确认：
  ```
  设备 A                                设备 B
      │                                    │
      │  1. mDNS 互相发现                  │
      │                                    │
      │  2. A 检查 B 的信任状态            │
      │     B 检查 A 的信任状态            │
      │                                    │
      │  3. 如果任一方未信任对方           │
      │     → 该方弹出信任确认弹窗         │
      │                                    │
      │  4. 双方都确认信任后               │
      │     → 建立双向数据连接             │
  ```

- [ ] mDNS 发现新节点时的处理：
  1. 检查是否为关联设备 → 自动信任
  2. 查询 `trusted_devices` 表
  3. `linked` / `trusted_always` → 自动 dial 连接
  4. `blocked` → 忽略
  5. 未知 / `pending` → Tauri Event 通知前端弹窗

- [ ] 前端信任弹窗：
  - 显示：对方昵称（如有）+ 设备指纹
  - 选项：本次信任 / 始终信任 / 拒绝
  - 可选：添加备注名

- [ ] 用户确认后：
  - 写入 `trusted_devices` 表
  - 触发 Swarm dial 建立连接
  - 连接成功后交换用户信息（Identify）

### Tauri 命令

- [ ] 实现 Tauri 命令：
  - `respond_trust_request(peer_id, decision, note)` — 响应信任弹窗
  - `list_trusted_devices` — 获取所有已知设备
  - `update_trust_level(peer_id, level)` — 修改信任级别
  - `update_device_note(peer_id, note)` — 修改设备备注
  - `remove_device(peer_id)` — 删除设备记录

**验证点**：
1. 设备 B 发现设备 A，弹窗询问。选择"始终信任"后连接。重启后自动连接不弹窗。
2. 选择"拒绝"后不连接，且不再弹窗。
3. 关联设备直接连接，不弹窗。

---

## 1.6 连接生命周期管理

**目标**：管理连接的建立、维护、断开和重连。

### 连接状态

```rust
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { established_at: Instant },
    Disconnecting,
}
```

### 连接建立

- [ ] Dial 策略：
  - 优先使用最近成功的地址
  - 并行尝试多个地址（happy eyeballs）
  - 超时时间：30 秒

- [ ] 连接建立后：
  1. 触发 Identify 交换
  2. 更新 `trusted_devices.last_connected`
  3. 触发文件同步（见 1.8）
  4. 通过 Tauri Event 通知前端

### 连接维护

- [ ] Keep-alive：
  - libp2p yamux 内置 keep-alive
  - 配置 interval：30 秒

- [ ] 连接健康检查：
  - 定期 ping（每 60 秒）
  - 超时阈值：10 秒无响应标记为不健康

- [ ] 最大连接数限制：
  - 默认：50 个并发连接
  - 可配置

### 连接断开

- [ ] 断开事件处理：
  - 更新 `trusted_devices.last_seen`
  - 通过 Tauri Event 通知前端
  - 清理相关状态（同步队列等）

- [ ] 优雅断开：
  - 应用退出时主动断开所有连接
  - 发送 goodbye 消息（可选）

### 自动重连

- [ ] 重连策略：
  - 仅对 `linked` 和 `trusted_always` 的节点自动重连
  - 指数退避：1s → 2s → 4s → 8s → ... → 最大 5 分钟
  - 最大重试次数：无限（但有最大间隔）

- [ ] 重连触发条件：
  - 连接意外断开
  - 网络恢复（从离线变为在线）

### Tauri 命令

- [ ] 实现 Tauri 命令：
  - `disconnect_peer(peer_id)` — 主动断开连接
  - `reconnect_peer(peer_id)` — 手动触发重连

**验证点**：
1. 连接建立后，拔掉网线 → 检测到断开 → 插上网线 → 自动重连。
2. 对于 `trusted_once` 的节点，断开后不自动重连。

---

## 1.7 Request-Response 文件同步协议

**目标**：已信任的节点之间通过 Request-Response 同步资源文件。

### 协议定义

- [ ] 定义消息类型：
  ```rust
  #[derive(Serialize, Deserialize)]
  enum SyncRequest {
      /// 请求对方的文件元数据列表
      ListFiles {
          since: Option<i64>,  // 增量：只返回此时间戳之后更新的
      },

      /// 请求指定文件的内容
      GetFile { file_id: String },

      /// 请求文件分块（大文件）
      GetFileChunk {
          file_id: String,
          offset: u64,
          length: u32,
      },

      /// 通知文件删除
      FileDeleted { file_id: String },

      /// 设备配对请求（见 1.3）
      PairingRequest { ... },
  }

  #[derive(Serialize, Deserialize)]
  enum SyncResponse {
      /// 文件列表
      FileList {
          files: Vec<FileMeta>,
          has_more: bool,      // 分页标记
      },

      /// 完整文件数据（小文件）
      FileData {
          file_id: String,
          data: Vec<u8>,
          content_hash: String,
      },

      /// 文件分块数据（大文件）
      FileChunk {
          file_id: String,
          offset: u64,
          data: Vec<u8>,
          is_last: bool,
      },

      /// 文件不存在
      NotFound { file_id: String },

      /// 未授权
      Unauthorized,

      /// 删除确认
      DeleteAck { file_id: String },

      /// 配对响应（见 1.3）
      PairingResponse { ... },
  }

  #[derive(Serialize, Deserialize)]
  struct FileMeta {
      file_id: String,
      filename: String,
      content_hash: String,  // SHA256
      mime_type: String,
      size: u64,
      created_at: i64,
      updated_at: i64,
      is_deleted: bool,      // 软删除标记
  }
  ```

- [ ] 实现 Request-Response codec（基于 bincode 序列化，比 JSON 更紧凑）
- [ ] 定义协议名称：`/swarmnote/sync/1.0.0`

### 请求处理

- [ ] 收到请求时，先检查对方 PeerId：
  - 检查是否为关联设备
  - 检查是否在信任列表中
  - 未信任 → 返回 `Unauthorized`

- [ ] 处理 `ListFiles`：
  - 查询本地 `resources` 表
  - 支持增量查询（`since` 参数）
  - 分页返回（每页 100 条）

- [ ] 处理 `GetFile` / `GetFileChunk`：
  - 从本地存储读取文件内容
  - 小文件（< 1MB）直接返回
  - 大文件分块返回

- [ ] 处理 `FileDeleted`：
  - 本地标记文件为已删除（软删除）
  - 返回 `DeleteAck`

### 大文件处理

- [ ] 分块大小：256KB
- [ ] 大文件阈值：1MB
- [ ] 传输流程：
  1. 请求方发送 `GetFile`
  2. 响应方检查文件大小
  3. 小文件 → 直接返回 `FileData`
  4. 大文件 → 返回 `FileChunk`（第一块）
  5. 请求方收到后继续请求后续 `GetFileChunk`
  6. 直到收到 `is_last: true`

- [ ] 并发控制：
  - 同时最多 3 个文件传输
  - 同一文件的分块串行传输

**验证点**：节点 A 有一个 10MB 的图片，节点 B 信任后能完整同步该图片。

---

## 1.8 同步状态管理

**目标**：管理同步队列、进度追踪、冲突处理。

### 同步状态

```rust
struct SyncState {
    peer_id: PeerId,
    status: SyncStatus,
    queue: VecDeque<SyncTask>,
    current_task: Option<SyncTask>,
    progress: SyncProgress,
    last_sync: Option<Instant>,
}

enum SyncStatus {
    Idle,
    Syncing,
    Error { message: String },
}

struct SyncProgress {
    total_files: usize,
    completed_files: usize,
    current_file: Option<String>,
    current_file_progress: f32,  // 0.0 - 1.0
}
```

### 同步流程

- [ ] 新连接建立后的同步流程：
  1. 发送 `ListFiles` 获取对方文件列表
  2. 与本地文件列表对比（按 `content_hash`）
  3. 找出差异：
     - 对方有、本地无 → 加入下载队列
     - 本地有、对方无 → 触发对方下载（通过 GossipSub 或等待对方主动同步）
     - 双方都有但 hash 不同 → 冲突处理
  4. 按队列逐个下载
  5. 下载完成后存入本地数据库

- [ ] 增量同步：
  - 记录上次同步时间戳
  - 使用 `ListFiles { since }` 只获取增量

### 冲突处理

- [ ] 冲突检测：同一 `file_id`，不同 `content_hash`
- [ ] 冲突策略（MVP）：
  - Last-Write-Wins（以 `updated_at` 较新的为准）
  - 保留冲突版本（重命名为 `filename_conflict_timestamp`）
- [ ] 通知用户：Tauri Event 提示有冲突文件

### 重复检测

- [ ] 按 `content_hash` 检测重复文件
- [ ] 相同内容的文件只存储一份（Content-Addressable Storage）

### 删除同步

- [ ] 本地删除文件：
  1. 标记为软删除（`is_deleted: true`）
  2. 向所有已连接的信任节点发送 `FileDeleted`
  3. 定期清理（7 天后物理删除）

- [ ] 收到删除通知：
  1. 标记本地文件为软删除
  2. 通过 Tauri Event 通知前端

### Tauri 命令与事件

- [ ] Tauri 命令：
  - `get_sync_status` → 当前同步状态
  - `trigger_sync(peer_id)` → 手动触发与某节点同步
  - `cancel_sync(peer_id)` → 取消同步

- [ ] Tauri Event：
  - `sync-started` → 开始同步
  - `sync-progress` → 同步进度更新
  - `sync-completed` → 同步完成
  - `sync-error` → 同步错误
  - `sync-conflict` → 发现冲突文件

**验证点**：
1. 节点 A 导入 5 张图片 → B 连接后自动同步 → 进度条显示 1/5, 2/5, ... → 完成。
2. A 和 B 同时修改同一文件 → 产生冲突 → 保留两个版本并提示用户。

---

## 1.9 本地存储（SeaORM + SQLite）

**目标**：资源文件和设备信息的本地持久化。

### 依赖配置

- [ ] 添加 SeaORM 依赖：
  ```toml
  sea-orm = { version = "~2.0.0-rc", features = [
      "sqlx-sqlite", "runtime-tokio-native-tls", "macros",
  ], default-features = false }
  libsqlite3-sys = { version = "0.30", features = ["bundled", "fts5"] }
  ```

### Migration 子 crate

- [ ] 创建 `migration/` 子 crate（`sea-orm-cli migrate init`）
- [ ] 编写首次迁移，创建以下表：
  - `node_identity` — 节点密钥
  - `user_identity` — 用户身份
  - `linked_devices` — 关联设备
  - `trusted_devices` — 信任设备
  - `resources` — 资源文件
  - `app_config` — 应用配置

### Entity 定义

- [ ] 定义 `resources` 表 Entity：
  ```rust
  pub struct Model {
      pub file_id: String,               // UUID
      pub filename: String,
      pub content_hash: String,          // SHA256 of content
      pub data: Vec<u8>,                 // 文件内容 BLOB
      pub mime_type: String,
      pub size: i64,
      pub source_peer: Option<String>,   // 来源节点 PeerId（本地导入为 None）
      pub is_deleted: bool,              // 软删除标记
      pub created_at: i64,
      pub updated_at: i64,
      pub deleted_at: Option<i64>,
  }
  ```

- [ ] 定义 `app_config` 表 Entity：
  ```rust
  pub struct Model {
      pub key: String,
      pub value: String,
  }
  ```

### 数据库初始化

- [ ] 应用启动时初始化数据库：
  ```rust
  let app_dir = app_handle.path().app_data_dir()?;
  std::fs::create_dir_all(&app_dir)?;
  let db_path = app_dir.join("swarmnote.db");
  let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

  let db = Database::connect(&db_url).await?;
  Migrator::up(&db, None).await?;
  ```

- [ ] 配置 SQLite：
  - WAL 模式（Write-Ahead Logging）防止数据丢失
  - 同步模式：NORMAL

### Tauri 命令

- [ ] 实现 Tauri 命令：
  - `import_file(path)` — 导入本地文件到 resources 表
  - `import_files(paths)` — 批量导入
  - `list_files` — 获取所有资源文件列表（不含已删除）
  - `get_file(file_id)` — 读取文件内容（返回 base64 或 binary）
  - `delete_file(file_id)` — 软删除文件
  - `get_config(key)` — 读取配置
  - `set_config(key, value)` — 写入配置

**验证点**：导入一张图片，关闭应用重启后仍能看到。删除后重启，文件不显示但数据库中有记录。

---

## 1.10 Kademlia DHT（跨网络发现）

**目标**：不在同一局域网的设备通过 DHT 发现和连接。

### DHT 配置

- [ ] 配置 Kademlia behaviour：
  ```rust
  let store = MemoryStore::new(local_peer_id);
  let mut kad_config = kad::Config::default();
  kad_config.set_query_timeout(Duration::from_secs(60));
  kad_config.set_record_ttl(Some(Duration::from_secs(24 * 60 * 60)));

  let kad = kad::Behaviour::with_config(local_peer_id, store, kad_config);
  ```

### 引导节点

- [ ] 引导节点列表管理：
  - 硬编码默认列表
  - 支持配置自定义引导节点
  - 存储在 `app_config` 表

- [ ] 启动时连接引导节点：
  1. 尝试连接所有引导节点
  2. 连接成功后执行 `FIND_NODE(self)` 加入 DHT
  3. 至少一个成功即可

### Provider Records

- [ ] 节点上线时发布 Provider：
  - 为拥有的每个文件发布 `PUT_PROVIDER(hash(file_id))`
  - 为用户 ID 发布 `PUT_PROVIDER(hash(user_id))`（用于多设备发现）

- [ ] 定期刷新（每 12 小时）：
  - Provider Record TTL：24 小时
  - 提前刷新防止过期

### 发现功能

- [ ] 文件发现：
  ```rust
  // 通过文件 ID 发现持有者
  let providers = kad.get_providers(Key::new(&hash(file_id))).await?;
  ```

- [ ] 用户设备发现：
  ```rust
  // 通过用户 ID 发现该用户的所有设备
  let providers = kad.get_providers(Key::new(&hash(user_id))).await?;
  ```

- [ ] Peer Routing：
  ```rust
  // 通过 PeerId 查找地址
  let addresses = kad.get_closest_peers(peer_id).await?;
  ```

### 手动连接

- [ ] 实现 Tauri 命令：
  - `connect_peer(multiaddr)` — 手动输入地址连接
  - `add_bootstrap_node(multiaddr)` — 添加引导节点

**验证点**：部署引导节点，两台不同网络的设备启动后能通过 DHT 发现对方的关联设备。

---

## 1.11 NAT 穿透

**目标**：NAT 后的设备也能被连接。

### AutoNAT 检测

- [ ] 配置 AutoNAT：
  ```rust
  let autonat = autonat::Behaviour::new(
      local_peer_id,
      autonat::Config::default(),
  );
  ```

- [ ] NAT 状态：
  - `Public`：公网可直连
  - `Private`：NAT 后，需要 Relay

- [ ] 检测结果缓存：避免频繁检测

### Relay 客户端

- [ ] 配置 Relay 客户端：
  ```rust
  let relay_client = relay::client::Behaviour::new(
      local_peer_id,
      relay::client::Config::default(),
  );
  ```

- [ ] Relay 预留流程：
  1. AutoNAT 判断为 Private
  2. 向引导节点请求 Relay 预留
  3. 获得 Relay 地址：`/p2p/<relay_id>/p2p-circuit/p2p/<local_id>`
  4. 通过 DHT 发布此地址

### DCUtR 打洞

- [ ] 配置 DCUtR：
  ```rust
  let dcutr = dcutr::Behaviour::new(local_peer_id);
  ```

- [ ] 打洞流程：
  1. 通过 Relay 建立初始连接
  2. DCUtR 尝试打洞
  3. 成功 → 升级为直连
  4. 失败 → 继续使用 Relay

### 网络状态暴露

- [ ] 实现 Tauri 命令：
  - `get_nat_status` → NAT 类型 + 公网地址（如有）
  - `get_relay_status` → Relay 连接状态

**验证点**：两台 NAT 后的设备通过 Relay 建立连接并同步文件。日志显示 DCUtR 打洞尝试结果。

---

## 1.12 引导节点实现

**目标**：提供公共的引导节点服务，支持 DHT Bootstrap 和 Relay。

### 独立程序

- [ ] 创建 `swarmnote-bootstrap` crate：
  ```
  crates/
    swarmnote-bootstrap/
      src/
        main.rs
      Cargo.toml
  ```

- [ ] 功能：
  - DHT Bootstrap Node
  - Relay Server
  - 健康检查 HTTP 接口

### 配置

- [ ] 配置文件格式（TOML）：
  ```toml
  [network]
  listen_addresses = ["/ip4/0.0.0.0/tcp/4001"]
  external_addresses = ["/ip4/1.2.3.4/tcp/4001"]

  [relay]
  enabled = true
  max_circuits = 1000
  max_circuit_duration_secs = 3600

  [health]
  http_port = 8080
  ```

### 部署

- [ ] Dockerfile：
  ```dockerfile
  FROM rust:1.85 AS builder
  WORKDIR /app
  COPY . .
  RUN cargo build --release -p swarmnote-bootstrap

  FROM debian:bookworm-slim
  COPY --from=builder /app/target/release/swarmnote-bootstrap /usr/local/bin/
  EXPOSE 4001 8080
  CMD ["swarmnote-bootstrap"]
  ```

- [ ] docker-compose.yml
- [ ] 部署文档

### 监控

- [ ] 健康检查接口：`GET /health`
- [ ] 指标接口：`GET /metrics`
  - 连接数
  - DHT 查询数
  - Relay 电路数

**验证点**：部署引导节点，客户端能连接并加入 DHT。Relay 功能正常。

---

## 1.13 错误处理与日志

**目标**：完善的错误处理和日志记录。

### 错误定义

- [ ] 定义网络错误类型：
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum NetworkError {
      #[error("Connection failed: {0}")]
      ConnectionFailed(String),

      #[error("Connection timeout")]
      ConnectionTimeout,

      #[error("Peer not trusted")]
      Unauthorized,

      #[error("Protocol error: {0}")]
      ProtocolError(String),

      #[error("Sync error: {0}")]
      SyncError(String),

      #[error("DHT error: {0}")]
      DhtError(String),

      #[error("Relay error: {0}")]
      RelayError(String),

      #[error("Database error: {0}")]
      DatabaseError(#[from] sea_orm::DbErr),
  }
  ```

### 错误恢复策略

| 错误类型 | 恢复策略 |
|---------|---------|
| 连接失败 | 指数退避重试 |
| 连接超时 | 立即重试一次，然后退避 |
| 协议错误 | 记录日志，断开连接 |
| 同步错误 | 重试 3 次后跳过该文件 |
| DHT 错误 | 切换引导节点重试 |

### 日志配置

- [ ] 使用 `tracing` + `tracing-subscriber`：
  ```rust
  tracing_subscriber::fmt()
      .with_env_filter(EnvFilter::from_default_env())
      .with_file(true)
      .with_line_number(true)
      .init();
  ```

- [ ] 日志级别：
  - ERROR：需要用户关注的错误
  - WARN：可恢复的问题
  - INFO：重要事件（连接建立、同步完成）
  - DEBUG：详细流程
  - TRACE：协议细节

- [ ] 日志文件：
  - 路径：`app_data_dir/logs/`
  - 轮转：按天轮转，保留 7 天

### 错误上报到前端

- [ ] Tauri Event：
  - `network-error` → 网络错误通知
  - `sync-error` → 同步错误通知

- [ ] 错误详情：
  ```rust
  struct ErrorEvent {
      code: String,
      message: String,
      recoverable: bool,
      timestamp: i64,
  }
  ```

**验证点**：模拟各种错误场景，确认日志记录完整，前端收到错误通知。

---

## 1.14 前端基础 UI

**目标**：提供网络层功能的基础前端界面。

### 技术栈

- [ ] 安装依赖：
  ```bash
  pnpm add zustand @tauri-apps/api
  pnpm add -D tailwindcss @tailwindcss/vite
  pnpx shadcn@latest init
  ```

### 基础布局

```
┌────────────────────────────────────────────────────┐
│  SwarmNote              [关联设备] [设备管理] ⚙    │
├──────────┬─────────────────────────────────────────┤
│          │                                         │
│ 文件列表  │          文件预览 / 空状态               │
│          │                                         │
│ ┌──────┐ │   [选中文件的预览内容]                   │
│ │ img1 │ │                                         │
│ │ img2 │ │   或                                    │
│ │ img3 │ │                                         │
│ │ ...  │ │   "导入文件开始使用"                     │
│ └──────┘ │                                         │
│          │                                         │
│ [+ 导入] │                                         │
├──────────┴─────────────────────────────────────────┤
│ 🟢 在线 | 已连接 2 个设备 | PeerId: A1B2:C3D4...   │
└────────────────────────────────────────────────────┘
```

### 页面组件

- [ ] 首次使用引导：
  - 设置昵称和头像颜色
  - 选择"新建身份"或"关联已有设备"

- [ ] 设备配对弹窗：
  - 显示配对码（大字体）
  - 倒计时显示有效期
  - 可切换为 QR 码显示

- [ ] 配对码输入弹窗：
  - 6 位数字输入框
  - 确认按钮

- [ ] 配对请求确认弹窗：
  - 显示请求设备信息
  - 确认/拒绝按钮

- [ ] 信任确认弹窗（Dialog）：
  - 触发：后端 Tauri Event 推送新设备发现
  - 显示：设备指纹 + 昵称（如有）
  - 选项：本次信任 / 始终信任 / 拒绝
  - 可添加备注名

- [ ] 设备管理页面：
  - 分组：关联设备 / 信任设备 / 已拒绝
  - 每个设备：昵称、指纹、状态（在线/离线）、最后在线时间
  - 操作：修改信任级别、修改备注、移除

- [ ] 关联设备页面：
  - 当前用户的所有关联设备
  - 添加新设备（生成配对码）
  - 解除关联

- [ ] 文件列表侧边栏：
  - 显示所有本地资源文件（缩略图 + 文件名 + 大小）
  - 导入按钮：选择本地文件导入
  - 同步来源标识（本地导入 / 从某节点同步）
  - 右键菜单：删除

- [ ] 文件预览区：
  - 图片：直接显示
  - 其他：显示文件信息（类型、大小、hash）

- [ ] 底部状态栏：
  - 网络状态指示（在线/离线/受限）
  - 已连接设备数（点击展开列表）
  - 本节点 PeerId（截断显示）
  - 同步进度（有同步时显示）

- [ ] 同步进度指示：
  - 正在从远程节点同步文件时显示
  - 文件名 + 进度条 + 百分比

- [ ] 设置页面：
  - 用户信息编辑（昵称、颜色）
  - 网络配置（监听端口、引导节点）
  - 日志级别
  - 关于

### 状态管理（Zustand）

- [ ] Store 定义：
  ```typescript
  interface AppStore {
      // 用户信息
      userProfile: UserProfile | null;

      // 设备列表
      linkedDevices: Device[];
      trustedDevices: Device[];
      discoveredPeers: Peer[];
      connectedPeers: Peer[];

      // 文件列表
      files: FileMeta[];
      selectedFile: string | null;

      // 网络状态
      networkStatus: NetworkStatus;
      syncStatus: SyncStatus;

      // Actions
      loadUserProfile: () => Promise<void>;
      loadFiles: () => Promise<void>;
      // ...
  }
  ```

### Tauri Event 监听

- [ ] 监听后端事件：
  - `peer-discovered` → 更新发现列表
  - `peer-connected` → 更新连接列表
  - `peer-disconnected` → 更新连接列表
  - `trust-request` → 弹出信任确认弹窗
  - `pairing-request` → 弹出配对请求确认弹窗
  - `sync-progress` → 更新同步进度
  - `file-added` → 更新文件列表
  - `file-deleted` → 更新文件列表
  - `network-error` → 显示错误提示

**验证点**：完整端到端流程：
1. 首次启动 → 设置昵称 → 进入主界面
2. 设备 A 生成配对码 → 设备 B 输入 → 关联成功
3. A 导入图片 → B 自动同步 → B 预览图片
4. 非关联设备发现 → 信任弹窗 → 确认后连接

---

## 执行顺序

```
1.9 本地存储 ──┐
               │
1.2 密钥持久化 ├──> 1.3 用户身份 ──> 1.4 mDNS ──> 1.5 信任 ──> 1.6 连接管理 ──┐
               │       与多设备                                              │
1.1 Swarm 初始化┘                                                            │
                                                                             │
1.7 文件同步 ──> 1.8 同步状态 ───────────────────────────────────────────────┼──> 1.14 前端
                                                                             │
1.10 DHT ──> 1.11 NAT 穿透 ──> 1.12 引导节点 ────────────────────────────────┘

1.13 错误处理与日志（贯穿整个开发过程）
```

### 阶段划分

**阶段 A：基础设施（可独立验证）**
- 1.1 Swarm 初始化
- 1.2 密钥持久化
- 1.9 本地存储
- 1.13 错误处理与日志

**阶段 B：局域网功能（可独立验证）**
- 1.3 用户身份与多设备
- 1.4 mDNS 发现
- 1.5 节点信任
- 1.6 连接管理
- 1.7 文件同步
- 1.8 同步状态管理

**阶段 C：跨网络功能**
- 1.10 Kademlia DHT
- 1.11 NAT 穿透
- 1.12 引导节点

**阶段 D：前端集成**
- 1.14 前端基础 UI

---

## 里程碑

| 里程碑 | 内容 | 验证标准 |
|--------|------|---------|
| M1 | 基础设施就绪 | Swarm 启动，PeerId 持久化，数据库可用 |
| M2 | 局域网 P2P | 两台设备局域网内发现、信任、同步文件 |
| M3 | 多设备关联 | 同一用户多台设备配对关联，自动同步 |
| M4 | 跨网络 P2P | 不同网络的设备通过 DHT 发现并同步 |
| M5 | 完整 UI | 前端界面完成，端到端流程顺畅 |
