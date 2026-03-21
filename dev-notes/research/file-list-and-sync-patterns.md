# 文件列表展示与同步机制调研

> 调研日期：2026-03-18
> 目标：为 SwarmNote 的文件列表 UI 和 P2P 同步状态展示提供设计参考

---

## 一、主流应用文件列表展示模式

### 1.1 导航结构对比

| 应用 | 导航模式 | 组织方式 | 特点 |
|------|----------|----------|------|
| Notion | 左侧边栏树形 + 面包屑 | 页面嵌套（无限层级） | 固定宽度 224px，心理分组设计，可折叠 |
| Obsidian | 左侧文件树 + 插件扩展 | 文件夹 + 文件 | 插件可定制（File Explorer++、Note Count） |
| Apple Notes | 两/三栏布局 | 文件夹 + 智能文件夹 | 列表视图 + 画廊视图（缩略图），支持置顶 |
| Bear | 三栏布局（侧边栏 → 列表 → 编辑器） | 标签系统（#hashtag） | 极简美学，左滑/右滑手势导航，支持嵌套标签 |
| Logseq | 左侧边栏 + 右侧边栏 | 日记为中心 + 页面引用 | 右侧边栏用于研究参考，不可重排 |
| Roam Research | 双侧边栏 | 块引用 + 双向链接 | Shift+点击在侧边栏打开，拖拽嵌入块 |
| Google Docs | 扁平文件列表 + 文件夹 | 文件夹 + 标星 + 最近 | 网格/列表视图切换，搜索强大 |
| 飞书文档 | 主页 + 我的空间 + 知识库 | 空间 → 文件夹 → 文档 | 极简交互，功能按需出现，隐藏不必要元素 |
| 腾讯文档 | 扁平列表 | 文件夹 + 最近 + 收藏 | 微信/QQ 生态深度集成 |
| 语雀 | 知识库目录（书本式） | 知识库 → 文档目录 | 结构化目录呈现，适合技术文档和知识库 |

### 1.2 关键设计模式总结

#### A. 三栏布局（侧边栏 → 文件列表 → 编辑器）
- **代表**：Bear、Apple Notes、Obsidian
- **特点**：经典桌面应用模式，侧边栏负责一级分类（文件夹/标签），中间栏显示文件列表，右栏为编辑区
- **适用**：个人笔记应用，桌面端体验优秀
- **SwarmNote 参考价值**：**高** — 作为桌面笔记应用，三栏布局是最自然的选择

#### B. 树形导航侧边栏
- **代表**：Notion、Obsidian、飞书
- **特点**：无限嵌套层级，可折叠/展开，拖拽排序
- **适用**：需要深层组织结构的场景
- **SwarmNote 参考价值**：**中** — 初期可用扁平列表，后续演进为树形

#### C. 标签/双向链接组织
- **代表**：Bear（标签）、Logseq/Roam（双向链接）
- **特点**：一个文档可属于多个分类，无严格层级
- **适用**：知识管理、研究笔记
- **SwarmNote 参考价值**：**中低** — Phase 1 不需要，可作为后续特性

#### D. 搜索为中心
- **代表**：Google Docs、Apple Notes
- **特点**：搜索栏常驻顶部，支持全文搜索、OCR 识别
- **适用**：文档数量多的场景
- **SwarmNote 参考价值**：**高** — 搜索是所有笔记应用的基础功能

### 1.3 文件元数据展示

各应用在文件列表项中常见的元数据：

| 元数据 | 展示方式 | 常见应用 |
|--------|----------|----------|
| 标题 | 主要文字，加粗 | 全部 |
| 内容预览 | 标题下方 1-2 行灰色文字 | Apple Notes, Bear, Notion |
| 最后修改时间 | 相对时间（"3 分钟前"）或绝对时间 | 全部 |
| 协作者头像 | 小圆形头像叠加 | Notion, Google Docs, 飞书 |
| 同步状态图标 | 勾号/转圈/云图标 | Obsidian, Notion |
| 标签/分类 | 彩色小标签 | Bear, Notion |
| 置顶标记 | 图钉图标或置顶区域 | Apple Notes, Bear, Notion |
| 文档类型图标 | 文档/表格/画板图标 | 飞书, 腾讯文档, 语雀 |
| 共享状态 | 共享图标/人数 | Google Docs, 飞书 |

