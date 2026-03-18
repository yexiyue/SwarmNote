# SwarmNote 技术选型文档

## 概述

本文档记录 SwarmNote 各技术领域的选型决策，包括调研过程、候选方案对比和最终选择的理由。

SwarmNote 是一个基于 Tauri v2 的去中心化 P2P 协作 Markdown 笔记应用，技术栈核心：**Rust 后端 + React 前端 + libp2p 网络 + yrs/yjs CRDT**。

---

## 1. 应用框架

**选型：Tauri v2**

不做对比，产品从立项起即确定使用 Tauri。选择理由：

- Rust 后端天然适配 libp2p 和 yrs
- 跨平台桌面端（Windows / macOS / Linux），后续可扩展移动端
- 相比 Electron 更轻量（安装包 ~10MB vs ~100MB+）
- 前端使用标准 Web 技术（React + TypeScript）

---

## 2. Markdown 编辑器

### 候选方案

| 方案 | 编辑模式 | yjs 集成质量 | Markdown 保真度 | React 支持 | 成熟度 |
|------|---------|-------------|----------------|-----------|--------|
| **CodeMirror 6** | 源码模式 | 一等（yjs 官方维护） | 完美 | 第三方 wrapper | 极高 |
| TipTap v3 | WYSIWYG | 官方扩展 | 好（MD 扩展较新） | 一等 | 极高 |
| Milkdown | WYSIWYG | 有同步稳定性问题 | 高 | 官方但简陋 | 中高 |
| BlockNote | Block（Notion 风格） | 内置 | **有损**（MD 非原生格式） | 最佳 | 中（pre-1.0） |

### 决策：CodeMirror 6 + y-codemirror.next

选择理由：

- **Markdown 保真度完美**：编辑原始 Markdown 源码，不存在格式转换损失。这是"去中心化 Obsidian"定位的核心要求
- **yjs 绑定由官方维护**：`y-codemirror.next` 是 yjs 团队的参考实现，绑定质量最高
- **Obsidian 同款引擎**：Obsidian 底层就是 CodeMirror 6，用户体验有标杆参照
- **P2P 已有验证**：md.uy 项目已使用 CodeMirror 6 + y-webrtc 实现 P2P Markdown 编辑
- **Awareness 内置**：通过 `yCollab` 扩展支持协作者光标和选区显示

不选 TipTap 的原因：产品定位是 Markdown 编辑器而非富文本编辑器。TipTap 的 Markdown 扩展（v3.7+）仍较新，round-trip 可能有 edge cases。部分协作功能需付费。

不选 Milkdown 的原因：社区报告协作同步稳定性问题（GitHub Discussion #1993），对 P2P 场景风险较高。

不选 BlockNote 的原因：Markdown 转换有损（`blocksToMarkdownLossy()`），原生格式是 JSON 而非 Markdown，与产品定位冲突。

### React 集成方案

CodeMirror 6 无官方 React wrapper，使用社区包 `@uiw/react-codemirror`（~1.7k GitHub stars），这是目前最流行的选择。

### 后续可能的增强

MVP 阶段为源码编辑模式。后续可考虑：

- 基于 CodeMirror 6 的 Decoration API 实现类似 Obsidian 的 "Live Preview" 模式（行内渲染标题、加粗等）
- 侧边预览面板

### 关注方向：Loro

Loro 是一个新兴的 Rust 原生 CRDT 库，性能优于 yjs，内置完整历史 DAG，有 CodeMirror 绑定（`loro-codemirror`），正在开发 Obsidian 插件 + E2E + P2P。目前生态尚不成熟（ProseMirror 绑定仅 ~130 stars），但与 SwarmNote 的技术方向高度吻合，值得持续关注。如果 Loro 生态在开发周期内成熟，可考虑迁移。

---

## 3. CRDT 引擎

### 候选方案

| 方案 | 语言 | 前端绑定 | 编辑器绑定 | 版本历史 | 成熟度 |
|------|------|---------|-----------|---------|--------|
| **yrs / yjs** | Rust / JS | 原生兼容 | 最丰富 | Snapshot + skip_gc | 最高 |
| Loro | Rust | WASM 绑定 | CodeMirror + ProseMirror | 内置 DAG | 早期 |
| automerge | Rust | @automerge/automerge | 有限 | 内置 | 中 |
| diamond-types | Rust | 无官方 | 无 | 无 | 低 |

### 决策：yrs（后端）+ yjs（前端）

