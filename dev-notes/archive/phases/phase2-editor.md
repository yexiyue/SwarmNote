# Phase 2：Markdown 编辑器 + CRDT 协作

**目标**：集成 CodeMirror 6 Markdown 编辑器，基于 yrs/yjs 实现文档的实时协作同步，完成文档管理和协作感知。

**前置依赖**：Phase 1（P2P 网络层 + 文件同步）已完成。

**完成标志**：两台设备可以同时编辑同一篇 Markdown 文档，实时看到对方的编辑和光标，离线编辑后重连自动合并。

---

## 2.1 yrs 文档管理（Rust 后端）

**目标**：后端实现基于 yrs 的文档生命周期管理和持久化。

- [ ] 添加 yrs 依赖：`yrs = "0.21"`
- [ ] 定义 `documents` 表 Entity：
  ```rust
  pub struct Model {
      pub doc_id: String,            // UUID
      pub title: String,
      pub current_state: Vec<u8>,    // 合并压缩后的 yrs 状态
      pub state_vector: Vec<u8>,     // 用于同步的 StateVector
      pub created_at: i64,
      pub updated_at: i64,
  }
  ```
- [ ] 定义 `updates` 表 Entity（增量更新日志）：
  ```rust
  pub struct Model {
      pub id: i32,
      pub doc_id: String,
      pub update_data: Vec<u8>,      // 单个 yrs Update
      pub client_id: Option<i64>,
      pub created_at: i64,
  }
  ```
- [ ] 定义 `snapshots` 表 Entity（版本快照标记）：
  ```rust
  pub struct Model {
      pub id: i32,
      pub doc_id: String,
      pub snapshot_data: Vec<u8>,    // 编码的 Snapshot
      pub label: Option<String>,     // "手动保存"、"关闭文档" 等
      pub created_at: i64,
  }
  ```
- [ ] 编写 Migration
- [ ] 实现文档生命周期管理：
  - 创建文档：生成 UUID + 初始化空 yrs::Doc（`skip_gc: true`）
  - 加载文档：从 `current_state` BLOB 恢复 yrs::Doc
  - 保存文档：将 yrs::Doc 编码为 BLOB 存入 `current_state`
  - 删除文档：级联删除 updates + snapshots
- [ ] Update 追加存储：每个 yrs update 写入 `updates` 表
- [ ] 定期合并压缩：将 updates 合并到 `current_state`（减少加载时间）
- [ ] 实现 Tauri 命令：
  - `create_document(title)` → doc_id
  - `list_documents` → Vec<DocMeta>
  - `rename_document(doc_id, title)`
  - `delete_document(doc_id)`
  - `load_document(doc_id)` → 当前完整状态（Uint8Array）

**验证点**：创建文档、写入内容、关闭重启后内容完整恢复。

---

## 2.2 Tauri IPC 打通 yrs/yjs

**目标**：前端 yjs Doc 和后端 yrs Doc 通过 Tauri IPC 双向同步。

- [ ] 前端 → 后端：
  - `apply_update(doc_id, update: Uint8Array)` — 前端编辑产生的 yjs update 发送到后端
  - 后端收到后 apply 到 yrs::Doc 并持久化
- [ ] 后端 → 前端：
  - Tauri Event `doc-remote-update` — 推送从远程节点收到的 update
  - 前端收到后 apply 到 yjs Doc
- [ ] 编码格式：yjs/yrs v1 二进制编码，直接传 `Uint8Array`，零序列化开销
- [ ] 前端初始化流程：
  1. 调用 `load_document(doc_id)` 获取完整状态
  2. 创建 yjs Doc 并 apply 初始状态
  3. 注册 `doc.on('update', ...)` 监听 → 每次变更调用 `apply_update`
  4. 监听 `doc-remote-update` 事件 → apply 远程变更

**验证点**：前端编辑 → 后端持久化 → 重启后前端恢复完整内容。

---

## 2.3 GossipSub 增量同步

**目标**：文档编辑实时广播到所有在线协作者。

