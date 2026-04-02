# yjs CRDT 同步

## 用户故事

作为用户，我希望在一台设备上编辑笔记后，其他局域网内的设备秒级看到更新。

## 依赖

- ✅ 设备配对（已完成，#26）
- ✅ 编辑器 yjs 集成（已完成，YDocManager）
- ✅ 文档全局 UUID 稳定化（已完成，扫描建库 + upsert）
- ✅ 工作区全局 UUID（已完成，`.swarmnote/workspace.json`）
- ✅ 工作区列表交换 API（已完成，#46）
- ✅ UUID-first 状态重构（已完成，WorkspaceState/DbState 主键为 UUID）

## 技术依赖

| 依赖 | 说明 |
|------|------|
| `yrs = { version = "0.25", features = ["sync"] }` | 启用 Send+Sync bounds，直接用底层 SV/Update API（不用 Protocol trait） |
| `pathdiff = "0.2"` | save_media 绝对路径转工作区相对路径 |
| BlockNote `resolveFileUrl` | 已内置，渲染时将相对路径转为 tauri:// URL |

不需要新增：y-sync（已归档）、y-sweet（服务器架构）、relative-path（解决不同问题）

## 需求描述

实现两种同步模式：
1. **增量同步**：本地编辑产生的 yjs updates 通过 GossipSub 实时广播给所有已连接设备
2. **全量同步**：新设备连接或重连时，通过 Request-Response 交换 state_vector，互发缺失的 updates

## 已就绪的基础设施（Phase 1 已完成）

### 工作区身份

- **Source of truth**：`.swarmnote/workspace.json` 持久化 `WorkspaceIdentity { uuid, name, created_at }`
- `ensure_identity()` 实现优先级：文件 > DB 回退 > 新生成
- 每次 `open_workspace` 时自动同步 workspace.json ↔ DB

### 文档 UUID 稳定化

- **工作区扫描建库**：`bind_workspace_to_window()` 时调用 `reconcile_with_db()`，为磁盘上所有 .md 文件分配 `Uuid::now_v7()` 并写入 DB
- **open_doc upsert**：`YDocManager::open_doc()` 使用 `INSERT OR IGNORE` + `SELECT`，基于 UNIQUE(workspace_id, rel_path) 保证幂等
- **不再懒生成**：所有文档在打开工作区时即有稳定 UUID

### 删除 Tombstone

- `deletion_log` 表已创建（migration `m20260330_000003`）
- `delete_document_by_rel_path()` 和 `delete_documents_by_prefix()` 已实现：
  - 写入 tombstone（doc_id, rel_path, deleted_at, deleted_by, lamport_clock+1）
  - ON CONFLICT 更新（支持重复删除）
  - 然后删除 documents 记录

### 协议定义

已在 `protocol/mod.rs` 中完整定义（需扩展资源同步变体）：

```rust
// 顶层路由
enum AppRequest  { Pairing(..), Workspace(..), Sync(SyncRequest) }
enum AppResponse { Pairing(..), Workspace(..), Sync(SyncResponse) }

// 同步子协议（含资源同步扩展）
enum SyncRequest {
    DocList { workspace_uuid: Uuid },     // 查询指定工作区的文档列表
    StateVector { doc_id, sv: Vec<u8> },  // 发送 SV，请求缺失 updates
    FullSync { doc_id },                  // 请求完整文档状态
    AssetManifest { doc_id },             // 请求文档的资源清单
    AssetChunk { doc_id, name, chunk_index },  // 分块拉取资源文件
}
enum SyncResponse {
    DocList { docs: Vec<DocMeta> },
    Updates { doc_id, updates: Vec<u8> },
    AssetManifest { doc_id, assets: Vec<AssetMeta> },
    AssetChunk { doc_id, name, chunk_index, data: Vec<u8>, is_last },
}

// 文档元数据
struct DocMeta {
    doc_id: Uuid,
    rel_path: String,
    title: String,
    updated_at: i64,           // 毫秒时间戳（协议层统一用 i64）
    deleted_at: Option<i64>,   // None = 活跃, Some = tombstone
    lamport_clock: i64,        // 单调递增版本时钟
    workspace_uuid: Uuid,      // 跨设备工作区匹配
}

// 资源元数据
struct AssetMeta { name: String, hash: Vec<u8>, size: u64 }
```

### DB Schema