选择理由：

- **前后端二进制编码完全兼容**：yrs 编码的 Update 和 yjs 可直接互通，无需序列化转换层
- **编辑器绑定最丰富**：y-codemirror.next 是首选，如需 fallback 还有 y-prosemirror、y-tiptap 等
- **生态最成熟**：已在 AppFlowy、AFFiNE 等生产项目中验证
- **Awareness 协议完整**：支持光标/选区/在线状态同步
- **UndoManager 支持**：支持 per-user undo（通过 transaction origin 区分本地和远程操作）
- **社区资源丰富**：文档、教程、论坛、参考实现充足

### 关键配置

```rust
use yrs::{Doc, Options};

let doc = Doc::with_options(Options {
    skip_gc: true,       // 必须：保留完整历史以支持版本快照
    client_id: unique_id, // 每个节点唯一 ID，用于 per-user undo
    ..Options::default()
});
```

### 前端依赖

```json
{
  "yjs": "^13.6",
  "y-codemirror.next": "^0.3.3",
  "lib0": "^0.2"
}
```

---

## 4. P2P 网络层

### 选型：libp2p（最新稳定版）

libp2p 是产品从立项起的确定选择。以下是各子协议的选型。

### 4.1 增量同步：GossipSub

| 方案 | 优点 | 缺点 |
|------|------|------|
| **GossipSub** | 高效扇出，mesh 机制，按 topic 订阅 | 不保证消息顺序和送达 |
| Floodsub | 实现简单 | 网络开销大 |

**决策：GossipSub**

- 按文档 ID 划分 topic，每个文档一个 topic
- CRDT 天然容忍乱序和重复消息，GossipSub 的弱保证完全足够
- 消息为 E2E 加密后的 yrs Update 二进制

### 4.2 全量同步：Request-Response

用于新节点加入或重连后的状态追赶。

同步流程：
```
节点 A（新加入）                     节点 B（已有文档）
    │                                    │
    │── SyncStep1(state_vector_a) ──────>│
    │                                    │
    │<── SyncStep2(encrypted_update) ────│
    │                                    │
    │── SyncStep2(encrypted_update) ────>│  (双向)
    │                                    │
    │═══ GossipSub 实时增量同步 ══════════│
```

### 4.3 Kademlia DHT

**决策：全功能 Kademlia DHT**

Kademlia DHT 是 SwarmNote 跨网络能力的核心基础设施，承担三个关键职责：

#### 文档发现（Provider Records）

当用户分享文档时，接收方通过 DHT 查找文档持有者：

```
分享流程：
1. Alice 创建文档 D，在 DHT 中发布 Provider Record：
   PUT_PROVIDER(key=hash(doc_id), provider=alice_peer_id)

2. Alice 将分享码发给 Bob（包含 doc_id + 加密密钥）

3. Bob 在 DHT 中查找文档提供者：
   GET_PROVIDERS(key=hash(doc_id)) → [alice_peer_id, ...]

4. Bob 连接到 Alice，通过 Request-Response 全量同步

5. Bob 也发布自己为 Provider：
   PUT_PROVIDER(key=hash(doc_id), provider=bob_peer_id)
```

- 每个持有文档的节点定期刷新 Provider Record（默认 24 小时过期）
- doc_id 做 hash 后作为 DHT key，不暴露原始文档 ID

#### 节点路由（Peer Routing）

通过 DHT 查找特定 PeerId 的网络地址，用于：

- 已知协作者的 PeerId 但不知道地址时，通过 DHT 查找
- 配合 Relay/DCUtR 实现 NAT 穿透

#### 去中心化引导

- 新节点连接到引导节点后，通过 DHT `FIND_NODE` 发现更多节点
- 不完全依赖硬编码引导节点列表，DHT 自身具备网络拓扑扩展能力

#### DHT 配置要点

```rust
use libp2p::kad::{self, store::MemoryStore};

let store = MemoryStore::new(local_peer_id);
let mut kad_config = kad::Config::default();
// 使用 Kademlia 作为 Server 模式（同时响应 DHT 查询）
kad_config.set_query_timeout(Duration::from_secs(60));

let kad = kad::Behaviour::with_config(local_peer_id, store, kad_config);
```

- **存储模式**：MemoryStore（桌面端内存足够，不需要持久化 DHT 数据）
- **Provider Record TTL**：默认 24h，节点在线时定期刷新
- **引导节点**：项目维护一组公共引导节点，同时允许用户自建

