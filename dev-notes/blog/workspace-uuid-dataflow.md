# Workspace 架构与文档 UUID 数据流转

> SwarmNote 是一个 P2P 笔记同步工具，两台设备之间不经过任何服务器，直接同步笔记。要实现这个目标，首先要解决一个基础问题：**设备 A 和设备 B 怎么知道 `notes/todo.md` 是"同一个文档"？** 本文从这个问题出发，讲清楚 SwarmNote 的 Workspace 双数据库架构、文档 UUID 的完整生命周期，以及为什么这样设计。

## 1. 为什么需要全局 UUID？

两台电脑各自打开 `notes/todo.md`——路径一样，但它们真的是"同一个文档"吗？

```mermaid
graph LR
    subgraph "设备 A"
        A_FILE["notes/todo.md"]
        A_UUID["UUID: ???"]
    end
    subgraph "设备 B"
        B_FILE["notes/todo.md"]
        B_UUID["UUID: ???"]
    end

    A_FILE -.-|"路径相同<br/>但如何确认同一个？"| B_FILE

    style A_FILE fill:#fff3e0,stroke:#ff9800
    style B_FILE fill:#fff3e0,stroke:#ff9800
    style A_UUID fill:#ffebee,stroke:#f44336
    style B_UUID fill:#ffebee,stroke:#f44336
```

**路径不能作为唯一标识**，因为：
- 文件可以重命名——rename 后路径变了，但它仍是同一个文档
- 不同工作区可能有同名文件——`work/notes/todo.md` 和 `personal/notes/todo.md` 是不同文档
- P2P 同步需要一个不随路径变化的、跨设备稳定的身份

所以，**每个文档在创建时就分配一个全局唯一的 UUID（v7，时间有序）**，这个 UUID 才是文档的真实身份。

## 2. 双数据库架构

SwarmNote 采用双数据库设计：一个全局的，一个每个工作区独立的。

```mermaid
graph TB
    subgraph "全局 (App Data 目录)"
        DEVICES_DB["devices.db<br/>━━━━━━━━━━━━<br/>paired_devices 表<br/>存储配对设备信息"]
    end

    subgraph "工作区 A (~/notes/.swarmnote/)"
        WS_JSON_A["workspace.json<br/>━━━━━━━━━━━━<br/>uuid · name · created_at<br/><b>工作区 UUID 源头</b>"]
        WS_DB_A["workspace.db<br/>━━━━━━━━━━━━<br/>workspaces · documents<br/>folders · deletion_log<br/>doc_chunks · ..."]
    end

    subgraph "工作区 B (~/work/.swarmnote/)"
        WS_JSON_B["workspace.json"]
        WS_DB_B["workspace.db"]
    end

    DEVICES_DB ---|"共享设备信任关系"| WS_DB_A
    DEVICES_DB ---|"共享设备信任关系"| WS_DB_B

    style DEVICES_DB fill:#e3f2fd,stroke:#1976d2
    style WS_JSON_A fill:#e8f5e9,stroke:#388e3c
    style WS_DB_A fill:#e8f5e9,stroke:#388e3c
    style WS_JSON_B fill:#f3e5f5,stroke:#7b1fa2
    style WS_DB_B fill:#f3e5f5,stroke:#7b1fa2
```

**为什么分两个 DB？**

| 设计决策 | 理由 |
|----------|------|
| `devices.db` 全局 | 配对设备是跨工作区的——你信任一台设备，不是信任某个工作区 |
| `workspace.db` per-workspace | 文档数据跟着工作区走，拷贝文件夹 = 拷贝数据，符合 local-first 原则 |
| `workspace.json` 而非 DB 列 | 工作区 UUID 是同步协议的先决条件——peer 需要在连接 DB 之前就知道 UUID |

### 多窗口状态管理

SwarmNote 支持多窗口，每个窗口绑定一个工作区。后端用 `RwLock<HashMap<String, DatabaseConnection>>` 管理连接：

```mermaid
graph LR
    subgraph "Tauri 后端"
        DBSTATE["DbState<br/>RwLock&lt;HashMap&gt;"]
        WSSTATE["WorkspaceState<br/>per-window info"]
    end

    W1["窗口 ws-a1b2c3<br/>~/notes"] --> DBSTATE
    W2["窗口 ws-d4e5f6<br/>~/work"] --> DBSTATE

    DBSTATE --> DB1["~/notes/.swarmnote/workspace.db"]
    DBSTATE --> DB2["~/work/.swarmnote/workspace.db"]

    style DBSTATE fill:#fff8e1,stroke:#f9a825
    style WSSTATE fill:#fff8e1,stroke:#f9a825
```