- [ ] 添加 GossipSub 到 `SwarmNoteBehaviour`
- [ ] Topic 命名规则：`/swarmnote/doc/<doc_id>`
- [ ] 打开文档时订阅对应 topic，关闭时取消订阅
- [ ] 本地 yrs update 产生后：
  1. 持久化到 SQLite
  2. 通过 GossipSub 广播到 topic
- [ ] 收到远程 GossipSub 消息后：
  1. Apply 到本地 yrs::Doc
  2. 持久化 update
  3. 通过 Tauri Event 推送到前端
- [ ] 去重处理：yrs 本身容忍重复 update，但可通过 StateVector 比较避免无效 apply

**验证点**：设备 A 和 B 同时打开同一文档，A 输入文字，B 实时看到。

---

## 2.4 Request-Response 全量同步

**目标**：新节点加入或重连后追赶文档状态。

- [ ] 扩展 Phase 1 的 Request-Response 协议，增加文档同步消息：
  ```rust
  enum SyncRequest {
      // ...Phase 1 的文件同步消息...

      /// 文档同步 Step 1：发送自身 StateVector
      DocSyncStep1 { doc_id: String, state_vector: Vec<u8> },
      /// 请求文档列表
      ListDocs,
  }

  enum SyncResponse {
      // ...Phase 1 的文件同步消息...

      /// 文档同步 Step 2：返回对方缺少的 Update
      DocSyncStep2 { doc_id: String, update: Vec<u8> },
      /// 文档列表
      DocList { docs: Vec<DocMeta> },
  }
  ```
- [ ] 同步流程：
  1. 新连接建立 → 发送 `ListDocs` 获取对方文档列表
  2. 对比本地文档列表，找出共同文档
  3. 对每个共同文档发送 `DocSyncStep1(state_vector)`
  4. 对方计算差异返回 `DocSyncStep2(update)`
  5. 双向执行（互相补齐）
  6. 全量同步完成后，切换到 GossipSub 增量模式
- [ ] 离线重连场景：重连后自动触发全量同步

**验证点**：设备 A 离线编辑文档，设备 B 也离线编辑。两者重新连接后，文档自动合并为一致状态。

---

## 2.5 CodeMirror 6 编辑器集成

**目标**：前端集成 Markdown 编辑器，绑定 yjs 协作。

- [ ] 安装前端依赖：
  ```bash
  pnpm add yjs y-codemirror.next lib0
  pnpm add @uiw/react-codemirror @codemirror/lang-markdown @codemirror/language-data
  ```
- [ ] 创建 `MarkdownEditor` React 组件：
  - 使用 `@uiw/react-codemirror` 封装
  - 配置 Markdown 语法高亮（`@codemirror/lang-markdown`）
  - 配置代码块内嵌语言高亮（`@codemirror/language-data`）
- [ ] yjs 绑定：
  - `yCollab` 扩展绑定 `Y.Text` 到 CodeMirror
  - Awareness provider 绑定（用于后续光标同步）
- [ ] 编辑器生命周期：
  1. 打开文档 → `load_document` 获取状态 → 创建 Y.Doc → apply 状态
  2. 编辑 → yjs update → `apply_update` 发送到后端
  3. 收到远程 update → apply 到 Y.Doc → CodeMirror 自动更新
  4. 关闭文档 → 清理 Y.Doc 和编辑器实例
- [ ] 快捷键支持：Markdown 常用快捷键（加粗、斜体、标题等）
- [ ] 图片支持：Markdown 图片语法 `![alt](file_id)` → 从本地资源库加载显示

**验证点**：打开文档，编辑 Markdown 内容，语法高亮正常，关闭重开内容不丢失。

---

## 2.6 协作感知（Awareness）

**目标**：实时显示协作者的光标、选区和在线状态。

- [ ] 后端实现 Awareness 状态同步：
  - Awareness 数据通过 GossipSub 广播（使用独立 topic：`/swarmnote/awareness/<doc_id>`）
  - 内容：用户昵称、颜色、光标位置、选区范围
- [ ] 前端通过 `y-codemirror.next` 的 `yCollab` 扩展显示：
  - 远程用户光标（带用户名标签）
  - 远程用户选区（半透明颜色高亮）