### 4.4 节点发现与连接

| 方案 | 场景 | 优先级 |
|------|------|--------|
| **mDNS** | 局域网自动发现 | MVP 必须 |
| **Kademlia DHT** | 跨网络文档发现、节点路由、去中心化引导 | MVP 必须 |
| **手动连接** | 输入节点地址 / 分享码 | MVP 必须 |
| **引导节点** | DHT 初始引导 + 跨网络连接入口 | MVP 必须 |
| Relay / DCUtR | NAT 穿透 | P1 |
| AutoNAT | NAT 检测 | P1 |

### Cargo.toml 依赖

```toml
libp2p = { version = "0.55", features = [
    "gossipsub",
    "request-response",
    "kad",
    "tcp",
    "noise",
    "yamux",
    "mdns",
    "relay",
    "dcutr",
    "autonat",
    "tokio",
    "identify",
] }
```

> 注：版本号以开发时的最新稳定版为准。原 swarmbook 选型为 0.54，实际开发时采用最新版。

---

## 5. E2E 加密

### 调研背景

P2P 场景下，E2E 加密的核心挑战是：
1. CRDT 更新需要加密后才能广播（中继节点不应读取内容）
2. 密钥分发没有中心服务器
3. 移除协作者后需要密钥轮换

参考架构：**SecSync**（Serenity Notes 使用的 E2E 加密 CRDT 方案，NLnet 资助）。

### 5.1 加密算法

| 算法 | Nonce 大小 | P2P 适用性 | 侧信道安全 | 纯 Rust |
|------|-----------|-----------|-----------|---------|
| **XChaCha20-Poly1305** | 24 字节 | 随机 nonce 安全 | 天然常数时间 | 是 |
| AES-256-GCM | 12 字节 | 随机 nonce 有碰撞风险 | 需硬件 AES-NI | 需 C 依赖 |

**决策：XChaCha20-Poly1305**

- **24 字节 nonce**：在 P2P 场景中无法协调 nonce 计数器，必须随机生成。XChaCha20 的 192-bit nonce 空间消除了碰撞风险
- **纯 Rust 实现**：跨平台编译无忧（Windows/macOS/Linux/Android/iOS）
- **无侧信道风险**：不依赖硬件加速，在任何设备上都是常数时间
- **SecSync 同款选择**：已在生产环境验证

### 5.2 加密对象

对每个 yrs Update 二进制整体加密后广播：

```
+--------+-------+---------+----------------+----------+
| doc_id | nonce | key_id  | ciphertext     | auth_tag |
| 32B    | 24B   | 4B      | variable       | 16B      |
+--------+-------+---------+----------------+----------+
```

- `doc_id` 明文：网络层需要据此路由消息
- `nonce` 随机生成：每条消息唯一
- `key_id` 支持密钥轮换：接收方查找对应解密密钥
- `ciphertext` 加密后的 yrs Update 二进制
- `auth_tag` AEAD 认证标签，防篡改

### 5.3 密钥管理

**密钥层级：**

```
用户主密钥（OS Keychain 保护）
  │
  ├── 用户身份密钥对
  │     ├── Ed25519（签名：验证更新来源）
  │     └── X25519（密钥交换：分发文档密钥）
  │
  ├── 文件夹密钥（256-bit 对称密钥，通过 Lockbox 分发）
  │     └── 文档密钥（HKDF 从文件夹密钥 + doc_id 派生）
  │
  └── 独立文档密钥（256-bit 对称密钥，单篇分享时使用）
```

**Lockbox 密钥分发：**

每个共享文档维护一组 Lockbox——文档密钥分别用每个协作者的 X25519 公钥加密：

```
文档元数据（不加密）：
{
  doc_id: "abc123",
  lockboxes: [
    { recipient_pubkey: "alice_pk", encrypted_doc_key: "..." },
    { recipient_pubkey: "bob_pk", encrypted_doc_key: "..." },
  ]
}
```

**密钥轮换（移除协作者时）：**

1. 生成新文档密钥
2. 用新密钥加密当前 CRDT 状态快照
3. 仅向剩余协作者分发新密钥的 Lockbox
4. 旧密钥加密的数据可清理

> 注意：P2P 系统无法阻止已移除用户保留之前已解密的数据。密钥轮换仅提供前向安全性。

### 5.4 Rust 依赖

