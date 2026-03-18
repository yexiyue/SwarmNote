# Phase 3：端到端加密 + 身份认证

**目标**：实现用户身份体系和文档级 E2E 加密，使得只有被授权的用户才能读取文档内容，中继节点和引导节点无法解密任何数据。

**前置依赖**：Phase 1（网络层）+ Phase 2（文档同步）已完成。

**完成标志**：文档内容在传输和存储层面均加密，只有持有文档密钥的协作者才能解密。用户通过分享码邀请协作者，被移除的协作者无法读取后续内容。

---

## 3.1 用户身份

**目标**：每个 SwarmNote 节点拥有唯一的加密身份。

### 密钥体系

```
用户主密钥（OS Keychain 保护）
  │
  ├── Ed25519 签名密钥对
  │     └── 用途：验证消息来源、签名文档更新
  │
  └── X25519 密钥交换密钥对
        └── 用途：与其他用户安全交换文档对称密钥
```

### 任务

- [ ] 定义 `identity` 表 Entity：
  ```rust
  pub struct Model {
      pub id: i32,                   // 固定为 1（单行表）
      pub ed25519_public: Vec<u8>,
      pub ed25519_private: Vec<u8>,  // 加密存储
      pub x25519_public: Vec<u8>,
      pub x25519_private: Vec<u8>,   // 加密存储
      pub nickname: String,
      pub color: String,             // 头像/光标颜色（hex）
  }
  ```
- [ ] 首次启动自动生成密钥对：
  - Ed25519（`ed25519-dalek`）
  - X25519（`x25519-dalek`，从 Ed25519 key 派生或独立生成）
- [ ] 密钥安全存储：
  - 方案 A：OS Keychain（macOS Keychain / Windows Credential Manager / Linux Secret Service）
  - 方案 B：SQLite 存储 + 用户密码加密（`XChaCha20-Poly1305` 加密私钥 BLOB）
  - MVP 选方案 B，后续可升级到方案 A
- [ ] 实现 Tauri 命令：
  - `get_identity` → 公钥 + 昵称 + 颜色
  - `update_profile(nickname, color)` — 修改昵称和颜色
- [ ] 身份交换协议：
  - 连接建立后，通过 libp2p Identify 或自定义协议交换公钥和昵称
  - 存储到 `trusted_devices` 表（扩展 Phase 1.3 的表结构）

**验证点**：首次启动生成身份，重启后身份一致。两台设备连接后互相看到对方昵称。

---

## 3.2 文档加密（对称加密）

**目标**：每篇文档拥有独立的对称加密密钥，所有 yrs Update 加密后再传输和存储。

### 加密算法

**XChaCha20-Poly1305**（见技术选型文档详细分析）

### 加密流程

```
本地编辑 → yrs Update (binary)
  → XChaCha20-Poly1305 加密（文档密钥 + 随机 nonce）
  → 加密消息广播到 GossipSub / 发送到 Request-Response

收到加密消息
  → 查找文档密钥（按 doc_id + key_id）
  → XChaCha20-Poly1305 解密
  → yrs Update apply 到本地 Doc
```

### 加密消息格式

```
+--------+-------+---------+----------------+----------+
| doc_id | nonce | key_id  | ciphertext     | auth_tag |
| 32B    | 24B   | 4B      | variable       | 16B      |
+--------+-------+---------+----------------+----------+
```

### 任务

- [ ] 添加加密依赖：
  ```toml
  chacha20poly1305 = "0.10"
  rand = "0.8"
  ```
- [ ] 定义 `document_keys` 表 Entity：
  ```rust
  pub struct Model {
      pub doc_id: String,
      pub key_id: i32,               // 支持密钥轮换
      pub key_data: Vec<u8>,         // 256-bit 对称密钥
      pub created_at: i64,
  }
  ```
- [ ] 创建文档时自动生成文档密钥（256-bit 随机）
- [ ] 实现加密/解密模块：
  ```rust
  fn encrypt_update(key: &[u8; 32], update: &[u8]) -> EncryptedMessage { ... }
  fn decrypt_update(key: &[u8; 32], msg: &EncryptedMessage) -> Result<Vec<u8>> { ... }
  ```
- [ ] 改造 GossipSub 发送路径：编辑 → update → 加密 → 广播
- [ ] 改造 GossipSub 接收路径：收到消息 → 解密 → apply
- [ ] 改造 Request-Response 同步：全量同步的数据也加密传输
- [ ] 本地存储加密选项：
  - `current_state` 和 `updates` 表中的 BLOB 是否也加密存储？
  - 建议 MVP 阶段本地不加密（SQLite 已在用户私有目录），后续可加

**验证点**：抓包分析 GossipSub 消息，确认内容为密文不可读。只有持有密钥的节点能正确显示文档。

---

## 3.3 密钥分发（Lockbox）

