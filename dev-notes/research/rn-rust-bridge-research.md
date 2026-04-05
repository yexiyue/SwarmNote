# React Native 桥接 Rust 代码方案调研

> 调研日期：2026-03-26
> 目的：评估 swarmnote 移动端使用 RN + Rust 架构的可行性，复用 swarmdrop 的 P2P 核心

---

## 一、背景

swarmnote 后续需要做移动端。swarmdrop 已有成熟的 Rust 后端（libp2p P2P 网络、E2E 加密传输、SeaORM 持久化等），目前通过 Tauri 的 `#[tauri::command]` 暴露给前端。目标是找到一种方案，让 React Native 能像 Tauri 一样调用 Rust 代码，实现核心逻辑跨平台复用。

---

## 二、方案对比

### 2.1 uniffi-bindgen-react-native (Mozilla / Filament)

- **仓库**: [jhugman/uniffi-bindgen-react-native](https://github.com/jhugman/uniffi-bindgen-react-native) | 484 stars
- **原理**: 基于 Mozilla UniFFI，通过 `#[uniffi::export]` 注解 Rust 代码，自动生成 TypeScript + C++ (JSI) 绑定，最终生成标准 React Native Turbo Module
- **链路**: JS → Hermes JSI → C++ → Rust (xcframework / .so)
- **DX 示例**:

```rust
// Rust 端
#[uniffi::export]
pub async fn say_after(ms: u64, who: String) -> String {
    format!("Hello, {who}!")
}
```

```typescript
// TypeScript 端 — 自动生成类型，直接函数调用
const message = await sayAfter(1000n, "World");
// 支持 AbortSignal 取消
const msg = await sayAfter(60000n, "World", { signal: controller.signal });
```

- **类型安全**: 完全自动生成 TS 类型声明，支持枚举、结构体、对象（带方法和 GC 集成）、回调
- **异步**: 完整支持。async fn → Promise，��持 AbortSignal 取消（内部 drop Future），双向异步调用
- **平台**: iOS + Android + WASM (Web)
- **成熟度**: Mozilla 已在 Firefox 移动端大规模使用，服务数亿用户
- **评价**: **当前首选**，DX 最接近 Tauri，功能最完整

### 2.2 Craby

- **仓库**: [leegeunhyeok/craby](https://github.com/leegeunhyeok/craby) | 220 stars
- **原理**: 与 uniffi 方向相反 — 从 TypeScript Schema 出发生成 Rust/C++ 绑定，直接集成纯 C++ TurboModule
- **DX**: 在 TS 中定义模块接口 (Spec)，运行 `craby generate` 生成 Rust 骨架，用 `#[craby_module]` 实现
- **独特功能**: Signals（Rust → JS 单向事件推送）
- **平台**: iOS + Android
- **成熟度**: RC 阶段，社区较小
- **限制**: 不支持返回 Uint8Array / ArrayBuffer
- **评价**: TypeScript-first 设计对前端开发者友好，但成熟度不够

### 2.3 Ferric + Node-API (Callstack)

- **仓库**: [callstackincubator/react-native-node-api](https://github.com/callstackincubator/react-native-node-api) | 183 stars
- **原理**: 利用 Node-API (N-API) 作为 ABI 稳定接口，基于 napi-rs 生成绑定
- **DX**: 使用 `#[napi]` 宏（与 Node.js napi-rs 生态完全一致）
- **核心优势**: 与 Node.js / Deno / Bun 生态共享代码，预构建二进制分发
- **平台**: iOS + Android
- **成熟度**: 早期开发，依赖 Hermes 自定义版本
- **评价**: **未来最值得关注**，一旦 Hermes 官方合并 Node-API 将成为标准方案

### 2.4 其他方案（不推荐）

| 方案 | 问题 |
|---|---|
| **jsi-rs** (211 stars) | 已停止维护 (2024-10)，仅支持 Android，async 阻塞式 |
| **Nitro + Rust** | Nitro 成熟但 Rust 非官方支持，需手写 C FFI 胶水层 |
| **手动 C Bridge + JSI** | 全手工，容易内存泄漏，工作量大 |

---

## 三、与 Tauri 的 DX 对比

| 维度 | Tauri | uniffi-bindgen-rn | Ferric (napi-rs) |
|---|---|---|---|
| **定义方式** | `#[tauri::command]` | `#[uniffi::export]` | `#[napi]` |
| **调用方式** | `invoke('name', args)` | 直接函数调用 | 直接函数调用 |
| **序列化** | JSON (serde) | 无（JSI 直通） | 无（Node-API 直通） |
| **类型安全** | 运行时 (serde) | 编译时 + 生成 TS | 编译时 + 生成 TS |
| **IPC 开销** | 较高（WebView fetch） | 低（JSI 直调） | 低（Node-API） |
| **异步** | async command | async fn → Promise + AbortSignal | async fn → Promise |

**结论**: RN 方案在性能和类型安全上甚至优于 Tauri（绕过 WebView 层，无 JSON 序列化开销）。

---

## 四、SwarmDrop 代码桥接可行性分析

### 4.1 架构现状

```
swarmdrop/
├── libs/core/           ← swarm-p2p-core (纯 Rust，无 Tauri 依赖)
│   ├── client/          ← NetClient API
│   ├── runtime/         ← libp2p behaviours
│   ├── config.rs        ← NodeConfig
│   └── event.rs         ← NodeEvent 枚举
├── src-tauri/src/       ← 应用层逻辑 (重度依赖 Tauri)
│   ├── commands/        ← Tauri IPC 命令
│   ├── network/         ← NetManager
│   ├── pairing/         ← PairingManager
│   ├── transfer/        ← SendSession/ReceiveSession + 加密
│   ├── file_source/     ← 文件枚举/读取
│   ├── file_sink/       ← 文件写入 (Android SAF)
│   ├── database/        ← SeaORM + SQLite
│   └── mcp/             ← MCP server
└── src/                 ← React 前端
```

### 4.2 可直接桥接的部分 (~60%)

| 模块 | 说明 | UniFFI 兼容性 |
|---|---|---|
| `libs/core` | 纯 Rust P2P 核心，无 Tauri 依赖 | ✅ 直接 `#[uniffi::export]` |
| `transfer/crypto.rs` | XChaCha20-Poly1305 加密 | ✅ 纯算法 |
| `pairing/` 核心逻辑 | 配对协议、DHT 操作 | ✅ 业务逻辑可复用 |
| `transfer/` 核心逻辑 | 发送/接收会话、分块传输 | ✅ 协议层可复用 |
| `database/` | SeaORM + SQLite | ✅ SQLite 在移动端原生支持 |
| `protocol.rs` | CBOR 请求/响应定义 | ✅ 纯数据结构 |

### 4.3 需要适配/重写的部分 (~40%)

| 模块 | 问题 | 解决方案 |
|---|---|---|
| `commands/` | `#[tauri::command]` | → `#[uniffi::export]` |
| 事件系统 | `app.emit()` Tauri 专有 | → uniffi callback interface |
| Stronghold 密钥存储 | `tauri-plugin-stronghold` | → RN 侧 `expo-secure-store` + Rust 接收密钥 |
| `file_sink/android_ops.rs` | Tauri Android 插件 (SAF) | → RN 侧处理文件选择，Rust 侧只做 I/O |
| `network/` NetManager | `tauri::AppHandle` 管理状态 | → 独立 Rust struct + `Arc` |

### 4.4 关键依赖移动端兼容性

| 依赖 | 可用 | 备注 |
|---|---|---|
| libp2p 0.56 | ✅ | 已有移动端案例 (IPFS Mobile, Firefox) |
| tokio | ✅ | 移动端标准 async runtime |
| chacha20poly1305 + blake3 | ✅ | 纯 Rust，无 C 依赖 |
| sea-orm + sqlx + sqlite | ✅ | SQLite 原生支持 |
| serde + serde_cbor | ✅ | 纯 Rust |
| ed25519-dalek | ✅ | 纯 Rust |
| dashmap | ✅ | 纯 Rust |

**无不可移植依赖**。

---

## 五、推荐重构策略

### 5.1 架构目标

```
swarmdrop/
├── libs/core/            ← 保持不变
├── libs/app-core/        ← 🆕 从 src-tauri 抽取的平台无关层
│   ├── src/
│   │   ├── lib.rs        ← #[uniffi::export] 所有 API
│   │   ├── network.rs    ← NetManager (去掉 AppHandle)
│   │   ├── pairing.rs    ← PairingManager
│   │   ├── transfer.rs   ← TransferManager
│   │   ├── crypto.rs     ← 加密模块
│   │   ├── database.rs   ← SeaORM ops
│   │   └── callback.rs   ← #[uniffi::export(callback_interface)]
│   └── Cargo.toml
├── src-tauri/             ← Tauri 桌面端 (薄层，调用 app-core)
└── swarmnote/             ← 🆕 RN 移动端 (uniffi-bindgen-react-native)
```

### 5.2 事件推送方案（替代 Tauri emit）

```rust
// Rust 端 - uniffi callback interface
#[uniffi::export(callback_interface)]
pub trait SwarmEventListener: Send + Sync {
    fn on_transfer_progress(&self, session_id: String, progress: TransferProgress);
    fn on_pairing_request(&self, peer_id: String, os_info: OsInfo);
    fn on_network_status_changed(&self, status: NetworkStatus);
    fn on_peer_discovered(&self, peer_id: String, name: String);
}

// RN/TS 端
const listener: SwarmEventListener = {
    onTransferProgress(sessionId, progress) { /* 更新 UI */ },
    onPairingRequest(peerId, osInfo) { /* 弹窗确认 */ },
};
await swarmCore.start(config, listener);
```

### 5.3 命令映射

```rust
// Tauri (现在)                              UniFFI (之后)
#[tauri::command]                            #[uniffi::export]
async fn start(app, keypair, ...)            async fn start(config, listener)

#[tauri::command]                            #[uniffi::export]
async fn generate_pairing_code(app)          async fn generate_pairing_code()
```

---

## 六、结论

**完全可行**。推荐方案：**uniffi-bindgen-react-native**。

1. **代码复用率高**: `libs/core` 完全不变，`src-tauri` 约 60% 业务逻辑可搬到 `app-core`
2. **无不可移植依赖**: libp2p、tokio、chacha20、sea-orm 全部支持移动端
3. **异步完美匹配**: swarmdrop 重度使用 tokio async，uniffi 原生支持 async fn → Promise
4. **事件系统有方案**: uniffi callback interface 替代 Tauri emit，类型安全
5. **DX 优于 Tauri**: 直接函数调用 + 编译时类型安全 + 无 JSON 序列化开销