```toml
# AEAD 加密
chacha20poly1305 = "0.10"

# 密钥交换
x25519-dalek = { version = "2", features = ["static_secrets"] }

# 签名
ed25519-dalek = { version = "2", features = ["rand_core"] }

# 密钥派生
hkdf = "0.12"
sha2 = "0.10"

# 安全随机数
rand = "0.8"
```

### 5.5 实现分阶段

| 阶段 | 内容 |
|------|------|
| Phase 1 | 每文档对称加密 yrs updates（加密后广播，接收后解密） |
| Phase 2 | Lockbox 密钥分发（X25519 密钥交换） |
| Phase 3 | 文件夹级密钥派生（HKDF） |
| Phase 4 | 密钥轮换（新快照 + 重新加密） |

---

## 6. 持久化

### 选型：SQLite + SeaORM 2.0

| 方案 | 优点 | 缺点 |
|------|------|------|
| **SQLite + SeaORM** | ORM 类型安全、Migration 管理、嵌入式、Tauri 常用 | FTS5 需 raw SQL |
| SQLite + rusqlite | 直接 SQL 控制、轻量 | 需手写 SQL |
| sled | Rust 原生 KV | 不再活跃维护 |
| redb | Rust 原生、无 unsafe | 无 FTS 支持、无查询能力 |

**决策：SeaORM 2.0 + SQLite**

选择理由：

- **类型安全**：Entity 模型直接映射 Rust struct，编译期检查
- **Migration 管理**：`sea-orm-migration` 提供版本化迁移，应用启动时自动执行
- **无需手写 SQL**：通过 SeaORM 的 Query Builder 进行 CRUD 操作
- **FTS5 兼容**：通过 `execute_unprepared` 和 `raw_sql!` 宏执行原始 SQL，FTS5 虚拟表与 ORM 管理的表共存
- **BLOB 支持**：`Vec<u8>` 直接映射 SQLite BLOB，适合存储 yrs 二进制数据
- **Bundled SQLite**：`sqlx-sqlite` 自动编译内嵌 SQLite，无需系统安装

SeaORM 2.0 于 2026 年 1 月 12 日发布（RC 阶段，API 已稳定），要求 Rust 2024 edition / MSRV 1.85 / tokio runtime。

### Entity 模型

```rust
// entity/document.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub doc_id: String,
    pub title: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub current_state: Vec<u8>,        // 合并压缩后的 yrs 状态
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub state_vector: Vec<u8>,         // 用于同步的 StateVector
    pub created_at: i64,
    pub updated_at: i64,
}

// entity/update.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "updates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub doc_id: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub update_data: Vec<u8>,          // 单个 yrs Update
    pub client_id: Option<i64>,
    pub created_at: i64,
}

// entity/snapshot.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snapshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub doc_id: String,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub snapshot_data: Vec<u8>,        // 编码的 Snapshot（StateVector + DeleteSet）
    pub label: Option<String>,         // "手动保存"、"关闭文档"、"同步完成" 等
    pub created_at: i64,
}

// entity/identity.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "identity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,                       // 固定为 1（单行表）
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub public_key: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub private_key: Vec<u8>,          // 加密存储
    pub nickname: String,
    pub color: String,
}

// entity/document_key.rs
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "document_keys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub doc_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub key_id: i32,
    #[sea_orm(column_type = "VarBinary(StringLen::None)")]
    pub key_data: Vec<u8>,             // 对称密钥（Lockbox 解密后）
    pub created_at: i64,
}
```

### FTS5 虚拟表（Raw SQL）

SeaORM 不管理 FTS5 虚拟表，通过 raw SQL 创建和查询：

```rust
// 应用启动时创建 FTS5 表
db.execute_unprepared(
    "CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
        title, body, content=documents, content_rowid=rowid, tokenize='unicode61'
    )"
).await?;

// 搜索
#[derive(FromQueryResult)]
struct SearchResult {
    doc_id: String,
    title: String,
    snippet: String,
    rank: f64,
}

let results = SearchResult::find_by_statement(raw_sql!(
    Sqlite,
    r#"SELECT d.doc_id, d.title,
              snippet(documents_fts, 1, '<mark>', '</mark>', '...', 32) as snippet,
              bm25(documents_fts, 10.0, 1.0) as rank
       FROM documents_fts
       JOIN documents d ON d.rowid = documents_fts.rowid
       WHERE documents_fts MATCH {query}
       ORDER BY rank
       LIMIT 20"#
))
.all(&db)
.await?;
```

