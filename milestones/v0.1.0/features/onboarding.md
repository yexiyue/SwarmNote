# Onboarding 引导流程

## 用户故事

作为首次使用 SwarmNote 的用户，我希望通过简单的引导流程完成初始设置，以便快速开始使用。

## 需求描述

首次启动应用时，展示多步骤引导流程，帮助用户完成工作区创建、设备命名和身份生成。后续启动直接进入主界面。

## 交互设计

### 用户操作流程

1. **Step 1 - 欢迎页**：展示 SwarmNote Logo 和简短介绍，"开始使用" 按钮
2. **Step 2 - 选择工作区目录**：调用系统文件夹选择对话框，用户选择或新建一个目录作为工作区根目录
3. **Step 3 - 设备名称**：输入设备名称（默认取系统主机名），用于后续 P2P 设备识别
4. **Step 4 - 完成**：展示配置摘要（工作区路径、设备名、PeerId 前 8 位），"进入 SwarmNote" 按钮

### 关键页面 / 组件

- `OnboardingWelcome` — 欢迎页
- `OnboardingWorkspace` — 工作区选择页（集成 Tauri 文件夹对话框）
- `OnboardingDeviceName` — 设备名输入页
- `OnboardingComplete` — 完成页（展示摘要）
- `OnboardingStepper` — 步骤指示器组件

### 设计参考

参考 `dev-notes/design/10-ui-requirements.md` 中 Onboarding 部分。

## 技术方案

### 前端

- 使用 Zustand 管理 onboarding 状态（当前步骤、用户输入）
- 步骤间滑动/淡入动画过渡
- 完成后将 onboarding 状态持久化（标记已完成）

### 后端

- `#[tauri::command] fn select_workspace_dir()` — 弹出文件夹选择对话框，返回路径
- `#[tauri::command] fn init_workspace(path, device_name)` — 初始化 `.swarmnote/` 目录，创建 workspace.db，生成 Stronghold 密钥，返回 PeerId
- `#[tauri::command] fn get_onboarding_status()` — 检查是否已完成 onboarding

### 数据结构

```rust
struct OnboardingResult {
    workspace_path: String,
    device_name: String,
    peer_id: String,
}
```

## 验收标准

- [ ] 首次启动显示引导流程，非首次启动直接进入主界面
- [ ] 可通过系统对话框选择工作区目录
- [ ] 选择的目录下自动创建 `.swarmnote/` 子目录和 `workspace.db`
- [ ] 设备名称默认填充系统主机名，用户可修改
- [ ] 完成页展示工作区路径、设备名、PeerId 摘要
- [ ] 步骤间有流畅的过渡动画

## 任务拆分建议

> 此部分可留空，由 /project plan 自动拆分为 GitHub Issues。

## 开放问题

- Onboarding 完成状态存储在哪里？全局配置 `~/.swarmnote/config.toml` 还是 workspace 内？
  - 建议：存在全局配置，因为 onboarding 是设备级别的，不是工作区级别的
