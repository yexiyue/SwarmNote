# 移动端文件系统与资源文件策略

> 调研日期：2026-03-26
> 目的：分析 Android/iOS 沙盒文件系统对 SwarmNote 移动端的影响，确定资源文件（图片/视频）处理方案

---

## 一、移动端文件系统限制

### 1.1 Android (Scoped Storage, Android 11+)

Android 10 引入、Android 11 强制执行 Scoped Storage，应用只能自由访问自己的沙盒目录。

| 方案 | 说明 | 限制 |
|---|---|---|
| App 私有目录 `/Android/data/<pkg>/` | 无需权限，卸载时删除 | 用户不可见，其他 app 无法访问 |
| SAF `ACTION_OPEN_DOCUMENT_TREE` | 用户选择目录后授予持久访问 | Android 11+ 禁止选根目录/Downloads/SD卡根 |
| MediaStore + `Documents/` | 写入共享 Documents 目录 | 只能看到自己创建的文件 |
| `MANAGE_EXTERNAL_STORAGE` | 完全外部存储权限 | Google Play 严格限审，笔记应用基本不会通过 |

**Obsidian 在 Android 的做法**：默认存在 `/Android/data/md.obsidian/files/`（用户不可见），或通过 SAF 让用户选文件夹（Android 13+ 限制增多，社区抱怨不断）。

### 1.2 iOS (严格沙盒)

| 方案 | 说明 | 限制 |
|---|---|---|
| App 沙盒 `Documents/` | 默认方案 | 其他 app 无法直接访问 |
| File Provider Extension | 在"文件" app 中显示 | 需写原生扩展，Expo 不直接支持 |
| iCloud Container | 自动同步 | 依赖 Apple 生态 |
| Document Picker | 可选择外部文件 | 用户体验割裂 |

### 1.3 对 SwarmNote 的影响

**P2P 同步不受影响**：同步发生在 yjs CRDT + 数据库层，不依赖文件系统。

**受影响的是 "Markdown 文件即真相源" 设计原则**：

| 桌面端设计 | 移动端现实 |
|---|---|
| 用户能直接浏览 `.md` 文件 | 沙盒内文件用户不可见 |
| 可以用 Obsidian 打开同一目录 | 跨 app 访问需要 SAF/File Provider |
| 文件夹就是组织结构 | 文件夹在沙盒内，用户感知不到 |
| 删库了从 `.md` 重建 | `.md` 和 SQLite 都在沙盒内，没本质区别 |

---

## 二、存储分层策略

桌面和移动端使用不同的"真相源"定义：

```
┌──────────────────────────────────────────────────┐
│              SwarmNote 存储分层                    │
├──────────────┬──────────────┬────────────────────┤
│  桌面端       │  移动端       │  同步层             │
├──────────────┼──────────────┼────────────────────┤
│ .md 文件      │ SQLite       │ yjs CRDT 更新       │
│ (真相源)      │ (真相源)      │ (平台无关)          │
│ SQLite 索引   │ .md 导出功能  │ swarm-p2p-core     │
│ (可重建)      │ (用户需要时)   │ (libp2p)           │
└──────────────┴──────────────┴────────────────────┘
```

- **移动端以 SQLite 为主存储**，不再维护实时的 `.md` 文件系统
- **提供"导出到文件"功能**，用户需要时可批量导出为 `.md`
- **同步层不变**，yjs CRDT 更新在桌面端写入 `.md` + SQLite 索引，在移动端只写入 SQLite
- **桌面端保持现有架构**，`.md` 文件仍然是真相源

---

## 三、资源文件（图片/视频）处理方案

### 3.1 桌面端现有设计

```
Document.md              ← 文本引用 ![](Document/screenshot.png)
Document/
  ├── screenshot.png      ← 图片
  └── demo.mp4            ← 视频
```

### 3.2 移动端方案：SQLite 元数据 + 沙盒文件存储

```
App Sandbox/
├── swarmnote.db                    ← SQLite (笔记文本 + 元数据)
└── resources/                      ← 资源文件目录
    ├── a1b2c3d4.png                ← content-hash 命名
    ├── e5f6g7h8.jpg
    └── i9j0k1l2.mp4
```

文本在 SQLite，资源文件在沙盒目录，用 content-hash (BLAKE3) 作为文件名。

### 3.3 数据模型

```sql
-- 资源表
CREATE TABLE resources (
  id          TEXT PRIMARY KEY,        -- UUID
  hash        TEXT NOT NULL UNIQUE,    -- BLAKE3 content hash（去重用）
  filename    TEXT NOT NULL,           -- 原始文件名 screenshot.png
  mime_type   TEXT NOT NULL,           -- image/png, video/mp4
  size        INTEGER NOT NULL,        -- 字节数
  width       INTEGER,                 -- 图片/视频宽度
  height      INTEGER,                 -- 图片/视频高度
  duration    INTEGER,                 -- 视频时长(ms)
  thumbnail   BLOB,                    -- 缩略图（几 KB，存 BLOB 没问题）
  local_path  TEXT,                    -- 本地沙盒路径（可能为空=未下载）
  sync_state  TEXT DEFAULT 'local',    -- local / syncing / synced
  created_at  INTEGER NOT NULL
);

-- 笔记-资源关联
CREATE TABLE doc_resources (
  doc_id      TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  PRIMARY KEY (doc_id, resource_id)
);
```

