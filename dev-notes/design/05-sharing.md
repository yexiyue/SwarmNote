# 分享机制

## 已配对设备分享

前提：双方已完成设备配对。

```
A 选择 Workspace/Folder/Document → 选择已配对设备 B → 选择角色
→ 根据角色分发对应密钥（通过 P2P 加密通道）
→ 写入本地权限表
→ 通过 P2P 通知 B："你被授权访问资源 X，角色为 Y"
→ B 收到密钥 + 权限信息 → 存储到本地 → 开始同步
```

## 链接分享

不需要预先配对，适合分享给不在同一网络的人。

### 邀请链接格式

```
swarmnote://invite/<token>
```

### 邀请数据（发布到 DHT）

```rust
struct ShareInvite {
    token: String,              // 随机生成的邀请令牌
    resource_type: ResourceType,
    resource_id: Uuid,
    role: Role,                 // 链接授予的角色
    encrypted_keys: Vec<u8>,    // 用 invite_secret 加密的密钥包
    creator_peer_id: PeerId,    // 创建者
    creator_addrs: Vec<Multiaddr>,
    created_at: i64,
    expires_at: i64,            // 过期时间
    max_uses: Option<u32>,      // 最大使用次数，None = 无限
    password_hash: Option<String>, // 密码保护（bcrypt hash）
}
```

### 链接分享流程

```
A 创建邀请：
  1. 生成 token + invite_secret（随机 32 字节）
  2. 用 invite_secret 加密角色对应的密钥包
  3. 构建 ShareInvite
  4. 发布到 DHT: key = SHA256("/swarmnote/invite/" + token)
  5. 生成链接 swarmnote://invite/<token>#<invite_secret_base64>
     （invite_secret 在 URL fragment 中，不会被 DHT 存储）

B 使用邀请：
  1. 打开链接 → 应用解析 token + invite_secret
  2. DHT 查询 → 获取 ShareInvite
  3. 检查：未过期、未超最大使用次数
  4. 如果有密码保护 → 提示输入密码 → 验证 password_hash
  5. 用 invite_secret 解密密钥包 → 获得 read_key / write_key 等
  6. 连接 A（通过 creator_addrs）
  7. 发送 InviteRedeemRequest { token, peer_id }
  8. A 验证 → 双方建立信任 + 记录权限
  9. B 用解密得到的密钥开始同步
```

### 链接安全性

- **invite_secret** 在 URL fragment（`#` 后）中，类似 Mega.nz 的做法
  - DHT 只存储加密后的密钥包，无法解密
  - 只有拿到完整链接的人才能解密
- **密码保护**（参考语雀）：额外一层验证，即使链接泄露也需要密码
- **有效期**（参考腾讯文档）：支持自定义过期时间
- **使用次数**：可限制链接最多被使用 N 次

### 链接分享 vs 配对分享的区别

| | 配对分享 | 链接分享 |
|--|---------|---------|
| 前提 | 需要先配对 | 不需要 |
| 信任 | 已有设备信任 | 通过邀请链接建立 |
| 密钥传输 | P2P 加密通道直传 | 嵌入链接 fragment，DHT 存加密包 |
| 安全性 | 高（端到端加密） | 中（依赖链接不泄露） |
| 密码保护 | 不需要 | 可选 |
| 有效期 | 无（持久授权） | 可设过期时间 |
| 场景 | 自己的多台设备 / 信任的人 | 分享给同事/朋友 |
