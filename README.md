# SwarmNote

**P2P 笔记同步工具** —— 设备间免费同步，离线自动合并，数据完全本地。

无需服务器、无需付费、无需折腾云盘。打开 SwarmNote，你的笔记在设备间自动流转。

## 特性

- **P2P 同步**：基于 libp2p，设备间直接通信，不经过任何第三方服务器
- **CRDT 自动合并**：基于 yrs/yjs，离线编辑后重连零冲突合并，不会丢失任何内容
- **本地优先**：数据完全存储在本地 SQLite，网络断开也能正常使用
- **局域网自动发现**：mDNS 自动发现同一网络下的设备，零配置
- **跨平台**：基于 Tauri v2，支持 Windows / macOS / Linux（移动端规划中）
- **开源免费**：MIT 协议

## 技术栈

| 层 | 技术 |
|----|------|
| 前端 | React 19 + TypeScript + CodeMirror 6 + Tailwind CSS |
| 后端 | Rust + Tauri v2 |
| CRDT | yrs (Rust) + yjs (JS) |
| P2P 网络 | [swarm-p2p-core](https://github.com/yexiyue/swarm-p2p) (libp2p) |
| 存储 | SQLite (rusqlite) |

## Swarm 生态

SwarmNote 是 Swarm 系列开源项目之一：

| 项目 | 说明 | 状态 |
|------|------|------|
| [SwarmDrop](https://github.com/yexiyue/swarmdrop) | P2P 文件传输 | v0.4.4 |
| **SwarmNote** | P2P 笔记同步 | 开发中 |
| [swarm-p2p-core](https://github.com/yexiyue/swarm-p2p) | P2P 网络 SDK | 已完成 |

所有项目共享 swarm-p2p-core 网络层。

## 开发

```bash
# 安装依赖
pnpm install

# 启动开发（前端 + Rust 后端）
pnpm tauri dev

# 构建发布
pnpm tauri build

# 前端 lint
pnpm lint

# Rust lint
cd src-tauri && cargo clippy -- -D warnings
```

## 路线图

- [x] 项目初始化 + 工具链配置
- [ ] swarm-p2p-core 集成 GossipSub
- [ ] MVP：局域网 P2P 笔记同步
- [ ] 实时协作编辑
- [ ] 跨网络同步 + NAT 穿透
- [ ] E2E 加密
- [ ] 一键发布博客（GitHub Pages）

## 协议

MIT
