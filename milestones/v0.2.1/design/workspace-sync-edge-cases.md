# 工作区同步边界情况分析

> 讨论日期：2026-04-02
> 关联 Issue：#46 后端工作区列表交换 API、#28 CRDT 同步

## 协议选型

**工作区列表交换使用 Req-Resp 协议**，不使用 GossipSub。

| 维度 | Req-Resp | GossipSub |
|------|----------|-----------|
| 语义 | 查询-应答，适合"给我你的列表" | 发布-订阅，适合持续流 |
| 寻址 | 点对点（指定 peer_id） | 广播（所有订阅者） |
| 响应 | 同步等待，可超时 | 异步投递，无确认 |
| 适用 | 一次性查询 ✓ | 实时增量更新 |

## 前置重构：WorkspaceState/DbState 主键改为 UUID

### 现状

当前所有运行时状态以 window label 为主键：

```
WorkspaceState:  HashMap<window_label, WorkspaceInfo>
DbState:         HashMap<window_label, DatabaseConnection>
YDocManager:     HashMap<(window_label, doc_id), YDoc>
```

window label 是 Tauri 窗口的临时标识（如 `"main"`, `"ws-abc123"`），不是工作区的身份。UUID 存在 `WorkspaceInfo.id` 中但从不作为索引。

### 问题

同步协议按 UUID 寻址（对方不知道你的 window label），但运行时状态按 label 索引。每次同步操作都需要遍历找 UUID，架构不匹配。

### 重构方案

改为 UUID 做主键，label 作为辅助索引：

```rust
struct WorkspaceState {
    workspaces: RwLock<HashMap<Uuid, WorkspaceInfo>>,  // 主索引
    bindings: RwLock<HashMap<String, Uuid>>,           // label → uuid
}

struct DbState {
    devices_db: DatabaseConnection,
    workspace_dbs: RwLock<HashMap<Uuid, DatabaseConnection>>,  // 主索引
}
```

对外提供两种访问路径：
- Tauri 命令层：`get_by_label(label)` → 内部 resolve label → uuid → 数据
- 同步层：`get_by_uuid(uuid)` → 直接索引

命令签名不变（仍接收 Tauri window label），转换逻辑封装在 state 层。

### 改动范围

| 文件 | 改动 |
|------|------|
| `workspace/state.rs` | 重写主索引 + 加 bind/unbind/resolve 方法 |
| `workspace/commands.rs` | 小改，用 `_by_label` 便捷方法 |
| `document/commands.rs` | 小改，`workspace_db_by_label` |
| `fs/watcher.rs` | key 改 uuid |
| `yjs/manager.rs` | key 改 uuid |
| 前端 | 不影响（label 是 Tauri 内部概念） |

此重构作为 #46 的前置步骤一起实施。

---

## 关键设计决策

### 只返回当前已打开的工作区

设备收到 `WorkspaceRequest::ListWorkspaces` 时，只返回 `DbState.workspace_dbs` 中已打开的工作区，不返回 `recent_workspaces` 中已关闭的。

**理由**：
- 已打开 = DB 已连接，doc_count 可查，后续 DocList/StateVector 可立即响应
- 未打开 = 无 DB 连接，需要临时打开，引入隐式生命周期管理
- `recent_workspaces` 中的路径可能已失效（目录删除/移动），返回后对方无法同步
- 如果工作区没在任何在线设备上打开，即使知道它存在也无法同步

### 协议结构（方案 A：独立 Workspace 变体）

```rust
enum AppRequest {
    Pairing(PairingRequest),
    Workspace(WorkspaceRequest),  // 新增
    Sync(SyncRequest),
}

enum AppResponse {
    Pairing(PairingResponse),
    Workspace(WorkspaceResponse),  // 新增
    Sync(SyncResponse),
}

enum WorkspaceRequest {
    ListWorkspaces,
}

enum WorkspaceResponse {
    WorkspaceList { workspaces: Vec<WorkspaceMeta> },
}

struct WorkspaceMeta {
    uuid: Uuid,
    name: String,
    doc_count: u32,
    updated_at: DateTime<Utc>,
}
```

三个顶层变体对应三个生命周期阶段：
- **Pairing** → 信任建立（谁能跟我通信）
- **Workspace** → 资源发现（对方有什么工作区）
- **Sync** → 数据同步（CRDT 状态交换）

Workspace 独立出来为 v0.3.0 的工作区共享/权限扩展预留空间。

## 边界情况

### 1. 多设备同一工作区的合并展示

**场景**：
```
A 在线:  uuid-1("笔记", 50篇, updated_at: 10:30)
B 在线:  uuid-1("笔记", 47篇, updated_at: 09:15)
C 查询合并列表
```

**方案**：前端按 UUID 去重合并，显示一条带可用来源数：
```
uuid-1  "笔记"  50篇  · 来自 2 台设备
```

- `name`：取 `updated_at` 最新的那个设备的 name
- `doc_count`：取最大值（更完整的副本）
- 同步时选 RTT 最低的 peer（最快完成全量传输）

### 2. 同步过程中设备离线

**场景**：
```
C 正在从 B 全量同步 uuid-2（100篇文档）
已完成 40 篇 → B 突然离线
```

