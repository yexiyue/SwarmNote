# 存储架构

## 设计原则

- **Markdown 优先**：文档存为真实的 `.md` 文件，用户能用任何编辑器打开，和 Obsidian vault 兼容
- **文件即数据**：用户在文件管理器中能直接看到、复制、备份自己的笔记
- **db 是索引不是数据**：SQLite 存同步元数据和 yjs 状态，丢失后可从 .md 文件重建
- **身份全局，数据跟着工作区走**：设备身份存全局目录，工作区数据可随目录移动

## 目录结构

### 全局目录（设备级，不随工作区移动）

```
~/.swarmnote/                    # 全局配置目录
├── identity/                    # Stronghold 密钥（设备身份）
├── devices.db                   # 已配对设备列表
└── config.toml                  # 全局配置（默认工作区路径等）
```

### 工作区目录（用户可见，可移动/复制）

```
~/Notes/我的笔记/                 # 工作区根目录
├── .swarmnote/                  # 隐藏目录（同步引擎数据）
│   ├── workspace.db             # 文档索引、yjs state、权限、评论
│   └── config.toml              # 工作区级配置
├── 学习笔记.md                  # 真实的 Markdown 文件
├── 学习笔记/                    # 同名资源目录（图片、附件等）
│   ├── screenshot.png
│   └── diagram.svg
├── 项目计划.md
├── 子目录/                      # 文件夹（就是真实的文件系统目录）
│   ├── 会议记录.md
│   └── 会议记录/
│       └── whiteboard.png
└── 随想.md
```

## 文档格式

- **文档**：标准 Markdown (`.md`) 文件
  - BlockNote 编辑 → `blocksToMarkdownLossy()` 保存为 .md
  - 打开 .md → `tryParseMarkdownToBlocks()` 加载到 BlockNote
  - 有损转换：Markdown 不支持的 BlockNote 特性（文字颜色、多列布局等）会丢失，但笔记场景下 Markdown 覆盖 95% 需求
- **资源文件**：同名目录存放，.md 中用相对路径引用 `![](./学习笔记/screenshot.png)`
- **文件夹**：直接映射到文件系统目录，不需要额外的元数据

## 文档组织结构

```
Workspace（工作区）= 文件系统中的一个目录（含 .swarmnote/）
├── Folder（文件夹，可嵌套）= 子目录
│   ├── Document = .md 文件
│   ├── Document
│   └── Folder
│       └── Document
└── Document（工作区根目录下的文档）
```

- 每个用户默认有一个"我的笔记"工作区（本地私有，不参与同步）
- 可创建协作工作区，邀请其他设备加入
- Workspace / Folder / Document 都有全局唯一 ID（UUID v7，时间有序），存在 workspace.db 中，与文件路径映射

参考：Notion 的 Workspace → Teamspace → Page 层级，语雀的 空间 → 知识库 → 文档 层级。SwarmNote 简化为三层并直接映射到文件系统，用户心智模型与 Obsidian 一致。

## 工作区发现机制

应用启动时，向上搜索 `.swarmnote/` 目录（和 Git 查找 `.git/` 一样）：

```
用户指定目录 ~/Notes/我的笔记/子目录/
1. 检查 ./子目录/.swarmnote/ → 没有
2. 检查 ../我的笔记/.swarmnote/ → 找到了
3. 工作区根 = ~/Notes/我的笔记/
4. 当前视图 = 子目录/（只显示这个目录下的文件）
```

如果一路到文件系统根目录都没找到 → 不是 SwarmNote 工作区 → 提示用户初始化。

## 工作区移动/迁移

- **workspace.db 内全部使用相对路径**（相对于工作区根目录），不存绝对路径
- 用户移动整个工作区目录（包含 `.swarmnote/`）到任意位置 → 无需任何配置即可正常使用
- 全局 `~/.swarmnote/config.toml` 中记录的工作区路径需要用户更新（或应用首次打开时重新定位）

## 两层数据与容灾

| 数据 | 存储位置 | 定位 | 丢失影响 |
|------|---------|------|---------|
| 文档内容 | `.md` 文件 | **真相源** | 数据丢失（但有 P2P 副本可恢复） |
| 资源文件 | 同名目录 | **真相源** | 图片/附件丢失 |
| yjs state | workspace.db | 同步索引 | 下次同步退化为全量同步 |
| state vector | workspace.db | 同步索引 | 同上 |
| 文档元数据 | workspace.db | 索引缓存 | 从文件系统重建 |
| 权限记录 | workspace.db | 授权数据 | 需要重新被授权 |
| 设备身份 | ~/.swarmnote/ | 全局身份 | 不受工作区影响 |

### workspace.db 丢失恢复流程

```
应用启动 → 检测 workspace.db 不存在：
1. 创建新的 workspace.db
2. 扫描工作区目录下所有 .md 文件 → 重建文档索引
3. 为每个 .md 生成新的 UUID v7（旧 ID 丢失）
4. 连接 peer 时：
   a. 告知"我没有 state vector"
   b. Peer 发送完整 yjs state（全量同步）
   c. 用 peer 的 yjs state 替换本地从 .md 解析的内容
5. 权限数据：peer 的 workspace.db 中有本地的权限记录
   → peer 重新推送权限信息 → 恢复

注意：如果 db 丢失后用户离线编辑了 .md，重连时会产生 CRDT 合并
→ 可能有少量内容重复（.md 快照 + peer 的 yjs 历史合并）
→ 不丢数据，但需要用户手动清理重复部分
```