**documents 表**（当前字段）：

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Uuid (PK) | 全局稳定 UUID |
| workspace_id | Uuid (FK) | 所属工作区 |
| folder_id | Option\<Uuid\> | 所属文件夹 |
| title | String | 文档标题 |
| rel_path | String | 相对路径（UNIQUE with workspace_id） |
| file_hash | Option\<Vec\<u8\>\> | Blake3 哈希（自写检测） |
| yjs_state | Option\<Vec\<u8\>\> | 完整 Y.Doc 状态（V1 编码） |
| state_vector | Option\<Vec\<u8\>\> | Y.Doc state vector（V1 编码） |
| lamport_clock | i64 | 单调版本时钟 |
| created_by | String | 创建者 PeerId |
| created_at | DateTimeUtc | ISO8601 TEXT |
| updated_at | DateTimeUtc | ISO8601 TEXT |

**deletion_log 表**：

| 字段 | 类型 | 说明 |
|------|------|------|
| doc_id | Uuid (PK) | 被删文档 UUID |
| rel_path | String | 删除时的相对路径 |
| deleted_at | DateTimeUtc | ISO8601 TEXT |
| deleted_by | String | 删除者 PeerId |
| lamport_clock | i64 | 删除时的版本时钟 |

### 运行时状态

**UUID-first 索引**（`workspace/state.rs`）：

```rust
// DbState: 主键 UUID，label 辅助索引
workspace_dbs: RwLock<HashMap<Uuid, DatabaseConnection>>   // 主索引
label_to_uuid: RwLock<HashMap<String, Uuid>>               // label → uuid

// WorkspaceState: 同样 UUID-first
workspaces: RwLock<HashMap<Uuid, WorkspaceInfo>>            // 主索引
bindings: RwLock<HashMap<String, Uuid>>                     // label → uuid
```

同步层通过 UUID 直接索引，Tauri 命令层通过 `_by_label()` 便捷方法。

### GossipSub API（swarm-p2p-core）

```rust
client.subscribe(topic: impl Into<String>) -> Result<bool>
client.unsubscribe(topic: impl Into<String>) -> Result<bool>
client.publish(topic: impl Into<String>, data: Vec<u8>) -> Result<()>

// 事件
NodeEvent::GossipMessage { source: Option<PeerId>, topic: String, data: Vec<u8> }
NodeEvent::GossipSubscribed { peer_id, topic }
NodeEvent::GossipUnsubscribed { peer_id, topic }
```

### 事件循环占位

`event_loop.rs` 中已有路由框架，Sync 和 GossipSub 为 warn/info 占位：

```rust
AppRequest::Sync(_) => warn!("sync handler not yet implemented");
NodeEvent::GossipMessage { .. } => info!("handler not yet implemented");
```

### YDocManager 关键 API

当前以 `(window_label, doc_uuid)` 为 key 管理已打开的文档：

```rust
pub struct YDocManager {
    docs: DashMap<(String, Uuid), Arc<DocEntry>>,
}

// DocEntry 内部方法（供 SyncManager 参考）
async fn apply_update(&self, update: &[u8]) -> AppResult<()>
async fn encode_full_state(&self) -> Vec<u8>
async fn snapshot(&self) -> AppResult<DocSnapshot>  // { yjs_state, state_vector, markdown }
```

持久化路径：`persist_snapshot()` → 写 DB (yjs_state + state_vector) + 写 .md 文件。

---

## 技术方案

### 同步协议

```mermaid
sequenceDiagram
    participant A as 设备 A
    participant B as 设备 B

    Note over A,B: 全量同步（已配对 peer 连接时自动触发）

    A->>B: SyncRequest::DocList
    B->>A: SyncResponse::DocList { docs: [...] }

    Note over A: diff 出需要同步的文档<br/>按优先级排序

    loop 逐文档同步（Semaphore 控制并发）
        A->>B: SyncRequest::StateVector { doc_id, sv_a }
        B->>A: SyncResponse::Updates { doc_id, missing_for_a }
        B->>A: SyncRequest::StateVector { doc_id, sv_b }
        A->>B: SyncResponse::Updates { doc_id, missing_for_b }
    end

    A->>A: emit "sync-progress" 事件
    Note over A,B: 全量同步完成，切换到增量模式

    Note over A,B: 增量同步（实时编辑）

    A->>A: 用户编辑 → yjs update
    A->>B: GossipSub publish(topic, update)
    B->>B: apply update → 写回 .md
```

### 增量同步（GossipSub）

- **topic 粒度**：按文档 `swarmnote/doc/{doc_uuid}`
- 打开文档时 `client.subscribe(topic)`，关闭时 `client.unsubscribe(topic)`
- 编辑产生 yjs update → `client.publish(topic, update)` 广播
- 收到 GossipSub 消息 → apply to Y.Doc → persist → 通知前端

### 全量同步（Request-Response）

