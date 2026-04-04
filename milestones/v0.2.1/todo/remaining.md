# v0.2.1 遗留问题清单

> 生成日期：2026-04-04  
> 已关闭 issue 中遗留的小问题 + 未关闭 issue 的完成情况

---

## #45 sidebar: 侧边栏同步状态增强（已关闭，2 处遗留）

**文件**：`src/components/layout/SyncStatusBar.tsx`

### 1. "未连接"状态标签错误（bug）

- **现象**：节点运行中（`nodeStatus === "running"`）但连接设备数为 0 时，显示「已连接」（绿点），应显示「未连接」
- **位置**：`SyncStatusBar.tsx:32-33`
- **修复**：将该分支的 `label` 改为 `t\`未连接\``，`dotClass` 改为灰色（`bg-gray-400`）

### 2. 缺少 hover tooltip

- **现象**：验收标准要求 hover 时显示详细信息 tooltip，当前无任何 tooltip
- **修复**：在 `<button>` 上包裹 shadcn `<Tooltip>`，内容显示节点状态、在线设备数、工作区同步进度等详细信息

---

## #47 auto-update: 桌面端自动更新（未关闭，基础设施未配置）

**状态**：前端 UI 100% 完成，后端/CI 未就绪，**issue 保持 OPEN**

**已完成**：
- ✅ 启动 3 秒后自动检查更新（`__root.tsx:41-44`）
- ✅ `ForceUpdateDialog`（不可关闭，自动下载进度）
- ✅ `PromptUpdateDialog`（可关闭，稍后提醒 / 立即更新）
- ✅ 下载完成后执行 `relaunch()`
- ✅ 设置 > 关于页面「检查更新」按钮 + 进度展示
- ✅ 已是最新 / 更新失败 等状态文案
- ✅ `tauri.conf.json` 已配置 UpgradeLink + GitHub Releases 双端点

**待完成**：

### 1. minisign 签名密钥未配置

- **位置**：`src-tauri/tauri.conf.json` → `updater.pubkey`
- **现状**：值为 `"TODO_REPLACE_WITH_MINISIGN_PUBLIC_KEY"`，导致所有平台无法验证更新包签名，更新会失败
- **修复**：生成 minisign 密钥对，将公钥填入 `tauri.conf.json`，将私钥配置为 GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY`

### 2. CI 构建产物未签名

- **位置**：`.github/workflows/`（需新增或修改 release workflow）
- **现状**：没有 release CI，构建产物未自动签名上传到 GitHub Releases
- **修复**：添加 release workflow，在构建时注入 `TAURI_SIGNING_PRIVATE_KEY`，将各平台产物上传至 GitHub Release

### 3. UpgradeLink 端点 Key 未配置

- **位置**：`src-tauri/tauri.conf.json` → `updater.endpoints[0]`
- **现状**：URL 中 `tauriKey=TODO_REPLACE_WITH_TAURI_KEY` 为占位符
- **修复**：填入真实的 UpgradeLink API Key（或直接删除该端点，仅使用 GitHub Releases）

---

## #49 onboarding: 引导页 P2P 流程改造（已关闭，1 处遗留）

**文件**：`src/components/onboarding/OnboardingLayout.tsx`

### 步骤指示器未根据路径动态调整

- **现象**：`StepIndicator` 始终显示 `TOTAL_STEPS`（5 个）圆点。选择「全新开始」跳过 PairingStep 后，步骤实际只有 4 步，但指示器仍显示 5 个点，第 4 个点（PairingStep）对用户无意义
- **位置**：`OnboardingLayout.tsx:12`
- **修复**：从 `onboardingStore` 读取 `userPath`，当 `userPath === "new"` 时将总步数显示为 4（过滤掉 PairingStep 对应的点），并相应调整当前步序号

---

## #50 workspace-picker: Workspace Picker 同步入口（已关闭，2 处遗留）

**文件**：`src/components/workspace/WorkspaceSyncDialog.tsx`

### 1. 已同步工作区 checkbox 未禁用

- **现象**：`ws.isLocal === true` 的工作区会显示「已同步」badge，但 checkbox 没有 `disabled` 属性，用户仍可勾选并重复触发同步
- **位置**：`WorkspaceSyncDialog.tsx:338-361`（`<label>` 内的 `<input type="checkbox">`）
- **修复**：为 `ws.isLocal` 的条目添加 `disabled` 和视觉灰化样式，阻止选中

### 2. 同步完成后未自动打开第一个工作区

- **现象**：验收标准要求「同步完成后自动打开第一个工作区」，但当前 done 阶段只显示各工作区的「打开」按钮，需要手动点击
- **位置**：`WorkspaceSyncDialog.tsx:244`（`setPhase("done")` 之后）
- **修复**：在 `handleStartSync` 结束后，自动调用 `handleOpenSyncedWorkspace` 打开第一个成功同步的工作区（`status === "done"` 的第一项）
