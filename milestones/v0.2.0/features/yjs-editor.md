# 编辑器 yjs 集成

## 用户故事

作为用户，我希望编辑器支持协作编辑能力，为后续 P2P 同步提供 CRDT 数据基础。

## 依赖

- 无依赖（L0，可独立开始）
- 基于 v0.1.0 已有的 BlockNote 编辑器改造

## 需求描述

将 v0.1.0 的纯 BlockNote 编辑器升级为 BlockNote + yjs 协作模式。编辑操作通过 yjs Y.Doc 管理状态，yjs updates 通过 Tauri IPC 传递给 Rust 端存储和转发。

存储模型为「MD 主 + yjs 同步层」：
1. 打开笔记：读 .md 文件 → BlockNote blocks → 初始化 Y.Doc
2. 编辑时：BlockNote 通过 yjs 协作层管理状态
3. 保存时：Y.Doc → BlockNote blocks → 导出 .md 写回磁盘
4. yjs state 在 SQLite 中持久化（用于全量同步的 state_vector 交换）

## 技术方案

### 前端

- 安装 yjs 相关依赖：`yjs`, `@blocknote/core`（协作支持内置）
- 每个文档创建独立 `Y.Doc`，使用 `XmlFragment` 作为 BlockNote 协作 fragment
- yjs update 事件 → `invoke('save_yjs_update', { docId, update })` 发送给 Rust
- 文档打开时从 Rust 获取 yjs state → 应用到 Y.Doc

### 后端

- 新增 Tauri commands：
  - `save_yjs_update(doc_id, update: Vec<u8>)` — 存储 yjs 增量更新
  - `get_yjs_state(doc_id) -> Vec<u8>` — 返回文档的 yjs 完整状态
  - `apply_remote_update(doc_id, update: Vec<u8>)` — 应用远端更新
- SQLite 新增 `yjs_states` 表：`doc_id, state_blob, state_vector, updated_at`
- yjs updates 追加写入，定期合并压缩

## 验收标准

- [ ] BlockNote 编辑器以 yjs 协作模式运行
- [ ] 编辑产生的 yjs updates 正确传递给 Rust 端存储
- [ ] 重新打开文档后，yjs state 正确恢复
- [ ] MD 文件仍正常保存和读取，与 yjs state 保持一致
- [ ] 编辑体验无明显延迟（< 16ms 输入响应）

## 开放问题

- BlockNote yjs collaboration 的具体 API 需查阅最新文档
- yjs state 压缩策略：多少条 updates 后触发一次 merge
- MD ↔ yjs 转换是否会导致格式漂移，需实测验证