窗口 label 由工作区路径的 hash 生成（`ws-{hash16}`），保证同一路径不会打开两个窗口。

## 3. 文档 UUID 的完整生命周期

一个文档从诞生到删除，UUID 经历 5 个阶段：

```mermaid
flowchart TD
    CREATE["① 创建<br/>前端 createFile() 或外部拖入 .md"]
    RECONCILE["② 对账<br/>工作区打开时 reconcile_with_db()"]
    OPEN["③ 打开<br/>open_doc() 加载 Y.Doc"]
    RENAME["④ 重命名<br/>rename_document()"]
    DELETE["⑤ 删除<br/>delete_document_by_rel_path()"]

    CREATE -->|"分配 Uuid::now_v7()"| RECONCILE
    RECONCILE -->|"补全缺失记录"| OPEN
    OPEN -->|"INSERT OR IGNORE + SELECT"| RENAME
    RENAME -->|"UUID 不变<br/>只更新 rel_path"| DELETE
    DELETE -->|"写 deletion_log 墓碑"| TOMBSTONE["墓碑持久化<br/>防止 sync 复活"]

    style CREATE fill:#c8e6c9,stroke:#2e7d32
    style RECONCILE fill:#bbdefb,stroke:#1565c0
    style OPEN fill:#fff9c4,stroke:#f9a825
    style RENAME fill:#ffe0b2,stroke:#ef6c00
    style DELETE fill:#ffcdd2,stroke:#c62828
    style TOMBSTONE fill:#f5f5f5,stroke:#616161
```

### ① 创建：UUID 在后端生成

前端调用 `createFile` 时，**不传 `id`**——后端生成 `Uuid::now_v7()` 并立即写入 DB。

```text
前端                              Rust 后端
  │                                  │
  ├─ fsCreateFile("notes","todo") ──→│  创建 .md 文件
  │                                  │
  ├─ upsertDocument({               │
  │    workspace_id,                 │
  │    title: "todo",         ──────→│  id 为空 → 生成 Uuid::now_v7()
  │    rel_path: "notes/todo.md"     │  INSERT documents 表
  │  })                              │
  │                                  │
  │←── DocumentModel { id: "019d..." │  返回带稳定 UUID 的记录
```

**关键设计**：UUID 由后端统一生成，前端永远不传 `id`。这避免了前端传路径当 UUID 的 bug。

### ② 对账：工作区打开时补全

用户可能通过文件管理器拷贝 `.md` 文件到工作区目录。这些文件在 DB 中没有记录。`reconcile_with_db` 在每次打开工作区时运行：

```mermaid
flowchart LR
    SCAN["扫描磁盘<br/>收集所有 .md 路径"]
    DB["查询 DB<br/>已有 rel_path 集合"]
    DIFF["计算差集<br/>磁盘有 DB 无"]
    INSERT["INSERT OR IGNORE<br/>分配新 UUID"]

    SCAN --> DIFF
    DB --> DIFF
    DIFF --> INSERT

    style SCAN fill:#e1f5fe,stroke:#0288d1
    style DB fill:#e1f5fe,stroke:#0288d1
    style DIFF fill:#fff9c4,stroke:#f9a825
    style INSERT fill:#c8e6c9,stroke:#2e7d32
```

**为什么用 `INSERT OR IGNORE`？** UNIQUE 约束 `(workspace_id, rel_path)` 保证即使并发调用（例如窗口快速重开），也不会产生重复记录。

**为什么"DB 有但磁盘无"不删除？** 文件可能是被移动了（rename = 旧路径消失 + 新路径出现）。孤儿记录留给 tombstone GC 处理。

### ③ 打开：并发安全的 upsert

`open_doc` 是文档生命周期的核心。它负责：加载 Y.Doc → 返回稳定 UUID → 启动 writeback 任务。

```mermaid
sequenceDiagram
    participant FE as 前端
    participant MGR as YDocManager
    participant DB as SQLite

    FE->>MGR: open_ydoc("notes/todo.md")

    Note over MGR: 内存快速路径：已打开？
    alt 已在内存中
        MGR-->>FE: {doc_uuid, yjs_state}
    else 未打开
        MGR->>DB: INSERT OR IGNORE<br/>(workspace_id, rel_path)
        Note over DB: UNIQUE 约束去重
        MGR->>DB: SELECT WHERE rel_path = ?
        DB-->>MGR: {id: "019d...", yjs_state: ...}

        alt DB 有 yjs_state
            Note over MGR: 从 yjs_state 恢复 Y.Doc
        else DB 无 yjs_state
            Note over MGR: 从 .md 文件解析<br/>markdown → Y.Doc
        end

        MGR->>DB: persist_snapshot<br/>(yjs_state + state_vector + .md)
        Note over MGR: 启动 writeback task<br/>(500ms poll, 1.5s debounce)
        MGR-->>FE: {doc_uuid: "019d...", yjs_state}
    end
```