**方案**：全量同步采用 per-doc 的 req-resp 循环，不是一次性大包：
```
DocList → 得到 100 个 doc_id → 逐个 FullSync/StateVector
                                ↓
第 41 个超时 → 标记为"部分同步"
                                ↓
B 重新上线 → 从未完成的 doc_id 继续
```

**同步状态机**：
```
IDLE → FETCHING_DOC_LIST → SYNCING_DOCS → COMPLETED
                                ↑    ↓
                           peer 离线/超时
                                ↓
                         PARTIAL_SYNCED
                                ↓
                         peer 重连 → 从断点继续
```

per-doc 粒度的好处：
- 断点续传天然支持
- 已同步的文档立即可用
- 和现有 `SyncRequest::StateVector/FullSync` 设计一致

### 3. 工作区改名

**场景**：
```
A: uuid-1, 目录名 "Notes"
B: uuid-1, 目录名 "MyNotes"（用户在文件系统重命名了目录）
```

**方案**：v0.2.1 不同步 workspace name。

- `WorkspaceIdentity.name` 由本地目录名派生，每台设备独立
- 合并展示时取 `updated_at` 最新的设备的 name
- 用户创建本地工作区时可自定义目录名
- workspace name 同步推迟到后续版本（需 last-write-wins 或 CRDT）

### 4. 本地已存在同 UUID 工作区

**场景**：
```
C 之前已从 A 同步过 uuid-1
现在查询列表，B 也报告了 uuid-1
```

**方案**：后端命令返回时标记 `is_local: true`：
```
uuid-1  "笔记"  50篇  · 已在本地 ✓  (灰色不可选)
uuid-2  "工作"  30篇  · 来自 B       (可选择同步)
```

**is_local 判断**：
- 优先匹配 `WorkspaceState.infos` 中已打开的工作区 UUID
- 补充扫描 `GlobalConfig.recent_workspaces` 中的 `.swarmnote/workspace.json` UUID
- 或维护全局 `known_workspace_uuids: HashSet<Uuid>` 缓存（启动时初始化）

**注意**：本地已存在但未打开的工作区，UUID 匹配需要读取文件系统上的 `workspace.json`，有 I/O 开销。可在 `get_remote_workspaces` 命令中一次性扫描。

### 5. 并发配对同步

**场景**：
```
A 和 B 同时配对成功
A 向 B 发 ListWorkspaces
B 同时向 A 发 ListWorkspaces
```

**结论**：完全安全。Req-resp 是对称的，两边独立处理。

更极端的场景：
```
A 有 uuid-1 (v1)，B 有 uuid-1 (v2)
A 选择同步 uuid-1 from B，B 同时选择同步 uuid-1 from A
→ 两边同时做 StateVector 交换 → CRDT 合并 → 两边都得到 v1∪v2 → 一致 ✓
```

CRDT 保证了并发同步的安全性，无需额外协调。

### 6. 空工作区

**场景**：
```
B 新建了 uuid-3 但还没写任何笔记
B 报告: {uuid-3, "新工作区", doc_count: 0}
```

**方案**：前端展示但不主动推荐。doc_count=0 的工作区灰色显示或添加提示"空工作区"。技术上允许同步（创建空的本地工作区结构）。

### 7. 请求超时

**场景**：
```
C 向 A、B、D 三台设备发送 ListWorkspaces
A 2s 内响应，B 5s 内响应，D 始终无响应（网络差）
```

**方案**：
- 单个 peer 超时 5s（不阻塞其他 peer）
- 并发向所有已配对在线 peer 发送请求
- 收集到的响应立即返回，超时的 peer 跳过
- 前端可提示"D 设备无响应，部分工作区可能未列出"

### 8. 协议版本不兼容

**场景**：
```
A 是 v0.2.1（支持 WorkspaceRequest）
B 是 v0.2.0（不识别 WorkspaceRequest）
```

**方案**：
- B 收到未知的 AppRequest 变体 → CBOR 反序列化失败 → Req-resp 超时
- A 端捕获超时错误，跳过 B
- 前端提示"B 设备版本过旧，请更新后重试"
- 可选：通过 Identify 协议的 agent_version 字段提前判断对方版本

## WorkspaceMeta 字段设计

```rust
struct WorkspaceMeta {
    uuid: Uuid,          // 跨设备唯一标识
    name: String,        // 目录名派生，用于展示
    doc_count: u32,      // 文档数量（不含已删除）
    updated_at: i64,     // 最后更新时间戳（ms），用于去重排序
}
```

**不包含的字段**：
- `path` — 文件系统路径是设备特有的，不应跨设备传输
- `created_by` — 创建者 PeerId 对同步决策无影响
- `created_at` — 创建时间对同步决策无影响

## RemoteWorkspaceInfo（Tauri 命令返回类型）

```rust
struct RemoteWorkspaceInfo {
    uuid: Uuid,
    name: String,
    doc_count: u32,
    updated_at: i64,
    peer_id: String,     // 来源设备
    peer_name: String,   // 来源设备名
    is_local: bool,      // 本地是否已存在
}
```

前端合并逻辑：按 `uuid` 分组 → 每组取 `updated_at` 最新的 `name` 和最大 `doc_count` → 标记 `sources: Vec<peer_name>`。
