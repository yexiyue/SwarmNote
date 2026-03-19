# 三级同步架构

根据数据类型选择不同的同步策略：

| 级别 | 适用场景 | 同步方式 | 粒度 |
|------|---------|---------|------|
| **L1: yjs 实时协作** | 两端同时在应用内编辑同一文档 | 字符级 CRDT 操作同步 | 字符 |
| **L2: FastCDC 分块同步** | .md 文件变更（应用内保存或外部编辑） | 按内容边界分块，增量传输变化的 chunk | 块（~4KB） |
| **L3: 全量同步** | 图片/视频/附件等资源文件 | 比较 mtime + blake3 hash，整文件传输 | 文件 |

## L1: yjs 实时协作

两端都在 BlockNote 中打开同一文档时自动启用：

```
A 和 B 都在应用内打开了"学习笔记.md"
→ 通过 P2P 检测到双方打开同一文档
→ 升级为 yjs 实时同步模式（字符级 CRDT）
→ 任一方关闭文档 → 保存 .md → 退回 L2 文件级同步
```

- yjs state 存在 workspace.db 中，用于实时协作期间的增量同步
- 关闭文档时，yjs doc 导出为 .md（`blocksToMarkdownLossy()`）

## L2: FastCDC 分块同步（.md 文件）

用于非实时场景下的 .md 文件同步，包括外部编辑器修改的情况：

```
检测到 .md 文件变更（fs watcher / notify）：
1. FastCDC 按内容边界分块（平均 ~4KB，最小 2KB，最大 8KB）
2. 每个 chunk 计算 blake3 hash
3. 发送 ChunkManifest { doc_path, chunks: [(offset, length, hash), ...] }
4. 对端对比本地 chunk hash → 找出差异 chunk
5. 请求变化的 chunk → 收到后重组文件
```

FastCDC（Content-Defined Chunking）的优势：

- 插入/删除文本时，只有受影响的 chunk 边界移动，大部分 chunk hash 不变
- 比固定大小分块的增量传输效率高得多
- blake3 计算速度极快（~3x BLAKE2, ~6x SHA-256），适合频繁检测变更

```rust
struct ChunkManifest {
    doc_path: String,           // 相对路径
    file_hash: [u8; 32],       // 整文件 blake3 hash（快速判断是否有变化）
    chunks: Vec<ChunkMeta>,
    last_modified: i64,
}

struct ChunkMeta {
    offset: u64,
    length: u32,
    hash: [u8; 32],             // blake3
}
```

### 冲突处理

两端都修改了同一 .md 文件时：

```
检测到冲突（两端 file_hash 都与 base 不同）：
1. 按 chunk 做三方对比（base chunk list + A chunks + B chunks）
2. 只有一端修改的 chunk → 取修改方
3. 两端都修改了同一 chunk → 标记冲突区域
4. 生成冲突副本：学习笔记.conflict-<timestamp>.md
5. 通知用户手动解决
```

### 外部编辑检测

```
Rust 后端使用 notify crate 监听工作区目录：
1. 检测到 .md 文件 mtime 变化
2. 计算新的 blake3 hash → 与 workspace.db 中记录的 hash 对比
3. hash 不同 → 文件被修改
4. 重新 FastCDC 分块 → 更新 chunk 索引 → 触发同步
5. 如果应用内正在显示该文档 → 重新加载到 BlockNote
```

## L3: 全量同步（资源文件）

图片/视频/附件等二进制资源不需要分块，整文件比较和传输：

```
同步流程：
1. 扫描资源目录，收集文件列表 + mtime + blake3 hash
2. 发送 AssetManifest { doc_path, assets: [(filename, hash, size, mtime), ...] }
3. 对端对比：
   - 本地没有 → 拉取整个文件
   - hash 相同 → 跳过
   - hash 不同 → 取 mtime 更新的版本（资源文件一般不会两端同时修改）
4. 大文件传输复用 SwarmDrop 的分块传输方案（传输层分块，非存储层）
```

```rust
struct AssetManifest {
    doc_path: String,               // 所属文档的相对路径
    assets: Vec<AssetMeta>,
}

struct AssetMeta {
    filename: String,               // 文件名
    hash: [u8; 32],                 // blake3
    size: u64,                      // 字节数
    mtime: i64,                     // 最后修改时间
    mime_type: String,              // image/png, video/mp4 等
}
```

- Phase 1 先支持**图片**，视频/音频/附件后续扩展
- 资源文件同步策略：**按需拉取**（打开文档时才请求缺失的资源），不做全量预同步
- 单文件大小建议限制（如 100MB），超大文件提示用户

## 三级同步的状态转换

```
                    ┌─────────────────────────┐
                    │  L3: 全量同步（资源文件） │
                    └─────────────────────────┘
                              ↑ 资源文件变更
                              │
┌─────────────┐  双方打开同一文档  ┌──────────────────────────┐
│ L2: FastCDC │ ←──────────────→ │ L1: yjs 实时协作          │
│ 分块同步    │  任一方关闭文档   │ （两端都在应用内编辑时）    │
└─────────────┘                  └──────────────────────────┘
      ↑
      │ .md 文件变更（应用内保存 / 外部编辑）
      │ fs watcher 检测
```

## 同步协议中的权限校验

```
收到同步请求(peer_id, resource_id):
  1. 解析资源 → 找到所属 workspace
  2. 查询 peer_id 的有效权限（直接授权 > 文件夹继承 > 工作区继承）
  3. 无权限 → 拒绝，返回 AccessDenied
  4. Reader → 发送文档内容（用 read_key 加密），拒绝对方的 CRDT update
  5. Editor/Owner → 双向同步
```

## 同步消息类型

```rust
enum SyncMessage {
    // 文档同步（L1 yjs / L2 FastCDC）
    DocUpdate { doc_id: Uuid, update: Vec<u8>, signature: Vec<u8> },
    DocSyncRequest { doc_id: Uuid, state_vector: Vec<u8> },
    DocSyncResponse { doc_id: Uuid, update: Vec<u8> },
    ChunkManifest { doc_path: String, file_hash: [u8; 32], chunks: Vec<ChunkMeta> },
    ChunkRequest { doc_path: String, chunk_hashes: Vec<[u8; 32]> },
    ChunkResponse { doc_path: String, chunks: Vec<(u64, Vec<u8>)> },

    // 资源同步（L3）
    AssetManifest { doc_path: String, assets: Vec<AssetMeta> },
    AssetRequest { doc_path: String, filename: String },
    AssetResponse { doc_path: String, filename: String, data: Vec<u8> },

    // 权限同步（仅 Owner 可发起，需 admin_key 签名）
    PermissionGranted { resource_type: ResourceType, resource_id: Uuid, peer_id: PeerId, role: Role, signature: Vec<u8> },
    PermissionRevoked { resource_type: ResourceType, resource_id: Uuid, peer_id: PeerId, signature: Vec<u8> },
    OwnerTransferred { resource_type: ResourceType, resource_id: Uuid, new_owner: PeerId, signature: Vec<u8> },
    KeyRotation { workspace_id: Uuid, key_version: u32, encrypted_keys: Vec<u8>, signature: Vec<u8> },
}
```
