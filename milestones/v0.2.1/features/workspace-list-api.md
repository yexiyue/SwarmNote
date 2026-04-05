# 后端工作区列表交换 API

## 用户故事

作为开发者，我需要后端提供工作区列表交换能力，以便前端在配对后能获取对方设备的工作区列表并发起同步。

## 依赖

- 无依赖（L0，可独立开始）
- v0.2.0 P2P 协议（AppRequest/AppResponse）已定义，本功能扩展协议

## 需求描述

v0.2.0 的 crdt-sync.md 中已设计了工作区同步的概念（WorkspaceList 交换），但实际 API 未暴露给前端。v0.2.1 需要：

1. 扩展 P2P 协议，支持查询对方的工作区列表
2. 提供 Tauri command 供前端调用
3. 支持在指定本地路径创建工作区并触发全量同步

## 技术方案

### 协议扩展

在 `AppRequest` / `AppResponse` 中新增工作区相关消息：

```rust
// 扩展 AppRequest
enum AppRequest {
    Pairing(PairingRequest),
    Sync(SyncRequest),
    // 新增
    Workspace(WorkspaceRequest),
}

enum WorkspaceRequest {
    /// 请求对方的工作区列表
    ListWorkspaces,
}

// 扩展 AppResponse
enum AppResponse {
    Pairing(PairingResponse),
    Sync(SyncResponse),
    // 新增
    Workspace(WorkspaceResponse),
}

enum WorkspaceResponse {
    WorkspaceList {
        workspaces: Vec<WorkspaceMeta>,
    },
}

#[derive(Serialize, Deserialize)]
struct WorkspaceMeta {
    uuid: Uuid,
    name: String,
    doc_count: u32,
}
```

### Tauri Commands

```rust
/// 获取所有已配对在线设备的工作区列表
#[tauri::command]
async fn get_remote_workspaces(
    // ...state
) -> AppResult<Vec<RemoteWorkspaceInfo>> {
    // 1. 遍历已连接的已配对 peer
    // 2. 向每个 peer 发送 WorkspaceRequest::ListWorkspaces
    // 3. 收集响应，按设备分组返回
}

/// 从远程设备同步工作区到本地
#[tauri::command]
async fn sync_remote_workspace(
    peer_id: String,
    workspace_uuid: String,
    local_path: String,
    // ...state
) -> AppResult<()> {
    // 1. 创建本地工作区目录 + .swarmnote/
    // 2. 初始化 workspace.db（写入 workspace UUID）
    // 3. 触发全量同步（复用 SyncManager）
    // 4. 全量同步完成后 emit 事件通知前端
}

/// 获取当前设备所有工作区的同步状态概况
#[tauri::command]
async fn get_workspace_sync_status(
    // ...state
) -> AppResult<Vec<WorkspaceSyncInfo>> {
    // 返回每个工作区的同步状态、进度、最后同步时间等
}
```

### 返回类型

```rust
#[derive(Serialize)]
struct RemoteWorkspaceInfo {
    uuid: Uuid,
    name: String,
    doc_count: u32,
    peer_id: String,
    peer_name: String,
    is_local: bool,  // 本地是否已存在此工作区（UUID 匹配）
}

#[derive(Serialize)]
struct WorkspaceSyncInfo {
    uuid: Uuid,
    name: String,
    status: WorkspaceSyncStatus,
    synced_device_count: u32,
    last_sync_at: Option<i64>,
    progress: Option<SyncProgress>,
}

#[derive(Serialize)]
enum WorkspaceSyncStatus {
    Synced,
    Syncing,
    Pending,
    Offline,
}

#[derive(Serialize)]
struct SyncProgress {
    synced: u32,
    total: u32,
}
```

### 事件

```rust
// 工作区同步状态变化
app.emit("workspace-sync-status-changed", WorkspaceSyncInfo { ... });

// 远程工作区同步完成
app.emit("remote-workspace-synced", RemoteWorkspaceSynced {
    uuid: Uuid,
    name: String,
    local_path: String,
});
```

### 处理 WorkspaceRequest

在 `event_loop.rs` 中，收到 `AppRequest::Workspace(WorkspaceRequest::ListWorkspaces)` 时：

1. 遍历当前设备已打开的工作区（`DbState` 中的 HashMap）
2. 从每个 workspace.db 读取工作区元数据（UUID、名称）
3. 统计文档数量
4. 返回 `WorkspaceResponse::WorkspaceList`

## 验收标准

- [ ] `get_remote_workspaces` 能获取所有已配对在线设备的工作区列表
- [ ] 响应中正确标记本地已存在的工作区（`is_local: true`）
- [ ] `sync_remote_workspace` 能在指定路径创建新工作区并触发全量同步
- [ ] 同步完成后 emit `remote-workspace-synced` 事件
- [ ] `get_workspace_sync_status` 返回所有工作区的同步概况
- [ ] 协议扩展与 v0.2.0 现有协议兼容（不破坏已有的 Sync/Pairing 消息）
- [ ] `cargo clippy -- -D warnings` 无警告
- [ ] `cargo test` 通过

## 开放问题

- `get_remote_workspaces` 的超时策略：如果某个 peer 响应慢怎么处理？建议单个 peer 5 秒超时，返回已收到的结果
- 是否需要缓存远程工作区列表？还是每次都实时查询？
- 已关闭的工作区（不在 DbState HashMap 中）是否也应该列入 WorkspaceList？可能需要扫描 config 中的最近工作区列表
