use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tracing::warn;

// ── 枚举 ──

/// 托盘内部状态：节点是否运行 + 连接设备数
#[derive(Clone, Copy)]
pub enum TrayNodeStatus {
    Stopped,
    Running { peer_count: usize },
}

/// 托盘图标种类，编译期保证不会拼错
#[derive(Clone, Copy)]
enum IconKind {
    Gray,
    Normal,
    #[allow(dead_code)]
    Yellow,
}

impl IconKind {
    fn load(self) -> Image<'static> {
        let bytes: &[u8] = match self {
            Self::Normal => include_bytes!("../icons/tray/icon-normal.png"),
            Self::Yellow => include_bytes!("../icons/tray/icon-yellow.png"),
            Self::Gray => include_bytes!("../icons/tray/icon-gray.png"),
        };
        Image::from_bytes(bytes).expect("Failed to load tray icon")
    }
}

/// 菜单项 ID，用枚举代替散落的字符串常量
enum MenuAction {
    Open,
    ToggleSync,
    Settings,
    Quit,
}

impl MenuAction {
    const STATUS_ID: &str = "status";

    fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::ToggleSync => "toggle-sync",
            Self::Settings => "settings",
            Self::Quit => "quit",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "toggle-sync" => Some(Self::ToggleSync),
            "settings" => Some(Self::Settings),
            "quit" => Some(Self::Quit),
            _ => None,
        }
    }
}

// ── TrayManager：封装所有托盘状态和行为 ──

/// 系统托盘管理器，封装 TrayIcon 及其状态更新逻辑
pub struct TrayManager {
    tray: TrayIcon,
    app: AppHandle,
    status: TrayNodeStatus,
    /// 最后一个被隐藏到托盘的工作区窗口 label
    last_hidden_label: String,
}

impl TrayManager {
    /// 在 `setup` 中创建托盘并注册到 Tauri State
    pub fn init(app: &AppHandle) -> tauri::Result<()> {
        let status = TrayNodeStatus::Stopped;
        let menu = Self::build_menu(app, status)?;

        let tray = TrayIconBuilder::with_id("main-tray")
            .icon(IconKind::Gray.load())
            .tooltip("SwarmNote")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(Self::on_menu_event)
            .on_tray_icon_event(Self::on_tray_icon_event)
            .build(app)?;

        let manager = Self {
            tray,
            app: app.clone(),
            status,
            last_hidden_label: "main".to_string(),
        };
        app.manage(tokio::sync::Mutex::new(manager));
        Ok(())
    }

    /// 记录最后被隐藏到托盘的工作区窗口
    pub fn set_last_hidden(&mut self, label: &str) {
        self.last_hidden_label = label.to_string();
    }

    /// 更新节点状态 — 唯一的公开状态变更入口
    pub fn set_status(&mut self, status: TrayNodeStatus) {
        self.status = status;
        self.refresh_icon();
        self.refresh_menu();
    }

    fn refresh_icon(&self) {
        let kind = match self.status {
            TrayNodeStatus::Running { .. } => IconKind::Normal,
            TrayNodeStatus::Stopped => IconKind::Gray,
        };
        let _ = self.tray.set_icon(Some(kind.load()));
    }

