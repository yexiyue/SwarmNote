# E2E 加密设计方案

> 参考架构：SecSync（Serenity Notes 使用的 E2E 加密 CRDT 方案，NLnet 资助）。
> 本文档描述底层加密实现细节，权限模型层面的密钥分发规则见 [04-permissions.md](04-permissions.md)。

## 设计原则

- **设备信任 ≠ 文档授权**：设备信任允许网络连接，文档授权通过密钥分发控制访问权限
- **中继节点不可读**：引导节点和 Relay 节点不能解密任何文档内容
- **前向安全性**：移除协作者后，新内容不可读（但无法阻止保留已解密的旧数据）

## 加密算法

**XChaCha20-Poly1305**

| 维度 | 选择理由 |
|------|---------|
| Nonce | 24 字节，P2P 无法协调计数器，必须随机生成，192-bit 空间消除碰撞风险 |
| 跨平台 | 纯 Rust 实现，无 C 依赖 |
| 侧信道 | 不依赖硬件加速，任何设备上都是常数时间 |
| 验证 | SecSync 同款选择，已在生产环境验证 |

## 密钥层级

```
用户主密钥（Stronghold 保护）
  ├── Ed25519 签名密钥对（验证消息来源、签名文档更新）
  ├── X25519 密钥交换密钥对（与其他用户安全交换文档对称密钥）
  ├── 工作区密钥组（见 04-permissions.md）
  │     ├── read_key   → 解密文档内容
  │     ├── write_key   → 签署编辑操作
  │     └── admin_key   → 签署权限变更
  └── 文件夹级密钥（可选扩展，HKDF 派生文档密钥）
```

## 加密消息格式

对每个同步消息整体加密后传输：

```
+--------+-------+---------+----------------+----------+
| doc_id | nonce | key_id  | ciphertext     | auth_tag |
| 32B    | 24B   | 4B      | variable       | 16B      |
+--------+-------+---------+----------------+----------+
```

- doc_id 明文：网络层需要据此路由消息
- nonce 随机生成：每条消息唯一
- key_id 支持密钥轮换
- auth_tag AEAD 认证标签，防篡改

## 密钥分发（Lockbox）

文档密钥用每个协作者的 X25519 公钥分别加密，形成 Lockbox：

1. Alice 获取 Bob 的 X25519 公钥
2. Alice 计算 `shared_secret = X25519(alice_sk, bob_pk)`
3. Alice 用 shared_secret 加密文档密钥，创建 Lockbox
4. 发送 Lockbox 给 Bob
5. Bob 计算相同 `shared_secret = X25519(bob_sk, alice_pk)`
6. Bob 解密 Lockbox 获得文档密钥

## 文件夹级密钥派生（HKDF）

```
文件夹密钥（256-bit 随机）
  ├── HKDF(folder_key, "doc:" + doc_id_1) → 文档 1 密钥
  ├── HKDF(folder_key, "doc:" + doc_id_2) → 文档 2 密钥
  └── HKDF(folder_key, "doc:" + doc_id_3) → 文档 3 密钥
```

```rust
fn derive_doc_key(folder_key: &[u8; 32], doc_id: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, folder_key);
    let mut doc_key = [0u8; 32];
    let info = format!("doc:{}", doc_id);
    hk.expand(info.as_bytes(), &mut doc_key).unwrap();
    doc_key
}
```

## 密钥轮换（移除协作者时）

1. Owner 移除协作者 Charlie
2. 生成新密钥组（key_version + 1）
3. 用新密钥加密当前状态快照
4. 为剩余协作者创建新 Lockbox
5. 后续所有消息使用新密钥加密
6. Charlie 仍持有旧密钥，可读取轮换前的数据（P2P 系统无法避免）

## 网络层集成

| 数据 | 是否加密 | 原因 |
|------|---------|------|
| yjs Update / FastCDC chunk | 加密 | 文档内容 |
| 资源文件传输 | 加密 | 用户数据 |
| Awareness 数据 | 不加密 | 不含文档内容（光标位置等） |
| DHT Provider Records | 不加密 | 仅含 hash(doc_id)，不暴露原始 ID |

## Rust 依赖

```toml
chacha20poly1305 = "0.10"
x25519-dalek = { version = "2", features = ["static_secrets"] }
ed25519-dalek = { version = "2", features = ["rand_core"] }
hkdf = "0.12"
sha2 = "0.10"
rand = "0.8"
```
