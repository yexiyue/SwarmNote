# 设备身份与配对

## 1. 设备身份（复用 SwarmDrop）

```
用户密码 → Stronghold 加密存储 → Ed25519 密钥对 → PeerId
```

- 首次启动：设置密码 → 生成密钥对 → 持久化到 Stronghold
- 后续启动：密码/生物识别解锁 → 加载密钥对
- PeerId = hash(公钥)，是设备的全局唯一标识

SwarmDrop 已验证此方案，直接复用 `auth-store` + `secret-store` + Stronghold 模式。

## 2. 设备配对（复用 SwarmDrop）

### 配对码流程

```
A 生成 6位配对码 → DHT 发布（监听地址 + 设备信息，5分钟过期）
B 输入配对码 → DHT 查询 → 获取 A 的 PeerId + 地址
B 发送配对请求 → A 确认 → 双方互存 PeerId + 公钥 → 信任建立
```

### 配对结果

双方各自在本地存储对方的 `PairedDevice` 信息：

```rust
struct PairedDevice {
    peer_id: PeerId,
    public_key: Ed25519PublicKey,  // 用于后续签名验证
    device_name: String,           // 用户可自定义
    os_info: OsInfo,
    paired_at: i64,
}
```

配对只建立**设备间信任通道**（交换公钥），不自动授予任何文档权限。