### 数据库初始化

```rust
use sea_orm::Database;
use migration::{Migrator, MigratorTrait};

// 使用 Tauri 的 app_data_dir 存储数据库（避免开发模式下触发文件监听重启）
let app_dir = app_handle.path().app_data_dir().unwrap();
std::fs::create_dir_all(&app_dir).unwrap();
let db_path = app_dir.join("swarmnote.db");
let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

let db = Database::connect(&db_url).await?;
Migrator::up(&db, None).await?;  // 自动执行迁移
```

### Cargo.toml

```toml
sea-orm = { version = "~2.0.0-rc", features = [
    "sqlx-sqlite",
    "runtime-tokio-native-tls",
    "macros",
], default-features = false }

# 确保 FTS5 支持
libsqlite3-sys = { version = "0.30", features = ["bundled", "fts5"] }
```

Migration crate 单独一个子 crate：

```toml
# migration/Cargo.toml
[dependencies]
sea-orm-migration = { version = "~2.0.0-rc", features = [
    "runtime-tokio-native-tls",
    "sqlx-sqlite",
] }
```

---

## 7. 全文搜索

### 候选方案

| 方案 | 集成复杂度 | 中文支持 | 搜索质量 | 内存 | CRDT 友好 |
|------|----------|---------|---------|------|----------|
| **SQLite FTS5 + jieba** | 极低 | jieba-rs 预分词 | BM25 | ~0 额外 | 极好 |
| tantivy + jieba | 中 | tantivy-jieba | BM25 + 模糊 | ~30MB | 好 |
| MeiliSearch | 高（独立进程） | 内置 CJK | 好 | ~100MB+ | 差 |

### 决策：SQLite FTS5 + jieba-rs 预分词

选择理由：

- **零额外基础设施**：FTS 索引与文档数据在同一个 SQLite 数据库中
- **CRDT 同步友好**：文档变更和索引更新在同一事务中完成，保证一致性
- **中文支持**：通过 jieba-rs 在写入时预分词，搜索时同样分词后匹配
- **内存零额外开销**：无需独立进程或额外索引文件
- **升级路径**：如后续需要模糊搜索等高级功能，可加入 tantivy 而不重构

### 中文分词方案

```rust
use jieba_rs::Jieba;
use std::sync::LazyLock;

static JIEBA: LazyLock<Jieba> = LazyLock::new(|| Jieba::new());

fn segment_for_fts(text: &str) -> String {
    JIEBA.cut(text, false).join(" ")
}

// 写入时预分词，通过 SeaORM 的 raw SQL 更新 FTS 索引
let segmented_title = segment_for_fts(&doc.title);
let segmented_body = segment_for_fts(&doc.body);
db.execute_unprepared(&format!(
    "INSERT INTO documents_fts(rowid, title, body) VALUES ({}, '{}', '{}')",
    doc_rowid, segmented_title, segmented_body
)).await?;

// 搜索时同样分词
let segmented_query = segment_for_fts(&user_input);
let results = SearchResult::find_by_statement(raw_sql!(
    Sqlite,
    r#"SELECT d.doc_id, highlight(documents_fts, 1, '<mark>', '</mark>') as snippet
       FROM documents_fts
       JOIN documents d ON d.rowid = documents_fts.rowid
       WHERE documents_fts MATCH {segmented_query}
       ORDER BY rank"#
))
.all(&db)
.await?;
```

### Cargo.toml

```toml
jieba-rs = "0.7"
```

---

## 8. 版本历史

### 机制

yrs 的 `Snapshot` 是轻量级逻辑时间标记（StateVector + DeleteSet），仅几十字节。配合 `skip_gc: true` 的文档 block store 可重建任意历史状态。

### 快照策略：事件驱动

在有意义的时间点保存快照：

| 事件 | 快照标签 |
|------|---------|
| 用户手动保存（Ctrl+S） | "手动保存" |
| 关闭文档 | "关闭文档" |
| 同步完成（收到远程更新后） | "同步完成" |
| 应用退出 | "应用退出" |

### 存储策略：混合模式

1. **updates 表**：追加存储每个增量 update（写前日志）
2. **snapshots 表**：事件触发的轻量标记
3. **documents.current_state**：定期合并压缩所有 updates（快速加载路径）

### 历史浏览

