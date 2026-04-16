# Rust 后端

## 架构概览

Cargo workspace 位于 `src-tauri/`，包含 root + `entity` + `migration` 三个 crate。`swarm-p2p-core` 通过 path 依赖（submodule `libs/core/`）引入。

### 模块职责

| 模块 | 职责 |
| ---- | ---- |
| `identity/` | 设备身份（PeerId）、OS keychain、设备名 |
| `workspace/` | 多窗口工作区管理、per-window DB 绑定 |
| `document/` | 文档/文件夹 CRUD（SeaORM） |
| `fs/` | 文件系统 I/O、notify debounce、媒体保存 |
| `network/` | P2P 节点生命周期、事件循环分发、DHT 宣告 |
| `pairing/` | 设备配对码、请求/响应流程 |
| `protocol/` | AppRequest / AppResponse、OsInfo |
| `device/` | 在线设备追踪 |
| `yjs/` | Y.Doc 生命周期、yrs ↔ DB 持久化、auto-save、外部 .md 变更检测 |
| `config/` | 全局配置持久化 |
| `tray.rs` | 系统托盘（桌面端） |
| `error.rs` | `AppError` 统一错误类型 |

## Tauri command 约定

### 使用 `#[tauri::command]` + `AppResult<T>`

```rust
#[tauri::command]
pub async fn my_command(
    state: State<'_, AppState>,
    arg: String,
) -> AppResult<MyResponse> {
    // ...
    Ok(response)
}
```

- 返回类型统一 `AppResult<T>` = `Result<T, AppError>`
- `AppError` 序列化为 `{ kind, message }` JSON 给前端消费
- 参数使用 snake_case，前端 `invoke()` 传参自动 camelCase → snake_case 转换
- `State<'_, T>` 注入共享状态，`AppHandle` 注入应用句柄

**相关文件**：`src-tauri/src/error.rs`、`src-tauri/src/lib.rs`（`generate_handler![]` 注册）

### Capability 声明

所有 command 必须在 `src-tauri/capabilities/*.json` 中 allow 才能被前端调用。Tauri v2 的安全模型，默认拒绝。

**相关文件**：`src-tauri/capabilities/`

### Rust lib 命名避免冲突

Rust lib 名称是 **`swarmnote_lib`** 而不是 `swarmnote`。Windows 下 lib 和 bin 同名会冲突，所以必须加 `_lib` 后缀。

**相关文件**：`src-tauri/Cargo.toml`（`[lib] name`）

## 双数据库

### devices.db (全局) + workspace.db (per-workspace)

- **devices.db**：app data 目录，存配对设备
- **workspace.db**：每个工作区根目录 `.swarmnote/`，存文档/文件夹/工作区元数据

`DbState` 通过 `RwLock<HashMap<String, DatabaseConnection>>` 管理多窗口。每个窗口绑定一个 workspace DB 连接。

**正确做法**：不要假设全局唯一 DB 句柄，用 window label 或 workspace id 做 key 查询。

**相关文件**：`src-tauri/src/workspace/state.rs`

### SeaORM + Uuid v7 主键

所有主键和外键统一 `Uuid`（v7）。ORM 版本 `sea-orm 2.0-rc`。迁移脚本在 `src-tauri/migration/`。

使用规范见 `sea-orm-2` skill。

## 日志

使用 **tracing**，不用 `log`。

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self))]
async fn my_fn(&self, arg: String) -> AppResult<()> {
    info!(%arg, "starting");
    // ...
}
```

**不要做**：`log::info!` / `println!` 在生产代码。

**相关文件**：`src-tauri/src/lib.rs`（tracing_subscriber 初始化）

## P2P 网络（swarm-p2p-core）

### 公开 API 快速参考

```rust
// 启动节点
let (client, mut receiver) = swarm_p2p_core::start::<AppRequest, AppResponse>(
    keypair,
    config,
).await;

// NetClient
client.dial(peer_id);
client.send_request(peer_id, req);
client.send_response(pending_id, resp);
client.bootstrap();
client.start_provide(key);
client.get_providers(key);
client.put_record(record);
client.get_record(key);

// 事件循环
while let Some(event) = receiver.recv().await {
    match event {
        NodeEvent::PeerConnected { peer_id, .. } => { /* ... */ }
        NodeEvent::InboundRequest { request, pending_id, .. } => { /* ... */ }
        // ...
    }
}
```

### 内置能力

- 传输：TCP + QUIC + Noise + Yamux
- 发现：mDNS + Kademlia DHT
- NAT：AutoNAT v2 + DCUtR 打洞 + Relay
- 协议：Request-Response + CBOR

### 事件循环在 `network/event_loop.rs`

`NodeEvent` 被分发给 `DeviceManager`、`PairingManager`，并通过 Tauri `emit` 广播到前端。前端通过 `listen()` 订阅。

**相关文件**：`src-tauri/src/network/event_loop.rs`、`libs/core/`（submodule）

## YDocManager — Y.Doc 生命周期

### per-doc 单例 + auto-save debounce

`YDocManager` 维护 `HashMap<DocUuid, DocState>`，每个 Y.Doc 对应一个 `Y.Doc` 实例 + debounce timer（默认 1.5s）+ fs watcher。

### 外部 .md 变更检测

使用 `notify_debouncer_mini`（100ms debounce）监听 workspace 目录。检测到自己没写过的 .md 改动时：
1. 读取文件内容
2. 调用 `replace_doc_content(&doc, new_md)` 用 `similar` 做 text-diff
3. yjs update origin = "remote"，不标 dirty

**自写检测**：写完 .md 后记录 blake3，notify 触发时如果 hash 一致则忽略。

**相关文件**：`src-tauri/src/yjs/manager.rs`、`src-tauri/src/fs/watcher.rs`

### Schema 容错 restore

`open_doc()` 先 apply 持久化的 `yjs_state` bytes，如果 Y.Text 为空（老版本 BlockNote schema，字段名不同），fallback 用 .md 文件内容 seed Y.Text 并重写 yjs_state。

**相关文件**：`src-tauri/src/yjs/manager.rs` 的 `open_doc`

## 错误处理

- Rust 端统一 `AppResult<T>` + `AppError { kind, message }`
- 不要 `.unwrap()` / `.expect()` 在 production path（测试除外）
- 外部 I/O 错误用 `?` + `From` 实现自动转换
- 业务错误用 `AppError::new(kind, msg)` 显式构造

**相关文件**：`src-tauri/src/error.rs`

## Tauri IPC 推送

### Event emit 约定

事件名使用 kebab-case，payload 结构化：

```rust
app.emit("yjs:flushed", FlushedPayload { doc_uuid })?;
app.emit("peer-connected", PeerPayload { ... })?;
```

前端 `listen<Payload>(eventName, handler)` 订阅。

**约定**：事件名以模块前缀命名（`yjs:*`、`network:*`、`pairing:*` 等）。