**目标**：通过 X25519 密钥交换安全地将文档密钥分发给协作者。

### Lockbox 机制

文档密钥用每个协作者的 X25519 公钥分别加密，形成 Lockbox：

```
文档元数据（不加密，随文档同步）：
{
  doc_id: "abc123",
  lockboxes: [
    {
      recipient_pubkey: <alice_x25519_pk>,
      encrypted_doc_key: <x25519_shared_secret 加密的文档密钥>,
      created_at: 1706000000,
    },
    {
      recipient_pubkey: <bob_x25519_pk>,
      encrypted_doc_key: ...,
      created_at: 1706000000,
    },
  ]
}
```

### 密钥交换流程

```
Alice（文档拥有者）                    Bob（被邀请者）
    │                                    │
    │  1. Alice 获取 Bob 的 X25519 公钥   │
    │     （通过 Identify 或信任设备列表）  │
    │                                    │
    │  2. Alice 计算 shared_secret:       │
    │     X25519(alice_sk, bob_pk)        │
    │                                    │
    │  3. Alice 用 shared_secret 加密      │
    │     文档密钥，创建 Lockbox           │
    │                                    │
    │── 4. 发送 Lockbox 给 Bob ──────────>│
    │                                    │
    │     5. Bob 计算相同 shared_secret:   │
    │        X25519(bob_sk, alice_pk)      │
    │                                    │
    │     6. Bob 解密 Lockbox 获得文档密钥  │
    │                                    │
    │<══ 7. Bob 可以解密文档内容了 ════════│
```

### 任务

- [ ] 添加密钥交换依赖：
  ```toml
  x25519-dalek = { version = "2", features = ["static_secrets"] }
  ```
- [ ] 定义 `lockboxes` 表 Entity：
  ```rust
  pub struct Model {
      pub id: i32,
      pub doc_id: String,
      pub recipient_pubkey: Vec<u8>,   // X25519 公钥
      pub encrypted_key: Vec<u8>,      // 加密后的文档密钥
      pub sender_pubkey: Vec<u8>,      // 发送者公钥（用于解密）
      pub created_at: i64,
  }
  ```
- [ ] 实现 Lockbox 创建：
  ```rust
  fn create_lockbox(
      sender_sk: &StaticSecret,
      recipient_pk: &PublicKey,
      doc_key: &[u8; 32],
  ) -> Lockbox { ... }
  ```
- [ ] 实现 Lockbox 解密：
  ```rust
  fn open_lockbox(
      recipient_sk: &StaticSecret,
      sender_pk: &PublicKey,
      lockbox: &Lockbox,
  ) -> Result<[u8; 32]> { ... }
  ```
- [ ] Lockbox 同步：
  - Lockbox 随文档元数据通过 Request-Response 同步
  - 新节点加入时拉取 Lockbox 列表
  - 解密成功后将文档密钥存入本地 `document_keys` 表

**验证点**：Alice 创建文档，分享给 Bob。Bob 收到 Lockbox 后能解密文档。Charlie 没有 Lockbox，无法解密。

---

## 3.4 文档分享流程

**目标**：用户友好的文档分享体验。

### 分享码设计

```
swarmnote://<base58 编码>

解码后内容：
{
  doc_id: "abc123",
  doc_key: <文档对称密钥>,         // 直接包含密钥（简单方案）
  bootstrap: ["<multiaddr>", ...], // 可选：引导节点地址
}
```

> 注意：分享码包含明文密钥，因此需要通过安全渠道传递（类似 Signal 群组链接）。

### 任务

- [ ] 实现分享码生成：
  - 选择文档 → 生成分享码（base58 编码）
  - 包含 doc_id + 文档密钥 + 当前节点地址（可选）
- [ ] 实现分享码解析：
  - 输入分享码 → 解码 → 提取 doc_id 和密钥
  - 通过 DHT `GET_PROVIDERS(hash(doc_id))` 查找文档持有者
  - 连接到持有者 → Request-Response 全量同步
  - 本地存储文档密钥
- [ ] 前端 UI：
  - 文档菜单 → "分享" → 显示分享码 + 复制按钮
  - 主界面 → "加入文档" → 输入/粘贴分享码
  - 分享后显示当前协作者列表
- [ ] Lockbox 自动创建：
  - 当已信任的设备通过分享码加入时，自动为其创建 Lockbox
  - 后续该设备可通过 Lockbox 解密，不再需要分享码中的明文密钥
- [ ] 权限管理（基础）：
  - 文档拥有者（创建者）
  - 协作者（通过分享获得密钥）
  - MVP 阶段不区分只读/可编辑，所有协作者均可编辑

**验证点**：Alice 分享文档 → Bob 输入分享码 → 自动同步 → 双方可协作编辑。

---

## 3.5 文件夹级密钥派生（HKDF）

