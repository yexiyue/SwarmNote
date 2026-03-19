# SQLite 数据模型

## 全局数据库（~/.swarmnote/devices.db）

```sql
-- 已配对设备
CREATE TABLE paired_devices (
    peer_id TEXT PRIMARY KEY,
    public_key BLOB NOT NULL,        -- Ed25519 公钥
    device_name TEXT NOT NULL,
    os_info TEXT,
    paired_at INTEGER NOT NULL
);
```

## 工作区数据库（.swarmnote/workspace.db）

```sql
-- 工作区
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,             -- UUID v7
    name TEXT NOT NULL,
    created_by TEXT NOT NULL,        -- PeerId
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 工作区密钥（Stronghold 加密存储，此处存加密后的 blob）
CREATE TABLE workspace_keys (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(id),
    read_key_enc BLOB NOT NULL,      -- 加密后的 read_key
    write_key_enc BLOB,              -- 加密后的 write_key（Reader 无此字段）
    admin_key_enc BLOB,              -- 加密后的 admin_key（仅 Owner）
    key_version INTEGER NOT NULL DEFAULT 1,  -- 密钥版本号，轮换时递增
    updated_at INTEGER NOT NULL
);

-- 文件夹（映射到文件系统目录）
CREATE TABLE folders (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    parent_folder_id TEXT REFERENCES folders(id),
    name TEXT NOT NULL,              -- 目录名
    rel_path TEXT NOT NULL,          -- 相对于工作区根目录的路径
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 文档（映射到 .md 文件）
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    folder_id TEXT REFERENCES folders(id),
    title TEXT NOT NULL,
    rel_path TEXT NOT NULL,          -- 相对于工作区根目录的 .md 文件路径
    file_hash BLOB,                  -- blake3 hash（用于变更检测）
    yjs_state BLOB,                  -- yjs 文档状态（实时协作用）
    state_vector BLOB,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 文档 chunk 索引（FastCDC 分块同步用）
CREATE TABLE doc_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    doc_id TEXT NOT NULL REFERENCES documents(id),
    chunk_offset INTEGER NOT NULL,
    chunk_length INTEGER NOT NULL,
    chunk_hash BLOB NOT NULL,        -- blake3
    UNIQUE(doc_id, chunk_offset)
);

-- 权限
CREATE TABLE permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_type TEXT NOT NULL,      -- 'workspace' | 'folder' | 'document'
    resource_id TEXT NOT NULL,
    peer_id TEXT NOT NULL,
    role TEXT NOT NULL,               -- 'owner' | 'editor' | 'reader'
    granted_by TEXT NOT NULL,
    granted_at INTEGER NOT NULL,
    UNIQUE(resource_type, resource_id, peer_id)
);

-- 邀请链接
CREATE TABLE share_invites (
    token TEXT PRIMARY KEY,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    role TEXT NOT NULL,
    encrypted_keys BLOB NOT NULL,    -- 用 invite_secret 加密的密钥包
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    max_uses INTEGER,
    used_count INTEGER NOT NULL DEFAULT 0,
    password_hash TEXT               -- bcrypt，NULL = 无密码保护
);

```
