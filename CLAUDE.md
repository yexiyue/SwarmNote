# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SwarmNote is a decentralized, local-first, peer-to-peer note-taking app built with Tauri v2 + React 19 + Rust. Notes sync between devices via P2P networking (libp2p) without a central server. Targets desktop (Windows/macOS/Linux) and Android.

## Development Commands

```bash
# Launch full Tauri desktop app (starts frontend + Rust backend)
pnpm tauri dev

# Frontend dev server only (Vite on port 1420)
pnpm dev

# Build frontend (TypeScript compile + Vite build)
pnpm build

# Build Tauri app for distribution
pnpm tauri build

# Frontend lint (Biome check)
pnpm lint

# Frontend lint CI mode (no auto-fix, exits non-zero on errors)
pnpm lint:ci

# Frontend format (Biome auto-fix)
pnpm format

# Rust format + lint
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy -- -D warnings

# Rust backend tests
cd src-tauri && cargo test

# Generate CHANGELOG from conventional commits
pnpm changelog

# Show unreleased changes
pnpm changelog:latest
```

No test framework is configured for the frontend yet.

## Code Quality Toolchain

- **Biome** (`biome.json`): 前端 lint + format（替代 ESLint + Prettier），`recommended` 规则集，自动 organize imports，行宽 100
- **rustfmt** (`src-tauri/rustfmt.toml`): Rust 代码格式化
- **Clippy**: Rust 静态分析，CI 中以 `-D warnings` 运行
- **Lefthook** (`lefthook.yml`): Git hooks 管理
  - `pre-commit`: Biome check（前端）+ cargo fmt --check（Rust），并行执行
  - `commit-msg`: commitlint 校验 Conventional Commits 格式
- **commitlint** (`commitlint.config.js`): 提交信息必须遵循 [Conventional Commits](https://www.conventionalcommits.org/)（`feat:`, `fix:`, `docs:`, `chore:` 等）
- **git-cliff** (`cliff.toml`): 基于 Conventional Commits 自动生成 CHANGELOG
- **GitHub Actions** (`.github/workflows/ci.yml`): PR/push 到 main/develop 时自动跑 lint + build + test

## Architecture

```text
swarmnote/
├── src/                  # React + TypeScript frontend
│   ├── main.tsx          # Entry point (React.StrictMode)
│   └── App.tsx           # Main component, Tauri invoke example
├── src-tauri/            # Rust backend (Tauri v2)
│   ├── src/lib.rs        # Tauri commands + app builder
│   ├── src/main.rs       # Desktop entry point
│   ├── capabilities/     # Tauri v2 security capability declarations
│   └── tauri.conf.json   # Tauri config (window, build, CSP)
├── docs/                 # Astro + Starlight documentation site
├── openspec/             # Spec-driven change management (OpenSpec)
└── dev-notes/            # Planning docs, PRD, tech selection
```

### Frontend-Backend Bridge

Frontend calls Rust via `invoke()` from `@tauri-apps/api/core`. Rust exposes functions with `#[tauri::command]` in `src-tauri/src/lib.rs`, registered via `tauri::generate_handler![]`. Capabilities must be declared in `src-tauri/capabilities/` for Tauri v2 security.

### Rust Library Naming

The Rust lib is named `swarmnote_lib` (not `swarmnote`) to avoid a Windows naming conflict between the lib and bin targets.

## Code Conventions

- **TypeScript**: strict mode, `noUnusedLocals`/`noUnusedParameters` enforced, ESNext modules, `react-jsx` transform
- **React**: functional components with hooks, PascalCase filenames
- **Rust**: standard rustfmt, `#[tauri::command]` pattern, snake_case
- **Package manager**: pnpm (not npm/yarn)
- **Git flow**: `main` → `develop` → `feature/*` branches, PRs required
- **Formatter**: Biome indent 2 spaces, line width 100

## Key Config Files

- `vite.config.ts` — dev server fixed to port 1420 (required by Tauri), excludes `src-tauri/` from watch
- `src-tauri/tauri.conf.json` — runs `pnpm dev` before dev, `pnpm build` before build, frontend dist at `../dist`
- `tsconfig.json` — target ES2020, bundler module resolution
- `biome.json` — Biome lint + format config, scoped to `src/` and root config files
- `lefthook.yml` — pre-commit and commit-msg hooks
- `cliff.toml` — git-cliff changelog generation config

## Sister Project: SwarmDrop & swarm-p2p-core

SwarmDrop 是同作者的去中心化文件传输工具，已验证完整 libp2p 链路（v0.4.4）。其核心 P2P 网络层已抽离为 `swarm-p2p-core` 库（`swarmdrop/libs/core/`，git submodule）。

### swarm-p2p-core 公开 API

```rust
// 启动节点，返回命令发送端 + 事件接收端
let (client, receiver) = swarm_p2p_core::start::<AppRequest, AppResponse>(keypair, config).await;

// NetClient<Req, Resp> — 发送网络命令
client.dial(peer_id)                    // 连接节点
client.send_request(peer_id, req)       // 发送请求
client.send_response(pending_id, resp)  // 回复请求
client.bootstrap()                      // 加入 DHT
client.start_provide(key) / get_providers(key)  // DHT provider
client.put_record(record) / get_record(key)     // DHT 记录

// EventReceiver<Req> — 接收网络事件
receiver.recv() -> NodeEvent<Req>
// 事件类型: PeersDiscovered, PeerConnected, InboundRequest, NatStatusChanged, HolePunchSucceeded...
```

### swarm-p2p-core 内置能力

| 层面     | 能力                                          |
| -------- | --------------------------------------------- |
| 传输     | TCP + QUIC + Noise 加密 + Yamux 多路复用      |
| 发现     | mDNS（局域网）+ Kademlia DHT（跨网络）        |
| NAT 穿透 | AutoNAT v2 + DCUtR 打洞 + Relay 中继          |
| 协议     | Request-Response + CBOR，泛型 `<Req, Resp>`   |

### SwarmNote 复用策略

SwarmNote Phase 1 应直接引入 `swarm-p2p-core` 作为 git submodule，只需：

1. 定义 SwarmNote 自己的 `AppRequest` / `AppResponse`（笔记同步协议）
2. 复用 `NetClient` / `EventReceiver` / `NodeConfig` 架构
3. 参考 SwarmDrop 的 `network/event_loop.rs` 模式桥接到 Tauri 事件

SwarmDrop 已验证的模块可直接参考：密钥管理（Stronghold）、设备配对（DHT + 6位码）、分块传输、XChaCha20-Poly1305 加密。

## Planned Architecture (Not Yet Implemented)

Development is phased (see `dev-notes/`):

1. **Phase 1**: P2P networking via swarm-p2p-core, SQLite local storage, device discovery
2. **Phase 2**: Editor with CodeMirror, CRDT sync via yrs/yjs
3. **Phase 3**: E2E encryption, key distribution, secure document sharing