- **触发时机**：已配对 peer 连接时自动触发 + 手动"重新同步"按钮
- **优先级排序**：
  1. P0 — 当前打开的文档（秒级完成）
  2. P1 — 最近编辑的文档（按 `updated_at` 降序）
  3. P2 — 其余文档（后台逐步追平）
- **并发控制**：Semaphore 限制并发文档数（参考 SwarmDrop 的 8 并发 chunk 模式）
- **进度事件**：节流 200ms emit `sync-progress` 给前端（预留 UI 接口，v0.2.0 暂不做进度 UI）

### Y.Doc 生命周期：已打开 vs 未打开文档

全量同步需要处理工作区中**所有文档**，但 YDocManager 只管理当前在编辑器中打开的文档。

**策略**：SyncManager 直接在 DB 层操作未打开的文档。

- **未打开的文档**：从 DB 读取 `yjs_state` → 创建临时 Y.Doc → 计算 state vector / 应用 updates → 持久化回 DB + 写 .md → 丢弃临时 Doc
- **已打开的文档**：通过 YDocManager 的 `apply_update()` 方法操作内存中的 Doc，由 YDocManager 的 debounce writeback 自动持久化

判断文档是否已打开：查询 `YDocManager.docs` 是否包含目标 doc_uuid。

### 文档同步状态

```rust
enum DocSyncStatus {
    Synced,    // 已与所有已连接设备同步
    Syncing,   // 正在接收/发送 updates
    Pending,   // 排队等待同步（全量同步中尚未轮到）
    LocalOnly, // 仅本地修改，未连接任何设备
}
```

Tauri 事件：`doc-sync-status-changed { doc_id, status }`

### 删除同步（Tombstone 机制）

#### 同步流程

```
Peer A 删除文档：
  1. 删除 .md 文件
  2. 写入 deletion_log（doc_id, now(), self_peer_id, clock+1）  ← 已实现
  3. 删除 documents 记录                                        ← 已实现
  4. 下次 DocList 交换时包含 tombstone

Peer B 收到带 tombstone 的 DocList：
  ├─ B 有此 doc 且 B.clock < tombstone.clock → 应用删除
  ├─ B 有此 doc 且 B.clock >= tombstone.clock → 冲突（删后又改，保留 B 版本）
  └─ B 没有此 doc → 忽略

Peer B 没有此 doc 且 deletion_log 中也没有 → 说明是新文档，拉取
```

#### Tombstone GC

- 所有已配对 peer 确认后 **且** 超过 30 天 → 可 GC
- 超过 6 个月 → 强制 GC（处理永久离线设备）
- 解除配对时，从 ack 要求列表中移除该设备

#### UX（简化版）

- 删除 = 直接软删除 + tombstone，暂不做 Trash UI
- 后续版本可加 Trash 页面（30 天可恢复 → 永久删除）

### 新文档同步

全量同步时发现对方有本地不存在的文档：
1. 检查 `deletion_log` — 如果有 tombstone 且 clock >= 对方 clock → 忽略（已删除）
2. 不在 `deletion_log` 中 → 自动拉取：`SyncRequest::FullSync { doc_id }` → 创建 .md + DB 记录

### 资源文件同步

文档关联的图片等资源文件存放在 `.assets` 后缀的同名目录中（Typora 惯例）：

```text
notes/my-note.md → notes/my-note.assets/screenshot-af3b9e2c.png
```

**关键设计**：

- **hash 文件名**：`save_media` 保存时加 blake3 前 8 位后缀（`screenshot-af3b9e2c.png`），从创建时消除跨设备同名冲突，天然去重
- **紧跟文档同步**：per-doc Y.Doc 同步完成后立即同步该文档的资源目录
- **分块传输**：256KB/块，4 并发拉取，参考 SwarmDrop 模式（简化版，无 bitmap 断点续传）
- **增量同步资源**：GossipSub 只发 yrs update，对方检测到缺失资源后通过 Req-Resp 按需拉取
- **删除跟随**：文档 tombstone 同步后一并删除资源目录

详见 [资源文件同步设计](../design/asset-sync.md)。

### 后端架构

- 新增 `sync/` 模块，包含：
  - `manager.rs` — SyncManager（task-based，非状态机），DashMap 防重入 + CancellationToken
  - `full_sync.rs` — 全量同步 async 函数（DocList → diff → per-doc sync + 资源同步）
  - `doc_sync.rs` — 单文档同步（SV 交换 / FullSync / 资源目录同步 / 临时 Doc 操作）
- SyncManager 集成到 NetManager，与 DeviceManager / PairingManager 平级
- event_loop 中 `AppRequest::Sync(_)` 分发到 SyncManager
- event_loop 中 `NodeEvent::GossipMessage` 分发到 SyncManager
- `PeerConnected` 事件（已配对 peer）→ 触发全量同步