**`INSERT OR IGNORE` + `SELECT` 模式**的意义：即使两个并发调用同时发现"DB 无记录"并尝试 INSERT，UNIQUE 约束保证只有一条记录存活。后续 SELECT 取到同一条记录，两个调用返回相同 UUID。

### ④ 重命名：UUID 不变

```text
rename_document("notes/todo.md" → "notes/done.md")
  ├─ DB: UPDATE documents SET rel_path = "notes/done.md" WHERE rel_path = "notes/todo.md"
  ├─ 内存: YDocManager.rename_doc(uuid, "notes/done.md")
  └─ UUID 始终不变 → 对端同步时能通过 UUID 识别"同一个文档换了路径"
```

### ⑤ 删除：墓碑机制

删除不是简单的 `DELETE FROM documents`。为了防止 sync 时被远端"复活"，需要留下墓碑：

```mermaid
flowchart LR
    DEL["delete_document_by_rel_path"]
    FIND["查找文档<br/>按 rel_path"]
    TOMB["写 deletion_log<br/>doc_id + deleted_at<br/>+ lamport_clock + 1"]
    RM["DELETE FROM documents"]

    DEL --> FIND --> TOMB --> RM

    style TOMB fill:#ffcdd2,stroke:#c62828
```

**目录删除级联**：删除目录时，先 `delete_documents_by_prefix(dir_path + "/")` 为所有子文档写墓碑，再删除磁盘文件。

## 4. Workspace UUID：跨设备工作区匹配

文档 UUID 解决了"同一个文档"的识别，但还有一个问题：**设备 A 的 `~/notes/` 和设备 B 的 `~/my-notes/` 是"同一个工作区"吗？**

答案同样是 UUID，但存储在 `workspace.json` 而非 DB：

```json
// ~/notes/.swarmnote/workspace.json
{
  "uuid": "019d3cd7-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "name": "My Notes",
  "created_at": "2026-03-20T10:00:00Z"
}
```

**三级优先级**：

```mermaid
flowchart TD
    START["打开工作区"]
    READ["读 workspace.json"]
    HAS_FILE{文件存在？}
    HAS_DB{DB 有记录？}
    USE_FILE["使用 workspace.json 的 UUID"]
    USE_DB["使用 DB 的 UUID<br/>写入 workspace.json"]
    GEN_NEW["生成 Uuid::now_v7()<br/>写入 workspace.json"]
    SYNC_DB["同步到 DB<br/>workspaces.id = UUID"]

    START --> READ --> HAS_FILE
    HAS_FILE -->|是| USE_FILE --> SYNC_DB
    HAS_FILE -->|否| HAS_DB
    HAS_DB -->|是| USE_DB --> SYNC_DB
    HAS_DB -->|否| GEN_NEW --> SYNC_DB

    style USE_FILE fill:#c8e6c9,stroke:#2e7d32
    style USE_DB fill:#fff9c4,stroke:#f9a825
    style GEN_NEW fill:#bbdefb,stroke:#1565c0
```

**为什么用文件而非 DB？**
- Peer 在连接时需要先交换 workspace UUID 来匹配工作区——此时可能还没打开 DB
- JSON 文件可以被其他工具读取（debug 友好）
- 解决了"先有 DB 还是先有 UUID"的鸡蛋问题

## 5. 全量同步设计

有了稳定的文档 UUID 和工作区 UUID，P2P 同步就有了基础。以下是全量同步的完整流程：

### 5.1 DocMeta 交换

两台设备配对连接后，先交换文档元数据：

```mermaid
sequenceDiagram
    participant A as 设备 A
    participant B as 设备 B

    Note over A,B: PeerConnected 事件触发

    A->>B: SyncRequest::DocList
    B-->>A: SyncResponse::DocList { docs: [DocMeta...] }

    Note over A: 对比本地 DocMeta 列表

    rect rgb(232, 245, 233)
        Note over A: 分类处理
        Note over A: 1. 双方都有 → 按 UUID 匹配<br/>比较 state_vector 决定谁缺更新
        Note over A: 2. A 有 B 无 → A 推送给 B
        Note over A: 3. B 有 A 无 → 检查 deletion_log
        Note over A: 4. B 标记 deleted → 检查 lamport_clock
    end
```