### 3.4 Content-hash 命名的好处

```
用户插入 screenshot.png
  → BLAKE3 hash → a1b2c3d4...
  → 存为 resources/a1b2c3d4.png
  → 笔记中引用 swarmnote://resource/a1b2c3d4
```

1. **天然去重**：同一张图插入多篇笔记，只存一份
2. **完整性校验**：同步时 hash 对比即可验证
3. **与 SwarmDrop 一致**：swarmdrop 的传输协议已经用 BLAKE3 做 chunk 校验

### 3.5 桌面 ↔ 移动引用转换

同步时需要做一层引用格式转换：

```
桌面 → 移动：
  相对路径 Document/screenshot.png
  → 下载文件到 resources/
  → 计算 hash → 写入 resources 表
  → 笔记中替换为 swarmnote://resource/{hash}

移动 → 桌面：
  swarmnote://resource/{hash}
  → 查 resources 表获取原始文件名
  → 复制到 Document/ 目录
  → 笔记中替换为相对路径
```

这层转换放在 Rust `app-core` 的同步模块里，对上层透明。

---

## 四、大文件同步策略

利用 SwarmNote 已设计的三级同步架构：

| 文件类型 | 同步级别 | 策略 |
|---|---|---|
| 小图片 (<1MB) | L3 全文件 | 直接传输，hash 比对 |
| 大图片/视频 (>1MB) | L2 FastCDC 分块 | 256KB 分块 + BLAKE3，支持断点续传 |
| 缩略图 | 随 SQLite 同步 | BLOB 字段，几 KB |

### 按需下载（省空间）

```
移动端收到同步：
  1. 先同步元数据（hash, size, mime_type, thumbnail）→ 立即可预览缩略图
  2. 用户点开笔记时再下载原图/视频
  3. local_path 为空 = 未下载，显示缩略图 + 下载按钮
  4. 存储空间不足时可清理已下载的原文件，保留缩略图
```

类似 iCloud 照片的"优化存储空间"。

---

## 五、RN 端实现工具链

| 功能 | 库 | 说明 |
|---|---|---|
| 相机拍照 | `expo-camera` / `expo-image-picker` | 拍照后复制到 resources/ |
| 相册选取 | `expo-image-picker` | 支持多选、视频 |
| 文件操作 | `expo-file-system` (SDK 54 新 API) | `File` / `Directory` 类，原生支持 SAF 和 Security Scoped Resources |
| 图片压缩 | `expo-image-manipulator` | 插入前可压缩 |
| 视频缩略图 | `expo-video-thumbnails` | 生成预览帧 |
| 图片显示 | `expo-image` | 支持缓存和渐进加载 |
| 剪贴板 | `expo-clipboard` | 粘贴图片 |
| 文件导出 | `expo-sharing` + `expo-file-system` | 导出 .md + 资源文件 |

均为 Expo 官方库，与 Expo 54 技术栈兼容。

---

## 六、主流笔记 App 的移动端策略参考

| App | 存储方式 | 文件可见性 | 同步 |
|---|---|---|---|
| **Obsidian** | 文件系统 `.md` | Android: SAF / 私有目录; iOS: iCloud | Obsidian Sync / iCloud / Syncthing |
| **Joplin** | SQLite 数据库 | 不可见，需导出 | REST API + 云存储 |
| **Logseq** | 文件系统 (正迁移 SQLite) | 类似 Obsidian | Logseq Sync / iCloud / Git |
| **Notesnook** | SQLite | 不可见 | 自建同步 |

**趋势**：Logseq 正从文件系统迁移到 SQLite，说明纯文件系统方案在移动端摩擦太大。

---

## 七、总结

```
┌──────────────────────────────────────────────────┐
│               资源文件处理架构                     │
├───────────┬──────────────┬───────────────────────┤
│  桌面端    │   移动端      │   P2P 同步             │
├───────────┼──────────────┼───────────────────────┤
│ 文件系统   │ 沙盒文件      │ 元数据优先同步          │
│ 相对路径   │ content-hash │ 原文件按需下载           │
│ 原始文件名 │ 缩略图在 DB   │ FastCDC 分块传大文件    │
│ 无去重     │ hash 自动去重 │ BLAKE3 校验            │
└───────────┴──────────────┴───────────────────────┘
             ↕ 引用转换层 (Rust app-core)
```

移动端资源文件不需要用户可见，只要 app 内正常显示、导出时能还原原始文件名即可。沙盒限制因此不再是问题。
