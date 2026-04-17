//! # swarmnote-core
//!
//! Platform-independent core for SwarmNote. Shared between the Tauri desktop
//! shell (`src-tauri/`) and the future mobile shell (`swarmnote-mobile`, via
//! uniffi-bindgen-react-native).
//!
//! ## Architecture (two-layer Core)
//!
//! * [`AppCore`] — device-level singleton (identity, P2P node, paired devices,
//!   pairing manager, ctrl-topic dispatcher).
//! * [`WorkspaceCore`] — workspace-level unit; desktop may hold many via
//!   `Arc`, mobile holds at most one.
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
//! and the mobile `mobile-core/` wrapper crate.
//!
//! ## Re-export layering
//!
//! Public API is organized into two namespaces:
//!
//! * [`api`] — the normal host-facing API surface. Both desktop and mobile
//!   wrappers should consume only this namespace.
//! * [`internal`] — deep access for the desktop shell's command layer
//!   (network / pairing / sync internals, fs business ops, yjs hydrate).
//!   **Not** intended for FFI wrappers.

pub mod app;
pub mod config;
pub mod device;
pub mod document;
pub mod error;
pub mod events;
pub mod fs;
pub mod identity;
pub mod keychain;
pub mod network;
pub mod pairing;
pub mod protocol;
pub mod workspace;
pub mod yjs;

/// Host-facing API surface. Both `src-tauri` and `mobile-core` should import
/// from here. Stable across patch versions.
pub mod api {
    pub use crate::app::{AppCore, AppCoreBuilder, FsFactory, WatcherFactory};
    pub use crate::device::{ConnectionType, Device, DeviceFilter, DeviceListResult, DeviceStatus};
    pub use crate::document::{
        title_from_rel_path, CreateFolderInput, DocumentCrud, UpsertDocumentInput,
    };
    pub use crate::error::{AppError, AppResult};
    pub use crate::events::{AppEvent, EventBus};
    pub use crate::fs::{
        FileEvent, FileEventCallback, FileSystem, FileTreeNode, FileWatcher, LocalFs,
    };
    pub use crate::identity::{DeviceInfo, IdentityManager};
    pub use crate::keychain::KeychainProvider;
    pub use crate::network::NodeStatus;
    pub use crate::pairing::{PairedDeviceInfo, PairingCodeInfo};
    pub use crate::workspace::{WorkspaceCore, WorkspaceInfo};
    pub use crate::yjs::manager::{OpenDocResult, ReloadStatus, YDocManager};
}

/// Deep-access module for the desktop shell. Exposes concrete network /
/// pairing / sync types and raw fs / yjs operations that FFI wrappers
/// should NOT depend on. Keeping this namespaced forces deliberate
/// `use swarmnote_core::internal::...` imports in `src-tauri`.
pub mod internal {
    pub use crate::device::DeviceManager;
    pub use crate::fs::ops;
    pub use crate::network::{AppNetClient, NetManager};
    pub use crate::pairing::{PairingManager, ShareCodeRecord};
    pub use crate::workspace::ensure_workspace_row;
    pub use crate::workspace::sync::{AppSyncCoordinator, WorkspaceSync};
    pub use crate::yjs::doc_state;
}