**SwarmNote 推荐**：Phase 1 展示 —— 标题、内容预览、最后修改时间、同步状态图标。

---

## 二、文件同步实现模式

### 2.1 各应用同步状态指示器

#### Obsidian Sync
- **位置**：状态栏右下角
- **图标状态**：
  - 绿色勾号 ✓：已完全同步
  - 旋转图标：正在同步
  - 错误图标：同步失败
- **日志系统**：可查看同步活动日志，分类过滤（全部、错误、跳过、合并冲突）
- **已知问题**：移动端同步图标不够显眼，用户经常不确定同步是否完成
- **暂停功能**：支持暂停同步，暂停时本地更改累积但不上传

#### Notion
- **位置**：页面顶部面包屑右侧
- **状态文字**："Saving..." / "All changes saved" / 上次同步时间
- **离线模式**：
  - 可下载页面供离线使用（进度条显示下载进度）
  - 离线编辑自动保存到本地
  - 重新上线后自动同步，顶部显示同步状态
  - 设置中有专门的"离线"标签页，管理离线可用页面
- **冲突处理**：使用自研 CRDT 系统减少冲突，但非文本属性（如 select 字段）仍可能冲突

#### Google Docs
- **位置**：菜单栏区域
- **状态文字**："正在保存..." / "所有更改已保存到云端硬盘"
- **极简设计**：用户几乎不需要关心同步——始终在线是默认假设
- **离线编辑**：通过 Chrome 扩展支持离线，重新连接后自动同步

#### 飞书文档
- **特点**：云端唯一版本，实时同步
- **协同编辑**：多人同时编辑时可看到其他人的光标和选区
- **通知机制**：文档内 @提及、评论回复直接推送到飞书消息
- **局限**：强依赖飞书平台，离线能力有限

#### 腾讯文档
- **冲突处理**：同一位置以最后操作者为准（Last-Write-Wins）
- **实时性**：多人可同时编辑同一位置，但不能实时看到别人修改（相比 Google Docs 弱）
- **多端入口**：QQ/微信/浏览器/小程序/APP

#### Syncthing（P2P 参考）
- **Web GUI 状态展示**：
  - **Global State**：完全同步后文件夹应有的总大小
  - **Local State**：本地当前实际大小
  - **Out of Sync**：需要从其他设备同步的数据量
  - 按文件夹展示同步状态
- **设备状态**：显示每个连接设备的连接状态、同步进度
- **接收端模式**：receive-only 文件夹有本地修改时显示红色"Revert Local Changes"按钮
- **不足**：扫描阶段无进度百分比显示

### 2.2 同步状态设计模式总结

```
同步状态状态机：

  [离线] ──连接成功──→ [已连接]
    ↑                      │
    │                   检测到变更
    │                      ↓
  连接丢失          [正在同步]
    ↑                 ╱       ╲
    │            成功╱         ╲失败
    │              ↓            ↓
    └─────── [已同步]    [同步失败/冲突]
                                │
                            用户解决
                                ↓
                           [已同步]
```

**推荐的同步状态集合**（适用于 SwarmNote）：

| 状态 | 图标 | 含义 |
|------|------|------|
| `synced` | ✓ 绿色勾号 | 本地版本与已知最新版本一致 |
| `syncing` | 旋转箭头 | 正在从/向对等节点同步 |
| `pending` | 上箭头 + 时钟 | 有本地更改等待同步（无可用 peer） |
| `conflict` | ⚠ 黄色警告 | 检测到冲突，需用户介入 |
| `offline` | 云 + 斜线 | 无网络连接 |
| `error` | ✗ 红色叉号 | 同步错误 |

### 2.3 CRDT 同步技术分析

#### Yjs vs Automerge

| 维度 | Yjs | Automerge |
|------|-----|-----------|
| 数据模型 | 共享类型（YMap, YArray, YText） | JSON-like 文档 |
| 学习曲线 | 需学习 Yjs 特定 API | 类似操作普通 JS 对象 |
| 编辑器支持 | 丰富（ProseMirror, CodeMirror, BlockNote） | 较少 |
| 性能 | 成熟稳定 | 新 Rust 实现大幅改善 |
| 网络无关 | 是 | 是 |
| 语言支持 | JS 为主，有 Rust 绑定（y-crdt） | 多语言（JS, Rust, Go, Python） |
| 文件大小 | 会增长，需定期压缩 | 会增长，需定期压缩 |