- [ ] 在线状态展示：
  - 编辑器右上角显示当前在线协作者列表
  - 每人一个颜色标记 + 昵称
- [ ] 用户信息来源：Phase 1.3 的设备信息 + 本地昵称/颜色设置

**验证点**：两台设备同时打开同一文档，能看到对方的光标位置和选区颜色。

---

## 2.7 文档管理 UI

**目标**：完整的文档列表和管理界面。

- [ ] 文档列表侧边栏：
  - 显示所有本地文档（标题 + 最后修改时间）
  - 新建文档按钮
  - 右键菜单：重命名、删除
- [ ] 文档排序：按修改时间（默认）、按标题
- [ ] 文档搜索框：
  - 标题模糊搜索（即时过滤）
  - 全文搜索（FTS5，回车触发）
- [ ] 全文搜索结果页：
  - 关键词高亮片段
  - 点击跳转到对应文档
- [ ] 空状态引导：无文档时显示"创建第一篇笔记"提示
- [ ] 自动保存指示：编辑器右下角显示"已保存" / "保存中..."

**验证点**：创建多篇文档，搜索关键词能找到对应文档，点击跳转正确打开。

---

## 2.8 版本历史

**目标**：查看和回退文档历史版本。

- [ ] 事件驱动快照：
  - 手动保存（Ctrl+S）→ 保存 Snapshot（label: "手动保存"）
  - 关闭文档 → Snapshot（label: "关闭文档"）
  - 全量同步完成 → Snapshot（label: "同步完成"）
  - 应用退出 → 对所有打开的文档保存 Snapshot
- [ ] 版本历史面板：
  - 时间线展示所有快照（时间 + 标签）
  - 点击快照 → 重建该时刻的文档状态 → 只读预览
- [ ] 版本对比：
  - 选中历史版本 vs 当前版本
  - 使用 `similar` crate 计算文本 diff
  - 前端展示增删行高亮
- [ ] 版本回退：
  - 从历史快照创建新的 update 覆盖当前状态
  - 回退操作本身也是一次编辑，可被撤销

**验证点**：编辑文档 → 手动保存 → 继续编辑 → 查看历史 → 看到两个版本 → 回退到之前版本。

---

## 2.9 导入导出

**目标**：支持与外部工具的数据互通。

- [ ] 导入 .md 文件：
  - 选择单个 .md 文件 → 创建新文档 → 写入内容
  - 自动提取文件名作为标题
- [ ] 批量导入 Obsidian vault：
  - 选择文件夹 → 递归扫描 .md 文件 → 批量创建文档
  - 保留文件名作为标题
  - `[[wikilink]]` 暂不处理（Phase 后续图谱功能处理）
- [ ] 导入 Notion 导出：
  - 支持 Notion "Export all as Markdown" 的 zip 文件
  - 解压后按 .md 文件逐个导入
- [ ] 导出单篇为 .md：
  - 从 yrs Doc 提取纯文本 → 写入 .md 文件
  - 系统文件选择对话框
- [ ] 导出单篇为 PDF：
  - Markdown → HTML → PDF（使用系统 print 或 headless 渲染）
- [ ] 批量导出：
  - 导出所有文档为 .md 文件到选定目录

**验证点**：导入一个 Obsidian vault（多个 .md 文件），所有文档出现在列表中且内容完整。导出后内容与编辑器一致。

---

## 执行顺序

```
2.1 yrs 文档管理 ──> 2.2 Tauri IPC ──> 2.5 CodeMirror 编辑器 ──> 2.7 文档管理 UI
                          │
                          ├──> 2.3 GossipSub 增量同步 ──> 2.6 协作感知
                          │
                          └──> 2.4 Request-Response 全量同步

2.8 版本历史（在 2.5 之后）
2.9 导入导出（在 2.7 之后）
```

- **2.1 → 2.2 → 2.5**：核心路径，先通后端再接前端
- **2.3 + 2.4**：在 IPC 通了之后并行开发
- **2.6**：在 GossipSub 基础上加 Awareness
- **2.7**：在编辑器可用后完善文档管理
- **2.8 + 2.9**：可与其他任务并行，优先级较低