    /// 恢复最后隐藏的工作区窗口（托盘左键 / "打开 SwarmNote"）
    fn restore_window(&self) {
        let label = &self.last_hidden_label;
        if let Some(window) = self.app.get_webview_window(label) {
            let _ = window.show();
            let _ = window.set_focus();
        } else if label != "main" {
            if let Some(window) = self.app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }

    fn refresh_menu(&self) {
        if let Ok(menu) = Self::build_menu(&self.app, self.status) {
            let _ = self.tray.set_menu(Some(menu));
        }
    }

    fn build_menu(app: &AppHandle, status: TrayNodeStatus) -> tauri::Result<Menu<tauri::Wry>> {
        let menu = Menu::new(app)?;

        // 状态行（不可点击）
        let status_text = match status {
            TrayNodeStatus::Running { peer_count: 0 } => "P2P 已连接".to_string(),
            TrayNodeStatus::Running { peer_count } => format!("P2P 已连接 · {peer_count} 台设备"),
            TrayNodeStatus::Stopped => "P2P 未连接".to_string(),
        };
        menu.append(&MenuItem::with_id(
            app,
            MenuAction::STATUS_ID,
            &status_text,
            false,
            None::<&str>,
        )?)?;

        menu.append(&PredefinedMenuItem::separator(app)?)?;

        menu.append(&MenuItem::with_id(
            app,
            MenuAction::Open.as_str(),
            "打开 SwarmNote",
            true,
            None::<&str>,
        )?)?;

        let toggle_text = match status {
            TrayNodeStatus::Running { .. } => "暂停同步",
            TrayNodeStatus::Stopped => "恢复同步",
        };
        menu.append(&MenuItem::with_id(
            app,
            MenuAction::ToggleSync.as_str(),
            toggle_text,
            true,
            None::<&str>,
        )?)?;

        menu.append(&PredefinedMenuItem::separator(app)?)?;

        menu.append(&MenuItem::with_id(
            app,
            MenuAction::Settings.as_str(),
            "设置",
            true,
            None::<&str>,
        )?)?;

        menu.append(&MenuItem::with_id(
            app,
            MenuAction::Quit.as_str(),
            "退出 SwarmNote",
            true,
            None::<&str>,
        )?)?;

        Ok(menu)
    }

    // ── 事件处理 ──

    fn on_menu_event(app: &AppHandle, event: MenuEvent) {
        let Some(action) = MenuAction::from_str(event.id().as_ref()) else {
            return;
        };
        match action {
            MenuAction::Open => {
                if let Some(state) = app.try_state::<TrayManagerState>() {
                    if let Ok(mgr) = state.try_lock() {
                        mgr.restore_window();
                    }
                }
            }
            MenuAction::ToggleSync => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    toggle_sync(&handle).await;
                });
            }
            MenuAction::Settings => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::workspace::commands::open_settings_window(
                        handle,
                        Some("network".into()),
                    )
                    .await
                    {
                        warn!("Failed to open settings window: {e}");
                    }
                });
            }
            MenuAction::Quit => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let net_state = handle.state::<crate::network::NetManagerState>();
                    let mut guard = net_state.lock().await;
                    if let Some(manager) = guard.take() {
                        manager.shutdown().await;
                    }
                    drop(guard);
                    handle.exit(0);
                });
            }
        }
    }

    fn on_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            if let Some(state) = app.try_state::<TrayManagerState>() {
                if let Ok(mgr) = state.try_lock() {
                    mgr.restore_window();
                }
            }
        }
    }
}

// ── Tauri State 类型别名 ──

pub type TrayManagerState = tokio::sync::Mutex<TrayManager>;

// ── 私有辅助 ──

async fn toggle_sync(app: &AppHandle) {
    let net_state = app.state::<crate::network::NetManagerState>();
    let mut guard = net_state.lock().await;

    if let Some(manager) = guard.take() {
        // 节点正在运行 → 停止
        manager.shutdown().await;
        drop(guard); // 释放锁后再更新托盘
        let _ = app.emit("node-stopped", ());
        if let Some(state) = app.try_state::<TrayManagerState>() {
            state.lock().await.set_status(TrayNodeStatus::Stopped);
        }
        tracing::info!("P2P node stopped via tray");
    } else {
        // 节点未运行 → 启动（释放锁，启动过程需要时间）
        drop(guard);
        let keypair = app
            .state::<crate::identity::IdentityState>()
            .keypair
            .clone();
        let db = app
            .state::<crate::workspace::state::DbState>()
            .devices_db
            .clone();
        if let Err(e) =
            crate::network::commands::do_start_p2p_node(app, &net_state, keypair, db).await
        {
            warn!("Failed to start P2P node from tray: {e}");
        }
    }
}