### 前端

- 监听 `sync-progress` 和 `doc-sync-status-changed` 事件
- 本地编辑 → invoke Rust → Rust 负责 GossipSub 广播
- 资源未就绪时显示加载中占位符，拉取完成后自动替换
- 前端进度 UI 预留接口（v0.2.0 暂不实现，侧边栏底部预留位置）

## 验收标准

### 文档同步

- [ ] A 编辑后 B 秒级看到更新（局域网 < 500ms）
- [ ] 增量同步通过 GossipSub 正确广播和接收
- [ ] 全量同步通过 state_vector 交换正确完成
- [ ] 3 台设备同时在线，同步无遗漏
- [ ] 新建文档自动同步到其他设备
- [ ] 删除文档通过 tombstone 同步到其他设备
- [ ] 删除后又编辑的冲突正确处理（Lamport clock 大者胜）
- [ ] 全量同步按优先级排序：当前文档 → 最近编辑 → 其余文档
- [ ] 全量同步中断后重连，从未完成的文档继续
- [ ] 每篇文档的同步状态正确更新并通知前端

### 资源同步

- [ ] 文档关联的图片等资源跟随文档同步（`.assets/` 目录）
- [ ] 资源分块传输（256KB/块，4 并发）
- [ ] 增量同步时新插入的图片通过 Req-Resp 补发
- [ ] 两台设备独立插入同名图片不冲突（hash 文件名）
- [ ] 文档删除时资源目录一并清理

### 代码质量

- [ ] `cargo clippy -- -D warnings` 无警告
- [ ] `pnpm lint:ci` 通过

## 工作区同步

### 工作区身份（已实现）

每个工作区有全局 UUID，存储在 `.swarmnote/workspace.json`。创建工作区时由 `ensure_identity()` 生成，打开工作区时自动同步到 DB。

### 工作区传播流程（#46 已实现列表交换）

```
配对完成后：
  A → WorkspaceRequest::ListWorkspaces → B
  B → WorkspaceResponse::WorkspaceList { workspaces: [...] } → A
  A → 合并所有 peer 响应，前端显示远程工作区列表
  A → 用户选择要同步的工作区 + 指定本地目录
  A → 创建本地工作区目录 + .swarmnote/ + workspace.db
  A → 开始全量同步
```

### 多工作区同步策略

- 每个工作区独立同步，各自有独立的 DocList / state_vector 交换
- 配对是设备级别的，工作区同步是工作区级别的
- 一台设备可以选择只同步部分工作区

## 实现路线

```
Phase 1: 底层重构（✅ 已完成）
  ├─ ✅ 工作区 UUID（.swarmnote/workspace.json 持久化）
  ├─ ✅ 文档 UUID 稳定化（扫描建库 + open_doc upsert + UNIQUE 约束）
  ├─ ✅ deletion_log 表 + Lamport clock 基础设施
  ├─ ✅ 协议定义（SyncRequest/SyncResponse/DocMeta 完整）
  ├─ ✅ UUID-first 状态重构（WorkspaceState/DbState）
  └─ ✅ 工作区列表交换 API（#46）

Phase 1.5: 前置重构（同步的前提条件）
  ├─ Y.Doc 相对路径重构：uploadFile 返回相对路径，移除 asset_url_prefix
  │   （不做则跨设备同步图片全部裂开）
  ├─ save_media 改为 hash 文件名（screenshot-af3b9e2c.png）
  ├─ 资源目录改为 .assets 后缀（my-note.assets/）
  ├─ DB 连接生命周期修复（多窗口同一工作区的 unbind 竞态）
  ├─ scan_workspace_tree 过滤 .assets 目录
  ├─ SyncRequest::DocList 增加 workspace_uuid 参数
  └─ Sync 协议扩展（AssetManifest / AssetChunk）

Phase 2: 全量同步
  ├─ SyncManager 骨架（task-based）+ NetManager 集成
  ├─ DocList 构建（从 DB 查询 documents + deletion_log）
  ├─ 入站 SyncRequest 处理（event_loop 集成）
  ├─ 已配对 peer 连接 → 自动触发对称全量同步
  ├─ per-doc StateVector 交换 + Updates 互发
  ├─ 未打开文档：临时 Y.Doc 加载/合并/持久化
  ├─ 已打开文档：通过 YDocManager.apply_sync_update() 协调
  ├─ 新文档拉取 + 删除 tombstone 处理
  ├─ per-doc 资源目录同步（AssetManifest → 分块拉取缺失资源）
  ├─ 优先级排序 + Semaphore 并发控制
  └─ 进度事件 emit（Tauri 事件，预留 UI 接口）

Phase 3: 增量同步
  ├─ open_doc 时 subscribe GossipSub topic
  ├─ close_doc 时 unsubscribe
  ├─ YDocManager.apply_update() → publish yjs update（裸二进制）
  ├─ GossipMessage 接收 → apply + persist + 通知前端
  ├─ 新增资源检测 → Req-Resp 按需拉取
  └─ 丢失补偿（定期 state_vector 校验）
```

