# 权限模型

## 角色定义

| 角色 | 读 | 编辑 | 管理权限 | 删除 | 密码学能力 |
|------|:--:|:----:|:--------:|:----:|:----------:|
| Owner | ✓ | ✓ | ✓ | ✓ | 持有主密钥（read_key + write_key + admin_key） |
| Editor | ✓ | ✓ | | | 持有 read_key + write_key |
| Reader | ✓ | | | | 仅持有 read_key |

- **Owner**：资源创建者，唯一能管理权限（授权/撤销/转让）和删除资源的角色
- **Editor**：可编辑文档内容（CRDT 双向同步）
- **Reader**：只读，只接收文档更新

> 三级权限模型。不设 Commenter（评论系统对个人/小团队笔记场景优先级低，增加的复杂度不值得）。不设 Manager（飞书有但我们不做，单 Owner 足够）。

## 密码学权限执行

P2P 没有服务器强制执行权限，改用**密钥分发**控制访问：

### 每个工作区的密钥体系

```text
Workspace 创建时 Owner 生成：
├── read_key    (对称密钥，ChaCha20-Poly1305)  → 解密文档内容
├── write_key   (对称密钥)                      → 签署 CRDT 编辑操作
└── admin_key   (对称密钥)                      → 签署权限变更操作
```

### 密钥分发规则

| 授予角色 | 分发的密钥 |
|---------|-----------|
| Reader | read_key |
| Editor | read_key + write_key |
| Owner（转让） | read_key + write_key + admin_key |

### 密钥传输

通过已配对的 P2P 加密通道传输（libp2p Noise 协议已提供传输层加密）。链接分享场景下，密钥嵌入在邀请 token 中（见 [05-sharing.md](05-sharing.md)）。

### 操作验证

收到远程操作时，本地验证：

```text
收到 CRDT Update：
  → 验证发送者持有 write_key（检查操作签名）
  → 验证失败 → 丢弃该 update

收到权限变更：
  → 验证发送者持有 admin_key
  → 验证失败 → 丢弃
```

## 权限继承

```text
Workspace 权限 + 密钥
  └─ 向下继承到 Folder
       └─ 向下继承到 Document
```

- 子级**默认继承**父级权限和密钥
- 子级可以**覆盖**继承的权限（提升或降低）
- 冲突解决：**直接授权优先于继承**，与 Notion 类似
  - 同一设备通过多条路径获得权限时，取**最高权限**（Notion 的"最高权限胜出"规则）
- 覆盖记录单独存储，删除覆盖则恢复继承

示例：

- Workspace 授予 B 设备 Editor → B 可编辑该工作区下所有文档
- 某个 Folder 将 B 降为 Reader → B 只能查看该文件夹下的文档
- 该 Folder 下某篇 Document 将 B 提升为 Editor → B 可编辑这篇文档

### Folder/Document 级别独立密钥（可选扩展）

默认所有文档共享工作区密钥。如果需要更细粒度控制（如某个 Folder 有独立的 write_key），可为该 Folder 生成独立密钥组，降级用户只分发 read_key。

此特性复杂度较高，MVP 阶段建议工作区级别统一密钥，后续按需扩展。

## 权限数据结构

```rust
enum ResourceType {
    Workspace,
    Folder,
    Document,
}

enum Role {
    Owner,
    Editor,
    Reader,
}

/// 一条权限记录
struct Permission {
    resource_type: ResourceType,
    resource_id: Uuid,          // workspace/folder/document 的 ID
    peer_id: PeerId,            // 被授权的设备
    role: Role,
    granted_by: PeerId,         // 授权者
    granted_at: i64,
}
```

## 权限解析算法

查询某设备对某文档的有效权限：

```text
1. 查该文档是否有直接授权 → 有则返回
2. 查父文件夹是否有直接授权 → 有则返回
3. 递归向上查到工作区 → 有则返回
4. 无任何授权 → 拒绝访问
```

如果通过多条路径获得不同角色（如工作区 Editor + 文件夹 Reader），取最高权限。

## 权限撤销与密钥轮换

### 核心挑战

P2P 场景下权限撤销比中心化产品困难：
- 无服务器可以立即切断访问
- 被撤销者已持有解密密钥，本地已有数据无法收回
- 被撤销者可能离线，无法实时通知

### 撤销策略

#### 即时效果

```text
Owner 撤销 B 的权限：
1. 从本地权限表删除 B 的记录
2. 广播 PermissionRevoked 消息给所有在线协作者
3. 所有在线节点停止向 B 发送文档更新
4. B 的本地数据保留，但不再接收新内容
```

#### 密钥轮换（彻底撤销）

当需要确保被撤销者无法解密后续内容时：

```text
Owner 轮换密钥：
1. 生成新的 read_key / write_key
2. 向所有仍有权限的设备分发新密钥（通过 P2P 加密通道）
3. 后续文档更新用新密钥加密
4. 被撤销者的旧密钥只能解密轮换前的内容
```

> 参考 Google Docs 的做法：移除协作者后，对方保留已下载的内容但无法访问新内容。密钥轮换是 P2P 下的等价实现。

### Owner 转让

```text
A 转让 Owner 给 B：
1. A 将 admin_key 通过加密通道发送给 B
2. A 本地将自己的角色降为 Editor（或其他指定角色）
3. 广播 OwnerTransferred 消息
4. 同一资源始终只有一个 Owner（避免飞书/Notion 中的多管理员冲突）
```
