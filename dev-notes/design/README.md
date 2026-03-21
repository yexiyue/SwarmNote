# SwarmNote 系统设计

本目录包含 SwarmNote 的核心系统设计文档，按模块拆分。

## 文档索引

| 文件 | 内容 |
|------|------|
| [01-device-identity.md](01-device-identity.md) | 设备身份（Stronghold + Ed25519）与 6 位配对码流程 |
| [02-storage-architecture.md](02-storage-architecture.md) | 存储架构：Markdown 优先、目录结构、工作区发现/移动、容灾恢复 |
| [03-sync-architecture.md](03-sync-architecture.md) | 三级同步：L1 yjs 实时协作 / L2 FastCDC 分块同步 / L3 资源全量同步 |
| [04-permissions.md](04-permissions.md) | 三级权限模型（Owner/Editor/Reader）、密码学执行、权限继承、撤销与密钥轮换 |
| [05-sharing.md](05-sharing.md) | 配对分享与链接分享（DHT 邀请、密码保护、有效期） |
| [07-data-model.md](07-data-model.md) | SQLite 数据模型（全局 db + 工作区 db） |
| [08-e2e-encryption.md](08-e2e-encryption.md) | E2E 加密底层实现（XChaCha20、Lockbox、HKDF 派生） |
| [09-decisions.md](09-decisions.md) | 设计决策记录、开放问题、分阶段实现路线 |
| [10-ui-requirements.md](10-ui-requirements.md) | UI 设计需求：页面清单、交互细节、快捷键 |

## 设计原则

1. **Markdown 优先**：文档存为 .md 文件，兼容 Obsidian，任何编辑器可打开
2. **密码学即权限**：P2P 无服务器，用密钥分发和签名验证替代服务端访问控制
3. **三级同步**：yjs 实时协作 → FastCDC 分块同步 → 资源全量同步，按场景选最优策略
4. **文件即数据**：.md 文件是真相源，SQLite 是可重建的索引

## 相关文档

- [../product-vision.md](../product-vision.md) — 产品愿景与路线图
- [../mvp-tasks.md](../mvp-tasks.md) — MVP 任务拆解
- [../research/](../research/) — 技术调研（BlockNote、文档分享权限、文件同步模式）