#### 冲突解决 UI 模式

学术研究（Almishev, 2025）提出三层架构：

1. **数据层（自动合并）**：CRDT 自动处理大部分并发操作合并，用户无感知
2. **逻辑层（语义冲突检测）**：识别 CRDT 无法自动解决的语义冲突（如同一字段被赋予互斥值）
3. **表示层（用户解决）**：通过 UI 展示冲突并让用户选择

**常见冲突解决 UI 模式**：
- **并排对比**：显示两个版本让用户选择（类似 Git merge）
- **内联标注**：在文档中直接标注冲突区域（类似 Google Docs 建议模式）
- **版本历史**：让用户浏览历史版本并恢复
- **Last-Write-Wins 自动解决**：腾讯文档方式，最后操作者为准
- **Toast 通知**：简单告知"已自动合并来自其他设备的更改"

**SwarmNote 推荐策略**：
- Phase 2 使用 Yjs（BlockNote 内置支持），CRDT 自动合并绝大多数编辑冲突
- Rust 端透传 Yjs 二进制更新，不需要理解内容
- 对于无法自动合并的冲突，先用简单的 Toast 通知 + 版本历史回退

---

## 三、P2P 特有的同步挑战

### 3.1 部分可用性（Partial Availability）

P2P 环境下不像 C/S 架构有"总是在线"的服务器，核心挑战包括：

#### 问题
- 不是所有 peer 都在线，数据可能分散在不同设备上
- 没有中央服务器充当"最终真相源"
- 网络拓扑动态变化（设备随时上下线）

#### Syncthing 的解决方案
- 显示 Global State（全局应有状态）vs Local State（本地实际状态）的差异
- 文件级别追踪同步状态（哪些文件从哪些设备同步）
- 支持 Relay 中继——当两个设备不能直接连接时通过中继传输

#### 推荐的 SwarmNote 方案

```
┌─────────────────────────────────────────┐
│          SwarmNote 同步模型              │
├─────────────────────────────────────────┤
│                                         │
│  本地 SQLite ←→ CRDT State ←→ P2P Sync │
│     (持久化)    (内存/yjs)    (swarm-p2p)│
│                                         │
│  每个文档维护：                           │
│  - 本地版本向量 (version vector)          │
│  - 已知 peer 的版本信息                   │
│  - 同步队列（待发送的更新）                │
│  - 最后同步时间戳                         │
│                                         │
└─────────────────────────────────────────┘
```

### 3.2 离线优先同步模式

#### 核心原则
1. **本地数据库是唯一真相源**：所有操作先写入本地 SQLite，UI 立即更新
2. **网络只是同步机制**：网络可用时同步，不可用时正常工作
3. **待同步写入队列**：修改标记为 `pending_sync`，连接到 peer 后逐个发送

#### 推荐的 Pending Changes 管理

```rust
// 每个文档的同步元数据
struct DocSyncMeta {
    doc_id: String,
    local_version: u64,          // 本地版本号
    last_synced_version: u64,    // 上次成功同步的版本
    pending_updates: Vec<Vec<u8>>, // 待发送的 yjs 更新
    sync_status: SyncStatus,     // synced | pending | syncing | conflict
    last_sync_time: Option<DateTime>,
    known_peers: HashMap<PeerId, u64>, // peer -> 已知版本
}
```

#### UI 层面的离线优先设计
- **立即响应**：所有操作立即反映在 UI 上，不等待网络确认
- **后台同步**：连接可用时自动在后台同步
- **状态透明**：用户随时能看到哪些更改还没同步
- **优雅降级**：无网络时隐藏协作功能，保留全部本地编辑能力

### 3.3 P2P 同步进度展示

参考 Syncthing 的设计，SwarmNote 可以在 UI 中展示：

#### 全局状态栏（底部/状态栏）
```
[🟢 已连接 2 个设备] [↑ 3 待同步 | ↓ 正在接收 2 个文档]
```