## 设计决策记录

| 决策 | 选择 | 理由 |
| ---- | ---- | ---- |
| 增量同步协议 | GossipSub | 天然支持多设备广播，swarm-p2p-core 已实现 |
| 全量同步协议 | Request-Response | 点对点拉取，支持优先级和并发控制 |
| GossipSub topic 粒度 | 按文档 `swarmnote/doc/{doc_uuid}` | 精确控制流量，只收打开的文档的更新 |
| 全量同步触发 | 自动（连接时）+ 手动重试 | 符合 local-first 自动化理念，手动兜底异常 |
| 文档身份 | 创建时全局 UUID，同步时传播 | 支持重命名，避免 rel_path 匹配歧义 |
| 无 DB 记录的文档 | 工作区扫描时统一建库 | 确保所有 .md 都有稳定 UUID |
| 删除同步 | 软删除 + tombstone + Lamport clock | 防止 resurrection，参考 Syncthing |
| 新文档同步 | 自动拉取并创建 | 配对后自动同步一切，无需手动 |
| 工作区身份 | `.swarmnote/workspace.json` 为 source of truth | 跨设备匹配，DB 为 runtime mirror |
| 工作区传播 | Req-Resp ListWorkspaces（#46） | 一次性查询，非持续流 |
| 协议顶层结构 | Workspace 独立变体 | Pairing/Workspace/Sync 三阶段生命周期 |
| 状态主键 | UUID-first，label 辅助索引 | 同步层按 UUID 寻址，Tauri 命令层按 label |
| 未打开文档同步 | SyncManager 直接 DB 操作 + 临时 Y.Doc | 不侵入 YDocManager，职责分离 |
| 全量同步对称性 | 两边独立发起 | CRDT 幂等消除协调需求 |
| SyncManager 模式 | Task-based（非状态机） | async fn 天然是状态机 |
| DocList 参数 | 加 workspace_uuid | per-workspace 同步，不泄露无关信息 |
| GossipSub 消息 | 裸二进制 | 接收方文档必定已打开，无需 wrapper |
| 资源目录命名 | `.assets` 后缀（Typora 惯例） | 关联直观，文件管理器中自然相邻 |
| 资源文件名 | blake3 hash 后缀 | 创建时消除跨设备冲突，天然去重 |
| 资源传输 | 分块 256KB + 4 并发 | 参考 SwarmDrop，避免大文件超时 |
| 资源同步时机 | 紧跟文档 Y.Doc 同步 | 打开文档时图片就绪 |
| 增量资源同步 | GossipSub 文本 + Req-Resp 补发资源 | GossipSub 有 64KB 限制 |
| 增量资源检测 | Rust 端 debounce 2s 后向 source peer 请求 AssetManifest diff | 完全复用全量同步的 sync_doc_assets，无需解析 Y.Doc 内容；source=None 时跳过，等全量补偿 |
| 同名文档冲突 | Lamport clock 裁决：clock 大者保留原名，小者重命名 (1)，相同时 uuid 字典序破平 | 两设备独立计算结果一致，保证最终一致性；复用 fs/crud.rs rename 流程 |
| 对称同步冗余 | 接受冗余，不做去重优化 | CRDT 幂等保证正确性，SV diff 为空时开销极小；保持对称设计简洁性 |
| Y.Doc URL 策略 | 存储相对路径，渲染时转换 | 设备无关，天然可同步 |
| 进度 UI | v0.2.0 预留接口不做 UI | 先跑通后端和事件 |
| 时间戳格式 | DB: ISO8601 TEXT / 协议: i64 毫秒 | DB 可读性 + 协议紧凑性 |

## 设计文档

- [CRDT 同步架构设计](../design/crdt-sync-architecture.md) — SyncManager、全量/增量同步协议、Y.Doc 生命周期、边界条件
- [资源文件同步设计](../design/asset-sync.md) — 资源目录命名、hash 文件名、分块传输、协议扩展

## 开放问题

- 工作区元数据同步（工作区名称修改、文件夹结构同步）
- Tombstone GC 精确触发条件和实现时机