`DocMeta` 结构携带了做决策所需的全部信息：

```rust
pub struct DocMeta {
    pub doc_id: Uuid,            // 文档全局 UUID
    pub rel_path: String,         // 相对路径（首次同步时用于 claim）
    pub title: String,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,  // None = 活跃, Some = 已删除（墓碑）
    pub lamport_clock: i64,       // 单调递增版本号
    pub workspace_uuid: Uuid,     // 所属工作区 UUID
}
```

### 5.2 State Vector 交换

确定哪些文档需要同步后，逐文档交换 yjs state vector：

```mermaid
sequenceDiagram
    participant A as 设备 A
    participant B as 设备 B

    Note over A,B: 文档 "019d..." 需要同步

    A->>B: StateVector { doc_id, sv_a }
    Note over B: 计算 A 缺少的 updates
    B-->>A: Updates { doc_id, missing_for_a }

    B->>A: StateVector { doc_id, sv_b }
    Note over A: 计算 B 缺少的 updates
    A-->>B: Updates { doc_id, missing_for_b }

    Note over A: apply(missing_for_a) → CRDT 自动合并
    Note over B: apply(missing_for_b) → CRDT 自动合并
```

**同步优先级**：
1. **P0** — 当前打开的文档（亚秒级）
2. **P1** — 最近编辑的文档（按 `updated_at` 降序）
3. **P2** — 其余文档（后台追赶）

### 5.3 离线合并场景

离线合并不需要特殊处理——它就是全量同步的一个实例：

```mermaid
graph TD
    subgraph "离线期间"
        A_EDIT["设备 A 编辑了<br/>doc1, doc2, doc3"]
        B_EDIT["设备 B 编辑了<br/>doc1, doc4"]
    end

    RECONNECT["重新连接<br/>PeerConnected 事件"]

    subgraph "全量同步"
        EXCHANGE["DocMeta 交换"]
        SV["State Vector 交换<br/>（双向）"]
        MERGE["yjs CRDT 自动合并<br/>无冲突"]
    end

    A_EDIT --> RECONNECT
    B_EDIT --> RECONNECT
    RECONNECT --> EXCHANGE --> SV --> MERGE

    style RECONNECT fill:#fff9c4,stroke:#f9a825
    style MERGE fill:#c8e6c9,stroke:#2e7d32
```

yjs 的 CRDT 特性保证：**无论两台设备在离线期间做了什么编辑，重连后的合并都是自动、无冲突的。**

### 5.4 墓碑同步：防复活

```mermaid
flowchart TD
    B_HAS["设备 B 发来<br/>DocMeta { deleted_at: Some(...) }"]
    A_CHECK{"设备 A 有这个文档？"}

    A_HAS_DOC["比较 lamport_clock"]
    CLOCK_CMP{"B.clock > A.clock ?"}

    APPLY_DEL["接受删除<br/>本地也删除 + 写墓碑"]
    KEEP["保留本地版本<br/>（删后又编辑的场景）"]
    IGNORE["忽略<br/>从未有过此文档"]

    B_HAS --> A_CHECK
    A_CHECK -->|有| A_HAS_DOC --> CLOCK_CMP
    A_CHECK -->|无| IGNORE
    CLOCK_CMP -->|是| APPLY_DEL
    CLOCK_CMP -->|否| KEEP

    style APPLY_DEL fill:#ffcdd2,stroke:#c62828
    style KEEP fill:#c8e6c9,stroke:#2e7d32
    style IGNORE fill:#f5f5f5,stroke:#9e9e9e
```

## 6. 数据完整性保障

整个系统在数据层面有 4 道防线：

| 防线 | 机制 | 保护什么 |
|------|------|----------|
| **UNIQUE 约束** | `UNIQUE(workspace_id, rel_path)` | 同一路径不会产生两条记录 |
| **INSERT OR IGNORE** | `on_conflict(...).do_nothing()` | 并发 upsert 不竞态 |
| **墓碑 deletion_log** | 删除时写墓碑，sync 时比较 clock | 已删文档不被复活 |
| **workspace.json** | 文件级 UUID 源头 | 工作区身份跨设备稳定 |

这些机制组合在一起，确保了一个核心属性：**文档的 UUID 从创建到删除，始终唯一、稳定、跨设备一致。** 这是 P2P 同步能正确工作的数据基础。