**目标**：分享整个文件夹时，从文件夹密钥自动派生每篇文档的密钥。

### 派生方案

```
文件夹密钥（256-bit 随机）
  │
  ├── HKDF(folder_key, "doc:" + doc_id_1) → 文档 1 密钥
  ├── HKDF(folder_key, "doc:" + doc_id_2) → 文档 2 密钥
  └── HKDF(folder_key, "doc:" + doc_id_3) → 文档 3 密钥
```

分享文件夹时只需分享一个文件夹密钥，每篇文档的密钥由 HKDF 自动派生。

### 任务

- [ ] 添加密钥派生依赖：
  ```toml
  hkdf = "0.12"
  sha2 = "0.10"
  ```
- [ ] 定义 `folders` 表 Entity（doc_id → folder_id 的关联）
- [ ] 实现密钥派生：
  ```rust
  fn derive_doc_key(folder_key: &[u8; 32], doc_id: &str) -> [u8; 32] {
      let hk = Hkdf::<Sha256>::new(None, folder_key);
      let mut doc_key = [0u8; 32];
      let info = format!("doc:{}", doc_id);
      hk.expand(info.as_bytes(), &mut doc_key).unwrap();
      doc_key
  }
  ```
- [ ] 文件夹分享码：与文档分享码类似，但包含 folder_key
- [ ] 新文档加入文件夹时自动派生密钥
- [ ] Lockbox 兼容：文件夹的 Lockbox 包含 folder_key 而非单个 doc_key

**验证点**：分享一个包含 3 篇文档的文件夹，接收方通过一个分享码获得所有文档的访问权限。

---

## 3.6 密钥轮换

**目标**：移除协作者后，生成新密钥防止前成员读取后续内容。

### 轮换流程

```
1. 拥有者移除协作者 Charlie
2. 生成新文档密钥（key_id + 1）
3. 用新密钥加密当前 CRDT 状态快照
4. 为剩余协作者（Alice, Bob）创建新 Lockbox
5. 后续所有 update 使用新密钥加密
6. Charlie 仍持有旧密钥，可读取轮换前的数据（无法避免）
```

### 任务

- [ ] 实现密钥轮换逻辑：
  - 生成新密钥 → key_id 递增
  - 重新创建所有剩余协作者的 Lockbox
  - 广播密钥轮换通知
- [ ] 多密钥支持：
  - 接收消息时按 `key_id` 字段查找对应密钥解密
  - 本地可能同时持有多个版本的密钥（用于解密历史数据）
- [ ] 前端 UI：
  - 文档协作者管理面板
  - 移除协作者按钮 → 触发密钥轮换
  - 显示密钥版本信息
- [ ] 限制：
  - P2P 系统无法阻止已移除用户保留之前已解密的数据
  - 密钥轮换仅提供前向安全性

**验证点**：Alice 移除 Charlie → Alice 和 Bob 继续编辑 → Charlie 无法解密新内容 → Charlie 仍能查看移除前的历史内容。

---

## 3.7 E2E 加密的网络层集成

**目标**：确保所有 P2P 传输的数据都经过 E2E 加密。

- [ ] GossipSub 消息：yrs Update 加密后广播
- [ ] Request-Response 文档同步：全量同步的状态数据加密传输
- [ ] Request-Response 文件同步：资源文件加密传输（文件密钥 = 所属文档的密钥，或独立密钥）
- [ ] Awareness 数据：光标/选区信息是否需要加密？
  - 建议不加密（不包含文档内容，仅位置信息）
  - 或使用同一文档密钥加密（更安全但增加开销）
- [ ] DHT Provider Records：不加密（仅包含 hash(doc_id)，不暴露原始 doc_id）
- [ ] 确保引导节点 / Relay 节点无法读取任何文档内容

**验证点**：部署一个 Relay 节点，两台设备通过 Relay 通信。检查 Relay 节点日志，确认无法读取文档内容。

---

## 执行顺序

```
3.1 用户身份 ──> 3.2 文档加密 ──> 3.3 密钥分发（Lockbox）──> 3.4 文档分享
                                       │
                                       ├──> 3.5 文件夹密钥派生
                                       │
                                       └──> 3.6 密钥轮换

3.7 网络层集成（在 3.2 完成后逐步进行）
```

- **3.1 → 3.2**：先有身份才能加密
- **3.2 → 3.7**：加密模块完成后立即集成到网络层
- **3.3 → 3.4**：Lockbox 实现后才能做分享流程
- **3.5 + 3.6**：可与 3.4 并行开发

---

## Rust 依赖汇总

```toml
# E2E 加密
chacha20poly1305 = "0.10"
x25519-dalek = { version = "2", features = ["static_secrets"] }
ed25519-dalek = { version = "2", features = ["rand_core"] }
hkdf = "0.12"
sha2 = "0.10"
rand = "0.8"
```
