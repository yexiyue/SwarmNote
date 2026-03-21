# SQLite 存储层

## 用户故事

作为开发者，我需要一个可靠的本地数据库来索引文档元数据、存储设备信息和管理权限，以便应用高效运行。

## 需求描述

按照 `dev-notes/design/07-data-model.md` 的完整 schema 建立 SQLite 数据库。v0.1.0 实际使用的表有限（workspaces、folders、documents），其余表建好占位为后续版本使用。

### 数据库分层

| 数据库 | 位置 | 用途 |
|--------|------|------|
| `devices.db` | `~/.swarmnote/devices.db` | 全局设备信息（paired_devices 表） |
| `workspace.db` | `<workspace>/.swarmnote/workspace.db` | 工作区数据（文档、文件夹、权限等） |

## 技术方案

### 后端

- 使用 `sea-orm` crate（全功能异步 ORM，基于 sqlx，类型安全）
  - SQLite 驱动：`sea-orm` 的 `sqlx-sqlite` feature
  - Entity 代码生成：使用 `sea-orm-cli` 或手写 Entity
- 数据库迁移：使用 `sea-orm-migration` crate 管理 schema 迁移
- 连接管理：`sea-orm` 的 `DatabaseConnection`（内置连接池）

### Schema（完整，参考 07-data-model.md）

**devices.db:**

```sql
CREATE TABLE paired_devices (
    peer_id TEXT PRIMARY KEY,
    public_key BLOB NOT NULL,
    device_name TEXT NOT NULL,
    os_info TEXT,
    paired_at TEXT NOT NULL
);
```

**workspace.db:**

```sql
-- 核心表（v0.1.0 使用）
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE folders (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    parent_folder_id TEXT REFERENCES folders(id),
    name TEXT NOT NULL,
    rel_path TEXT NOT NULL,
    creator TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    folder_id TEXT REFERENCES folders(id),
    title TEXT NOT NULL,
    rel_path TEXT NOT NULL,
    file_hash TEXT,
    yjs_state BLOB,
    state_vector BLOB,
    creator TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 占位表（后续版本使用）
CREATE TABLE workspace_keys (...);
CREATE TABLE doc_chunks (...);
CREATE TABLE permissions (...);
CREATE TABLE share_invites (...);
```

### Tauri Commands

- `#[tauri::command] fn db_get_documents(workspace_id)` — 查询文档列表
- `#[tauri::command] fn db_upsert_document(doc)` — 插入或更新文档记录
- `#[tauri::command] fn db_delete_document(id)` — 删除文档记录
- 类似的 folder CRUD commands

## 验收标准

- [ ] 应用启动时自动创建/迁移数据库
- [ ] devices.db 在 `~/.swarmnote/` 下正确创建
- [ ] workspace.db 在工作区 `.swarmnote/` 下正确创建
- [ ] 完整 schema 全部建表成功
- [ ] 文档 CRUD 操作通过数据库正确持久化
- [ ] 应用重启后数据完整保留

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- ~~使用 `rusqlite`（同步）还是 `sqlx`（异步）？~~ **已决定：使用 sea-orm（基于 sqlx 的全功能 ORM）**
- UUID v7 生成用什么 crate？
  - 建议 `uuid` crate 的 v7 feature
