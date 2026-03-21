# Stronghold 密钥管理与设备身份

## 用户故事

作为用户，我希望我的设备有一个唯一的加密身份，以便将来安全地与其他设备同步笔记。

## 需求描述

使用 Tauri Stronghold 插件安全地生成和存储设备密钥对。在 Onboarding 完成时自动生成，后续启动时自动加载。参考 `dev-notes/design/01-device-identity.md`。

### 密钥体系

| 密钥 | 用途 | 算法 |
|------|------|------|
| Ed25519 签名密钥对 | 消息签名、CRDT 操作验证 | Ed25519 |
| X25519 密钥交换对 | Lockbox 密钥分发（后续 E2E 加密） | X25519 |
| PeerId | 设备唯一标识 | 从 Ed25519 公钥派生 |

## 技术方案

### 后端

- 使用 `tauri-plugin-stronghold` 管理密钥存储
- Stronghold snapshot 文件：`~/.swarmnote/identity.stronghold`
- 密码保护：Onboarding 时输入（或自动生成，待定）
- 密钥生成流程：
  1. 生成 Ed25519 密钥对
  2. 从 Ed25519 私钥派生 X25519 密钥对
  3. 从 Ed25519 公钥计算 PeerId（libp2p 兼容格式）
  4. 存入 Stronghold

### Tauri Commands

- `#[tauri::command] fn generate_identity(password)` — 生成密钥对并存储
- `#[tauri::command] fn load_identity(password)` — 加载已有密钥
- `#[tauri::command] fn get_peer_id()` — 返回 PeerId 字符串
- `#[tauri::command] fn get_device_info()` — 返回设备名、PeerId、OS 等

### 数据结构

```rust
struct DeviceIdentity {
    peer_id: String,
    device_name: String,
    os_info: String,
    created_at: String,
}
```

### 存储位置

```
~/.swarmnote/
├── identity.stronghold    ← 加密密钥存储
├── devices.db             ← 配对设备数据库
└── config.toml            ← 全局配置（设备名等）
```

## 验收标准

- [ ] Onboarding 完成时自动生成 Ed25519 + X25519 密钥对
- [ ] 密钥通过 Stronghold 加密存储在 `~/.swarmnote/identity.stronghold`
- [ ] 可从密钥派生 libp2p 兼容的 PeerId
- [ ] 应用重启后可自动加载已有密钥（无需重新输入密码 / 自动解锁）
- [ ] PeerId 在完成页和设置中可查看

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- Stronghold 是否需要密码保护？
  - 选项 A：Onboarding 时设置密码（更安全，但用户每次启动需输入）
  - 选项 B：使用系统密钥链自动解锁（更便捷）
  - 选项 C：自动生成随机密码存在系统密钥链中（兼顾安全和便捷）
  - 建议：v0.1.0 先用选项 C 或固定密码，后续迭代再优化
- 密钥备份/恢复策略？
  - 推迟到后续版本考虑
