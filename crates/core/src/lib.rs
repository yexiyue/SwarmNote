//! # swarmnote-core
//!
//! Platform-independent core for SwarmNote. Shared between the Tauri desktop
//! shell (`src-tauri/`) and the future mobile shell (`swarmnote-mobile`, via
//! uniffi-bindgen-react-native).
//!
//! ## Architecture (two-layer Core)
//!
//! * [`AppCore`] — device-level singleton (identity, P2P node, paired devices,
//!   pairing manager, ctrl-topic dispatcher). **Skeleton in PR #1; populated
//!   in PR #3.**
//! * `WorkspaceCore` — workspace-level unit; desktop may hold many via `Arc`,
//!   mobile holds at most one. **Populated in PR #2.**
//!
//! Platform-specific side-effects are injected through four traits, each
//! in its domain module:
//!
//! * [`fs::FileSystem`] + [`fs::FileWatcher`]
//! * [`events::EventBus`]
//! * [`keychain::KeychainProvider`]
//!
//! This crate is intentionally free of `tauri`, `keyring`, `notify`, or any
//! host-specific dependency — those live in the desktop `src-tauri/src/platform/`
//! and (future) mobile `mobile-core/` crates.

pub mod app;
pub mod config;
pub mod document;
pub mod error;
pub mod events;
pub mod fs;
pub mod identity;
pub mod keychain;
pub mod protocol;
pub mod workspace;
pub mod yjs;

// Top-level re-exports — host code can write
//   use swarmnote_core::{AppCore, FileSystem, EventBus, AppEvent, ...};
// without having to chase down internal module paths.
pub use app::AppCore;
pub use document::{title_from_rel_path, CreateFolderInput, DocumentCrud, UpsertDocumentInput};
pub use error::{AppError, AppResult};
pub use events::{AppEvent, EventBus, NetworkStatus};
pub use fs::{FileEvent, FileEventCallback, FileSystem, FileTreeNode, FileWatcher, LocalFs};
pub use identity::{DeviceInfo, IdentityManager};
pub use keychain::KeychainProvider;
pub use workspace::{WorkspaceCore, WorkspaceInfo};
pub use yjs::manager::{OpenDocResult, ReloadStatus, YDocManager};