1. 重建过去状态：`txn.encode_state_from_snapshot(&snapshot)`
2. 显示差异：将两个状态重建为文本，用 `similar` crate 计算 diff
3. 回退：从历史快照创建新的 update 覆盖当前状态

### 清理策略

- 近 1 小时：保留全部 updates
- 近 24 小时：每 10 分钟合并一次
- 近 7 天：每小时合并一次
- 更早：仅保留命名快照

### Cargo.toml

```toml
similar = "2"   # 文本 diff
```

---

## 9. 前端技术栈

### UI 框架

| 选型 | 说明 |
|------|------|
| **React + TypeScript** | 应用框架 |
| **shadcn/ui + Tailwind CSS** | 组件库 + 样式 |
| **Zustand** | 状态管理 |
| **Vite** | 构建工具（已有） |

### 选择理由

**shadcn/ui + Tailwind**：可定制性强，组件代码直接拷贝到项目中（非 node_modules 依赖），Tauri 社区常用组合。

**Zustand**：轻量（< 1KB），类型安全，API 简洁，适合 SwarmNote 这种中等复杂度的应用。不需要 Redux 的中间件生态，也不需要 Jotai 的原子化粒度。

### package.json 依赖

```json
{
  "dependencies": {
    "react": "^19",
    "react-dom": "^19",
    "yjs": "^13.6",
    "y-codemirror.next": "^0.3.3",
    "lib0": "^0.2",
    "@uiw/react-codemirror": "^4.23",
    "@codemirror/lang-markdown": "^6",
    "@codemirror/language-data": "^6",
    "zustand": "^5",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2"
  },
  "devDependencies": {
    "typescript": "^5.6",
    "tailwindcss": "^4",
    "vite": "^6",
    "@vitejs/plugin-react": "^4"
  }
}
```

---

## 10. 前后端通信（Tauri IPC）

### 数据流

```
┌──────────────────────────────────────┐
│           React 前端                  │
│                                      │
│  CodeMirror 6 (Markdown 编辑)        │
│    ↕ y-codemirror.next               │
│  Y.Doc (yjs)                         │
│    ↕ Uint8Array (yjs binary)         │
│  Tauri invoke / events               │
└──────────┬───────────────────────────┘
           │ IPC: binary (Uint8Array)
┌──────────▼───────────────────────────┐
│         Tauri Rust 后端               │
│                                      │
│  yrs::Doc ←→ apply/encode update     │
│       ↕                              │
│  E2E 加密层 (XChaCha20-Poly1305)     │
│       ↕                              │
│  libp2p Swarm                        │
│    ├─ GossipSub (加密增量广播)        │
│    ├─ Request-Response (加密全量同步) │
│    ├─ mDNS (局域网发现)               │
│    └─ Relay/DCUtR (NAT 穿透)         │
│       ↕                              │
│  SQLite + SeaORM (持久化 + FTS5)    │
└──────────────────────────────────────┘
```

### 通信方式

- **前端 → 后端**：`invoke` 命令，传递 yjs 编码的 `Uint8Array`
- **后端 → 前端**：Tauri `Event` 推送远程节点的 Update
- **编码格式**：yjs/yrs 的 v1 二进制编码，零序列化开销

---

## 11. 完整依赖清单

### Rust (Cargo.toml)

```toml
[dependencies]
# 应用框架
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CRDT
yrs = "0.21"

# P2P 网络
libp2p = { version = "0.55", features = [
    "gossipsub",
    "request-response",
    "kad",
    "tcp",
    "noise",
    "yamux",
    "mdns",
    "relay",
    "dcutr",
    "autonat",
    "identify",
    "tokio",
] }

# 异步运行时
tokio = { version = "1", features = ["full"] }

# E2E 加密
chacha20poly1305 = "0.10"
x25519-dalek = { version = "2", features = ["static_secrets"] }
ed25519-dalek = { version = "2", features = ["rand_core"] }
hkdf = "0.12"
sha2 = "0.10"
rand = "0.8"

# 持久化（ORM + SQLite）
sea-orm = { version = "~2.0.0-rc", features = [
    "sqlx-sqlite",
    "runtime-tokio-native-tls",
    "macros",
], default-features = false }
libsqlite3-sys = { version = "0.30", features = ["bundled", "fts5"] }

# 全文搜索分词
jieba-rs = "0.7"

# 文本 diff（版本历史）
similar = "2"
```

Migration 子 crate：