#### 文件列表中的同步状态
```
📄 会议记录 2026-03           ✓ 已同步      3分钟前
📄 项目设计文档               ↑ 待同步      刚刚修改
📄 读书笔记                   ⟳ 正在同步    从 MacBook 同步中
📄 API 设计草稿               ⚠ 冲突        需要解决
```

#### 设备面板（可折叠）
```
已连接设备：
  💻 MacBook Pro    [已同步]  最后同步: 2分钟前
  📱 iPhone 15      [同步中]  3/12 文档已同步

离线设备：
  🖥 台式机          最后在线: 2天前  有 5 个未同步更改
```

### 3.4 与 swarm-p2p-core 的集成方案

基于 CLAUDE.md 中描述的 swarm-p2p-core API，同步流程：

1. **设备发现**：mDNS（局域网）+ Kademlia DHT（跨网络），`PeersDiscovered` / `PeerConnected` 事件
2. **文档同步协议**：定义 `AppRequest` / `AppResponse` 枚举

```rust
enum SyncRequest {
    // 询问对方有哪些文档的哪些版本
    GetVersionVector,
    // 请求特定文档的更新
    GetDocUpdates { doc_id: String, since_version: u64 },
    // 推送本地更新给对方
    PushDocUpdates { doc_id: String, updates: Vec<Vec<u8>> },
}

enum SyncResponse {
    VersionVector(HashMap<String, u64>),
    DocUpdates { doc_id: String, updates: Vec<Vec<u8>> },
    Ack,
}
```

3. **同步触发时机**：
   - 新 peer 连接时：交换 version vector，同步差异
   - 本地文档修改时：向所有在线 peer 推送更新
   - 定时轮询：防止遗漏

---

## 四、SwarmNote Phase 1 设计建议

### 4.1 文件列表 UI

- 采用 **三栏布局**（侧边栏 → 文件列表 → 编辑器），参考 Bear / Apple Notes
- 侧边栏：全部笔记、最近编辑、收藏/置顶
- 文件列表项：标题 + 内容预览（1行）+ 修改时间 + 同步状态小图标
- 排序：默认按最后修改时间倒序
- 搜索栏常驻顶部

### 4.2 同步状态展示

- 全局同步状态在底部状态栏：连接设备数 + 待同步数
- 每个文档在列表中显示小图标：synced / pending / syncing
- 详细同步日志在设置/调试面板中（参考 Obsidian Sync 的日志设计）

### 4.3 技术实现优先级

1. 本地 SQLite 存储 + 文件列表 CRUD
2. swarm-p2p-core 集成 + 设备发现
3. 基于 version vector 的文档同步协议
4. 同步状态追踪和 UI 展示
5. Phase 2 再引入 Yjs/CRDT 做细粒度编辑同步

---

## 参考来源

- [Notion Sidebar UI Breakdown](https://medium.com/@quickmasum/ui-breakdown-of-notions-sidebar-2121364ec78d)
- [Bear vs Apple Notes](https://clickup.com/blog/bear-vs-apple-notes/)
- [Logseq vs Roam Research](https://travischan.medium.com/logseq-vs-roam-research-65168fe57464)
- [飞书文档体验设计拆解](https://www.woshipm.com/pd/5595076.html)
- [协同办公笔记软件综合评测](https://zhuanlan.zhihu.com/p/498709185)
- [Obsidian Sync Status Icons](https://help.obsidian.md/sync/messages)
- [Notion Offline Guide](https://www.notion.com/help/guides/working-offline-in-notion-everything-you-need-to-know)
- [Syncthing GUI Documentation](https://docs.syncthing.net/intro/gui.html)
- [CRDT Libraries Guide 2025](https://velt.dev/blog/best-crdt-libraries-real-time-data-sync)
- [CRDT Implementation Guide](https://velt.dev/blog/crdt-implementation-guide-conflict-free-apps)
- [Offline-First Architecture Guide](https://www.droidcon.com/2025/12/16/the-complete-guide-to-offline-first-architecture-in-android/)
- [awesome-local-first](https://github.com/alexanderop/awesome-local-first)
- [在线协作文档综合评测](https://zhuanlan.zhihu.com/p/498710837)
- [Cinapse: Why We Moved Away from CRDTs](https://www.powersync.com/blog/why-cinapse-moved-away-from-crdts-for-sync)