```toml
# migration/Cargo.toml
[dependencies]
sea-orm-migration = { version = "~2.0.0-rc", features = [
    "runtime-tokio-native-tls",
    "sqlx-sqlite",
] }
```

> 注：版本号以开发时 crates.io 最新稳定版为准。

### 前端 (package.json)

```json
{
  "dependencies": {
    "react": "^19",
    "react-dom": "^19",
    "yjs": "^13.6",
    "y-codemirror.next": "^0.3.3",
    "lib0": "^0.2",
    "@uiw/react-codemirror": "^4.23",
    "@codemirror/lang-markdown": "^6",
    "@codemirror/language-data": "^6",
    "zustand": "^5",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2"
  },
  "devDependencies": {
    "typescript": "^5.6",
    "tailwindcss": "^4",
    "vite": "^6",
    "@vitejs/plugin-react": "^4"
  }
}
```

---

## 12. 风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| CodeMirror 6 无 WYSIWYG | 用户体验不如 Notion 直观 | 后续实现 Live Preview（CodeMirror Decoration API）；源码模式本身是 Obsidian 验证过的体验 |
| y-codemirror.next 仍为 v0.x | API 可能变动 | 锁定版本；binding 实际已稳定，风险低 |
| yrs 与 yjs 版本不兼容 | 前后端编码不互通 | 锁定兼容版本对，写集成测试验证 |
| E2E 加密增加同步复杂度 | 开发周期拉长 | 分阶段实现；Phase 1 仅做对称加密，不做密钥管理 |
| jieba-rs 分词字典内存开销 ~15MB | 启动内存增加 | 桌面端可接受；移动端可考虑懒加载或更轻量的分词器 |
| 大文档 skip_gc 导致内存增长 | 重度编辑文档内存占用高 | 定期压缩合并；设置告警阈值；极端情况下允许丢弃历史重建文档 |
| GossipSub 消息丢失 | 编辑内容未同步 | CRDT 容忍重复/乱序；定期全量同步兜底；本地持久化保底 |
| NAT 穿透失败 | 跨网络节点无法互连 | 提供引导节点 + Relay 兜底 |
| SQLite FTS5 无模糊搜索 | 搜索体验不如专业搜索引擎 | MVP 够用；后续可加 tantivy 作为高级搜索 |

---

## 13. 参考资料

### 编辑器

- [CodeMirror 6 文档](https://codemirror.net/)
- [y-codemirror.next GitHub](https://github.com/yjs/y-codemirror.next)
- [@uiw/react-codemirror](https://github.com/uiwjs/react-codemirror)
- [md.uy - P2P Markdown 编辑器实现](https://mr19.xyz/blog/md-uy/)

### CRDT

- [yrs 文档](https://docs.rs/yrs)
- [yjs 文档](https://docs.yjs.dev)
- [y-crdt GitHub](https://github.com/y-crdt/y-crdt)
- [Yrs 架构深度解析 - Bartosz Sypytkowski](https://www.bartoszsypytkowski.com/yrs-architecture/)
- [Loro GitHub](https://github.com/loro-dev/loro)（关注方向）

### E2E 加密

- [SecSync - E2E 加密 CRDT 架构](https://www.secsync.com/)
- [Serenity Notes](https://github.com/serenity-kit/serenity-notes-clients)
- [Tag1 Deep Dive: E2E Encryption in Yjs (Part 1)](https://www.tag1consulting.com/blog/deep-dive-end-end-encryption-e2ee-yjs)
- [Tag1 Deep Dive: E2E Encryption in Yjs (Part 2)](https://www.tag1consulting.com/blog/deep-dive-end-end-encryption-e2ee-yjs-part-2)
- [Kleppmann: Decentralized Key Agreement for CRDTs](https://martin.kleppmann.com/2021/11/17/decentralized-key-agreement.html)
- [Kerkour: CRDT + E2EE Research Notes](https://kerkour.com/crdt-end-to-end-encryption-research-notes)

### 搜索

- [SQLite FTS5 文档](https://www.sqlite.org/fts5.html)
- [jieba-rs GitHub](https://github.com/messense/jieba-rs)

### P2P 网络

- [libp2p 文档](https://docs.libp2p.io/)
- [GossipSub 规范](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/README.md)

### 架构参考

- [Ink & Switch: Local-First Software](https://www.inkandswitch.com/essay/local-first/)
- [Anytype any-sync 协议](https://tech.anytype.io/any-sync/overview)
